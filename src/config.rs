use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use toml;
use reqwest;

#[derive(Debug, Deserialize)]
pub struct Config{
    pub about_arg: String,
    pub about_notice: String,

    pub sageru_url: String,
    pub sageru_port: u16,
    pub sageru_name: String,
    pub sageru_channel:String,
    
    pub vichan_pipe_uri: String,
    pub vichan_post_url: String,
    pub vichan_post_fn: String,
    pub vichan_post_rate: u64,
    pub verification_pass:String
}

pub fn parse_toml_file(p:String) -> Config{
    let mut h:File = File::open(p).expect("Config file could not be opened");
    let mut f:String = String::new();
    h.read_to_string(&mut f).expect("Config File could not be read");
    
    let mut config:Config = toml::from_str(&f).expect("Failed to parse TOML");
    let r = reqwest::blocking::Client::new()
        .get(config.vichan_post_url.to_owned() + &config.vichan_post_fn)
        .send()
        .unwrap();
    config.about_notice = format!("{} ({})" , config.about_notice , r.text().unwrap().trim());
    println!("{}" , config.about_notice);
    config
}
