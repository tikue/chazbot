#[desc = "A simple IRC bot written in Rust."];
#[license = "MIT"];

extern mod extra;
use extra::getopts::{getopts, optopt};
use lib::Bot;
use std::ascii::StrAsciiExt;
use std::os::args;
use std::rand::Rng;
use std::rt::io::Reader;
use std::rt::io::timer::Timer;

mod lib;

static BORED: [&'static str, ..10] = [
    "so...",
    "hey guys? question",
    "bananas you POS",
    "and then he was like 'oh chaz you so funny'",
    "but for real guys",
    "tough crowd tonight.",
    "i just had the craziest idea",
    "when do you think they'll notice we're in here?",
    "how do I have time for this ugh",
    "haha that's what she said this one time",
];

static LAFFS: [&'static str, ..5] = [
    "lol",
    "haha",
    "hehe",
    "jaja",
    "hoho"
];

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

    let (port, chan) = stream();
    do spawn {
        let mut timer = Timer::new().unwrap();
        loop {
            timer.sleep(60000 * 5); // 5 min
            chan.send(());
        }
    }

    loop {
        if (port.peek()) {
            port.try_recv();
            let say = chaz.rng.choose(BORED).to_owned();
            chaz.say(say);
        }
        match chaz.read_line() {
            Some(line) => {
                let no_space = line.replace(" ", "").to_ascii_lower();
                let lower_line = line.to_ascii_lower();
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
                } else if lower_line.contains(chaz.nick)
                    || no_space.contains(chaz.nick) {
                    let guy = line.slice(1, line.find('!').unwrap());
                    if chaz.rng.gen_weighted_bool(10) {
                        chaz.say(format!("and then {} was all like", guy));
                        let sep = format!("{} :", chaz.channel);
                        let v: ~[&str] = line.split_str_iter(sep).collect();
                        let said = v[1];
                        chaz.say(format!("\"{}\"", said.slice_to(said.len() - 2)));
                        chaz.say(~"like i even GAF");
                    } else {
                        chaz.converse(guy);
                    }
                } else if LAFFS.iter().any(|&laff| line.contains(laff)) {
                    let laff = chaz.rng.choose(LAFFS).to_owned();
                    chaz.say(laff);
                } else if line.contains("rofl") || line.contains("LOL") {
                    chaz.say(~"LOLOLOLLLLLL");
                } else if line.to_ascii_lower().contains("wow") {
                    chaz.say(~"wowowow doogie hauser");
                } else if chaz.rng.gen_weighted_bool(25) {
                    chaz.say(~"sharif don't liiiiike it lolol u know? ino uno");
                }
            }
            None => return,
        }
    }
}
