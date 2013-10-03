#[desc = "A simple IRC bot written in Rust."];
#[license = "MIT"];

extern mod extra;
use extra::getopts::{getopts, optopt};
use lib::Bot;
use std::os::args;
use std::rand::Rng;
use std::rt::io::Reader;
use std::rt::io::timer::Timer;

mod lib;

static BORED: [&'static str, ..10] = [
    "so...",
    "hey guys? question",
    "bananas, you POS",
    "and then he was like 'oh chaz you so funny'",
    "but for real guys",
    "tough crowd tonight.",
    "i just had the craziest idea",
    "when do you think they'll notice we're in here?",
    "how do I have time for this ugh",
    "haha that's what she said this one time",
];

enum Key {
    Me,
    Nick(~str),
    Ping,
    Server,
}

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

    let (port, chan) = stream();
    do spawn {
        let mut timer = Timer::new().unwrap();
        loop {
            timer.sleep(60000 * 5); // 5 min
            if !chan.try_send(()) { break; }
        }
    }

    let me = chaz.nick.clone();
    let parse_key = |key: &str| {
        if key == ":concrete.mozilla.org" {
            Server
        } else if key == "PING" {
            Ping
        } else if key == ":" + me {
            Me
        } else {
            let name_end = key.find('!')
                .expect(format!("unrecognized key: {}", key));
            let nick = key.slice(1, name_end).to_owned();
            Nick(nick)
        }
    };
    loop {
        if (port.peek()) {
            port.recv();
            let say = chaz.rng.choose(BORED).to_owned();
            chaz.say(say);
        }
        match chaz.read_line() {
            Some(line) => {
                print!("from server: {}", line);

                let line_split: ~[&str] = line.splitn_iter(' ', 1).collect();

                // Possible keys: PING | :[<nick>!.* | concrete.mozilla.org | 
                let key = line_split[0];
                let content = line_split[1].trim();

                match parse_key(key) {
                    Server => (), // formalities
                    Ping => chaz.writeln("PONG " + content),
                    Me => if content.starts_with("MODE") && !chaz.joined {
                        chaz.join_chan();
                        chaz.converse(me);
                    },
                    Nick(name) => chaz.respond_to(name, content),
                }
            }
            None => return,
        }
    }
}
