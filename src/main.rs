#[desc = "A simple IRC bot written in Rust."];
#[license = "MIT"];

extern mod extra;
use extra::getopts::{getopts, optopt};
use lib::Bot;
use std::os::args;

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
}
