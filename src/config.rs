
use std::fs::File;
use std::io::Read;
use serde_json;

#[derive(Debug)]
pub struct Config{
    about_arg: String,
    sageru_url: String,
    sageru_name: String,
    sageru_channel:String,
    
    vichan_pipe_uri: String,
    vichan_post_url: String,
    verification_pass:String
    
}

pub fn parse_json_file(p:String) -> Config{
    let mut h:File = File::open(p).expect("Config file could not be opened");
    let mut f:String = String::new();
    h.read_to_string(&mut f).expect("Config File could not be read");

    let j:serde_json::Value = serde_json::from_str(&f).expect("Failed to parse JSON");
    
    let a = j.get("about_arg").expect("about_arg not found in JSON");
    let u = j.get("sageru_url").expect("sageru_url not found in JSON");
    let n = j.get("sageru_name").expect("sageru_name not found in JSON");
    let c = j.get("sageru_channel").expect("sageru_channel not found in JSON");
    
    let pipe = j.get("vichan_pipe_uri").expect("vichan_pipe_uri not found in JSON");
    let post= j.get("vichan_post_url").expect("vichan_post_url not found in JSON");
    let verif= j.get("verification_pass").expect("verification_pass not found in JSON");

    Config{ 
        about_arg: a.to_string(),
        sageru_url: u.to_string(),
        sageru_name: n.to_string(),
        sageru_channel: c.to_string(),
        
        vichan_pipe_uri: pipe.to_string(),
        vichan_post_url: post.to_string(),
        verification_pass: verif.to_string(),
    }
}
