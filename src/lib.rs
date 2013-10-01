#[desc = "A simple IRC bot written in Rust."];
#[license = "MIT"];

use std::rt::io::TcpStream;

pub struct Bot {
    nick: &'static str,
    channel: &'static str,
    conn: TcpStream,
}
