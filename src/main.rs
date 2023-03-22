mod sageru;
mod vichan;
mod config;

use std::{thread, env};
use std::sync::mpsc;



/// Launch the function, initiating a connection to IRC Sageru, passing the handler and a channel IO into another thread where it loops and waits for new inputs on the IRC.
/// Upon a certain chat message (!relay) it writes to the channel the channel info and connected thread(As obtained by communication with the Vichan API post endpoint). 
/// This reader thread should log what the relay writes to the server, but doesn't send it to the Vichan post channel.
/// A second thread exists to handle requests to write messages to the IRC. It gets messages from the pipe thread.
/// Therefore there are two channels. One for messages to be written to IRC and one for messages read from IRC.
/// 
/// A thread exists to handle the IB Vichan which waits on messages into it's channel from IRC.
/// It is connected to the IRC Sageru reader channel. 
/// This channel sends content in batches, pausing every N seconds to collect new messages from IRC.
/// It sends via HTTP to the Vichan posting endpoint.
/// Another thread exists for the IRC writer which is sent messages from vichan from Pipe.
/// When the pipe has new inputs it writes the data to the IRC writter channel
/// 
/// For your information, Vichan is an imageboard server which is sent user generated content and displays other people's content.
/// Sageru is an anonymous IRC channel with no names other than Anonymous and your own(which is hidden of course).
/// The Vichan imageboard server takes in the sent messages and writes it to a configured location. 
/// The IRC reader will send everything and as much quantity as configured. 
/// It's up to vichan to decide what is allowed and not.
/// This can be used as a public IRC log in the future.
/// 
/// The main thread waits and does nothing after initiating the pipe thread and the IRC thread. 
/// This is a program interfacing between the IRC network and the pipe communcation feed using various libraries.
/// There are four threads and two IO channels as well as the ability to write to one file and a pipe.
/// The Sageru IRC reader channel writes to a log
/// The Vichan file creates a pipe which is written to from vichan.
/// In the future this program will be incorperated into the Tsukuyomi imageboard engine as part of it's ability to integrate with IM networks outside of it's own internatal one
fn main() {
    // Get and parse config json
    let f:String = env::args()
        .skip(1)
        .next()
        .expect("A file argument was not provided");
    let c = config::parse_toml_file(f);
    // Vi & Sageru channels
    let (sageru_sender, sageru_reciever) = mpsc::channel::<String>();
    let (vi_sender, vi_reciever) = mpsc::channel::<String>();

    // start threads with channels and config borrow
    sageru::start(&c , sageru_sender , vi_reciever);
    vichan::start(&c , vi_sender , sageru_reciever);

    // no need for the main thread
    println!("Vichan-Sageru relay initialization finished. Waiting for termination.");
    thread::park();
}

