#[desc = "A simple IRC bot written in Rust."];
#[license = "MIT"];

extern mod extra;
use extra::getopts::{getopts, optopt};
use lib::Bot;
use std::os::args;
use std::rt::io::{Reader};

mod lib;

fn main() {
    let opts = [
        optopt("addr"),
        optopt("chan"),
        optopt("nick"),
    ];    
    let args = args();
    let matches = match getopts(args.tail(), opts) {
        Ok(m) => m,
        Err(f) => fail!(f.to_err_msg()),
    };
    let addr = matches.opt_str("addr").or(Some(~"63.245.216.214:6667")).unwrap();
    let chan = matches.opt_str("chan").or(Some(~"#very-student")).unwrap();
    let nick = matches.opt_str("nick").or(Some(~"chaz")).unwrap();

    let mut chaz = Bot::new(nick, chan, addr).unwrap();
    println("starting");
    chaz.init();
    let mut joined = false;

    loop {
        match chaz.read_line() {
            Some(line) => {
                println!("from server: {}", line);
                if line.contains("MODE") {
                    if !joined {
                        joined = true;
                        let join = format!("JOIN {}", chaz.channel);
                        chaz.writeln(join);
                    }
                } else if line.starts_with("PING") {
                    let reply = format!("PONG{}", line.slice_from(4));
                    println!("me: {}", reply);
                    chaz.writeln(reply);
                } else if line.contains("mozilla.org") {
                    continue;
                } else if line.contains(format!("{}: PING", chaz.nick)) {
                    chaz.say(~"POOOOOOOOOONG!!!!");
                } else if line.contains(format!("{}", chaz.nick)) {
                    chaz.converse(line.slice_to(line.find('!').expect("wat")));
                } else if line.contains("lol") || line.contains("haha")
                    || line.contains("hehe") {
                    chaz.say(~"lolol");
                } else if line.contains("rofl") || line.contains("LOL") {
                    chaz.say(~"LOLOLOLLLLLL");
                }
            }
            None => return,
            
        }
    }
}
