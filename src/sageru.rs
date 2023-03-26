use crate::config::Config;

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::net::TcpStream;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc , Mutex};

use std::thread;

pub fn start(c:&Config , sageru_sender:Sender<String> , vi_reciever:Receiver<String>){


    let  (mut reader, writter) = connect(&c.sageru_url , &c.sageru_port , &c.sageru_name, &c.sageru_channel);

    let mut log = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open("./sageru-log.txt")
        .expect("Could not open Log file");

    let w = Arc::new(Mutex::new(writter));

    // Wait on a message
    let about = c.about_arg.to_owned();
    let about_msg = c.about_notice.to_owned();
    let chan = c.sageru_channel.to_owned();
    let w_read = w.clone();
    thread::spawn(move || {
        println!("Waiting for Sageru Messages");
        let mut line = String::new();
        while let Ok(_) = reader.read_line(&mut line) {
            print!("IRC: {}", line);
            if is_linkbot(&line){

            } else if is_about(&line , &chan,  &about) {
                w_read.lock().unwrap().write(format!("PRIVMSG {} :{}\r\n", chan , about_msg).as_bytes()).expect("Could not write to about response");
                w_read.lock().unwrap().flush().unwrap();
            } else if is_chatter(&line) {
                log.write_all(line.as_bytes()).expect("Could not write to log");
                
                match sageru_sender.send(line.to_owned()){
                    Err(e) => println!("Sageru - IRC => Vi Failed: {}", e),
                    __ => {}
                }
            }
            line.clear();
        }
        panic!("IRC Thread abruptly terminated");
    });

    // Write from Vi
    let chan = c.sageru_channel.to_owned();
    let w_write = w.clone();
    thread::spawn(move || {
        loop {
            match vi_reciever.recv(){
                Ok(m) => {
                    w_write.lock().unwrap().write(format!("PRIVMSG {} :{}\r\n", chan , m).as_bytes()).expect("Could not write to test response");
                    w_write.lock().unwrap().flush().unwrap();
                },
                Err(e) => {
                    // println!("Vi => IRC Reciever Failed: {}" , e);
                }
            }
        }
    });

    
}

fn is_chatter(message:&str) -> bool{
    message.starts_with(":Anonymous!~anonymous@unknown PRIVMSG")
}

fn is_about(message:&str , chan:&str , cmd:&str) -> bool{
    message.starts_with(&format!(":Anonymous!~anonymous@unknown PRIVMSG {} :{}" , chan , cmd))
}

fn is_linkbot (message: &str ) -> bool{
    message.contains("@ kissu.moe")
}

fn connect(url:  &str, port: &u16, name:&str, channel:&str) -> ( BufReader<TcpStream> , BufWriter<TcpStream>){

    println!("{} {}" , url, port);
    let mut stream = TcpStream::connect((url, *port)).expect("Could not connect to server location");
    
    writeln!(stream, "NICK {}", name).expect("IRC Nick error");
    writeln!(stream, "USER {} 0 * :{}", name, name).expect("IRC User error");
    writeln!(stream, "JOIN {}", channel).expect("IRC Chan Join error");
    writeln!(stream, "PRIVMSG {} :{}", channel , "startup test-FN" ).expect("IRC Chan Join error");
    
    ( BufReader::new(stream.try_clone().unwrap()) , BufWriter::new(stream.try_clone().unwrap()))
}
