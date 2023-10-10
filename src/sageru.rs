use crate::config::Config;

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::net::TcpStream;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc , Mutex};

use std::thread;

use regex::Regex;

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
        loop {
            //println!("LOOP SAG");
            if let Ok(_) = reader.read_line(&mut line) {
                if is_ignored(&line){
                   // println!("IS IGN");
                } else if is_about(&line , &chan,  &about) {
                    let mut about_writter = w_read.lock().unwrap(); 
                    if let Err(_) = about_writter.write(format!("PRIVMSG {} :{}\r\n", chan , about_msg).as_bytes()) {
                        println!("Could not write to about response");
                    } else{
                        _ = about_writter.flush();
                    }
                } else if is_chatter(&line) {
                    if let Err(_) = log.write_all(line.as_bytes()){
                        println!("Could not write to log");
                    };
                    let line = reverse_markup(line.to_owned());
                    match sageru_sender.send(line.to_owned()){
                        Err(e) => println!("Sageru - IRC => Vi Failed: {}", e),
                        __ => {}
                    }
                } else{
					println!("FAIL LN: {}" , line);
				}
                line.clear();
            } else{
                println!("IRC is down!");
				std::process::exit(99);
            }
        } 
    });

    // Write from Vi
    let chan = c.sageru_channel.to_owned();
    let w_write = w.clone();
    thread::spawn(move || {
        loop {
            //println!("LOOP READ VI");
            match vi_reciever.recv(){
                Ok(m) => {
                    println!("OK");
                    let m = m.replace("\n", "    ")
                        .replace("\r", "");
                    // slice into 400 char chunks
                    let mut m = m.as_bytes();
                    let mut m = m.chunks(400);
                    while let Some(chunk) = m.next(){
                        let chunk = foreward_markup(String::from_utf8_lossy(chunk).to_string());
                        let chunk = chunk + "0-@Kissu";
                        let mut vi_writter = w_write.lock().unwrap();
                        if let Err(_) = vi_writter.write(format!("PRIVMSG {} :{}\r\n", chan , chunk).as_bytes()) {
                            println!("Could not write to about response");
                        } else{
                            _ = vi_writter.flush();
                        }
                    }
                    let mut vi_writter = w_write.lock().unwrap();
                    if let Err(_) = vi_writter.write(format!("PRIVMSG {} :{}\r\n", chan , m).as_bytes()) {
                        println!("Could not write to about response");
                    } else{
                        _ = vi_writter.flush();
                    }
                },
                Err(e) => {
                    println!("Vi => IRC Reciever Failed: {}" , e);
                }
            }
        }
    });

    
}

fn foreward_markup(mut msg: String) -> String{

    msg = msg.replace("[b]", "")
    .replace("[/b]", "")
    .replace("[i]", "")
    .replace("[/i]", "")
    .replace("[ul]", "")
    .replace("[/ul]", "")
    .replace("[code]", "")
    .replace("[/code]", "")
    .replace("[s glowblue]", ",12")
    .replace("[s glowpink]", ",13")
    .replace("[s glowgreen]", ",10")
    .replace("[s glowgold]", ",8")
    .replace("[s heading]", "4")
    .replace("[s spoiler]", "1,1")
    .replace("[s quote]", "3")
    .replace("[s yen]", "6")
    .replace("[/s]", "");

    let special_enter = Regex::new(r"\[[su] [^\]]*?\]").unwrap();
    let special_exit = Regex::new(r"\[/[su]\]").unwrap();
    let generic = Regex::new(r"\[/?(det|sum|str|sjis|)\]").unwrap();

    let msg = special_enter.replace_all(&msg, "");
    let msg = special_exit.replace_all(&msg, "");
    let msg = generic.replace_all(&msg, "");

    msg.to_string()
}

fn reverse_markup(msg: String) -> String{

    let color = Regex::new(r"[0-9]*(,[0-9]+)?").unwrap();
    let bold = Regex::new(r"(.*?)(|$|\n)").unwrap();
    let ital = Regex::new(r"(.*?)(|$|\n)").unwrap();
    let under = Regex::new(r"(.*?)(|$|\n)").unwrap();
    let code = Regex::new(r"(.*?)(|$|\n)").unwrap();

    let msg = color.replace_all(&msg, "");
    let msg = bold.replace_all(&msg, "[b]$1[/b]");
    let msg = ital.replace_all(&msg, "[i]$1[/i]");
    let msg = under.replace_all(&msg, "[ul]$1[/ul]");
    let msg = code.replace_all(&msg, "[code]$1[/code]");

    msg.to_string()
}

fn is_chatter(message:&str) -> bool{
    message.starts_with(":Anonymous")
}

fn is_about(message:&str , chan:&str , cmd:&str) -> bool{
    message.starts_with(&format!(":Anonymous!~anonymous@unknown PRIVMSG {} :{}" , chan , cmd))
}

fn is_ignored (message: &str ) -> bool{
    message.contains("@ kissu.moe")
}

fn connect(url:  &str, port: &u16, name:&str, channel:&str) -> ( BufReader<TcpStream> , BufWriter<TcpStream>){

    println!("{} {}" , url, port);
    let mut stream = TcpStream::connect((url, *port)).expect("Could not connect to server location");
    
    writeln!(stream, "NICK {}", name).expect("IRC Nick error");
    writeln!(stream, "USER {} 0 * :{}", name, name).expect("IRC User error");
    writeln!(stream, "JOIN {}", channel).expect("IRC Chan Join error");
    // writeln!(stream, "PRIVMSG {} :{}", channel , "14Kissu-Chat Ver0.0 (3!relay14 for info)" ).expect("IRC Chan Join error");
    
    ( BufReader::new(stream.try_clone().unwrap()) , BufWriter::new(stream.try_clone().unwrap()))
}
