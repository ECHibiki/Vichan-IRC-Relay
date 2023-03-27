use crate::config::Config;
use std::collections::HashMap;
use std::fs::{OpenOptions , File};
use std::io::Read;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc , Mutex};

use std::thread;
use std::time::Duration;

use reqwest;
use tokio::{self , runtime::Runtime};

pub fn start(c:&Config , vi_sender:Sender<String> , sageru_reciever:Receiver<String>){

    // Write to IRC from board 
    // Wait on pipe
    let pipe = c.vichan_pipe_uri.to_owned();
    thread::spawn(move || {
        loop {
            println!("PIPE OPEN {}" ,  pipe);
            let mut fo:Option<File> = None;
            while let None = fo  {
                println!("Open Op");
                match  OpenOptions::new()
                .read(true)
                .open(pipe.to_owned()) {
                    Ok(f) => {
                        println!("Pipe found");
                        fo = Some(f);   
                    },
                    Err(e) => {
                        println!("{e}");
                    }
                }
            }
            
            let mut pipe_file = fo.unwrap();
            let mut output = String::new();
            while let Ok(_) = pipe_file.read_to_string(&mut output) {
                if output == "" {
                    continue
                }
                println!("{output}");
                match vi_sender.send( output.to_owned()) {
                    Ok(_) =>{},
                    Err(e) => {println!("Board Vi => IRC Failed: {e}")}
                }; 
                output.clear();
            }
            println!("PIPE CLOSE");
        }
        
    });

    // Wait on Sageru
    // Create a queue that sends batches every X Ms
    let arc_queue:Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let queue = arc_queue.clone();
    thread::spawn(move || {
        println!("Waiter run");
        loop {
            match sageru_reciever.recv() {
                Ok(m) => {
                    println!("Waiter REC");
                    queue.lock().unwrap().push(m);
                },
                Err(e) => {println!("Board - IRC => Vi Failed: {e}")}
            }
        }
    });
    let cooldown = c.vichan_post_rate;
    let url = c.vichan_post_url.to_owned();
    let func = c.vichan_post_fn.to_owned();
    let pass = c.verification_pass.to_owned();
    thread::spawn(move || {
        println!("Tiemr run");
        loop {
            let r = reqwest::Client::new();
            thread::sleep(Duration::from_secs(cooldown));
            let mut q = arc_queue.lock().unwrap();
            if q.len() == 0 {
                continue;
            }
            let mut hm = HashMap::<&str, String>::new();
            hm.insert("posts" , q.join("\0") );
            hm.insert("pass" , pass.to_owned());

            q.clear();
            let message = r.post(url.to_owned() + &func)
            .form(&hm)
            .send();
            let f = async {
                match message.await{
                    Ok(_m) => (), //println!("{} - {}" , m.status() , m.text().await.unwrap() ), 
                    Err(e) => println!("{e}") 
                }
            };
            let rt = Runtime::new();
            match rt {
                Ok(ru) => ru.block_on(f),
                Err(e) => println!("{e}")            
            }
        }
    });
    
}