#[desc = "A simple IRC bot written in Rust."];
#[license = "MIT"];

extern mod extra;
use extra::container::Deque;
use extra::dlist::DList;
use std::ascii::StrAsciiExt;
use std::rand;
use std::rand::{IsaacRng, Rng};
use std::rt::io::{io_error, IoError, OtherIoError, Reader, TcpStream, Writer};
use std::rt::io::TcpStream::connect;
use std::rt::io::timer::Timer;
use std::str::from_utf8;

pub struct Bot {
    nick: ~str,
    channel: ~str,
    conn: TcpStream,
    priv buf: [u8, ..1024],
    priv unread: DList<~str>,
    joined: bool,
    rng: IsaacRng,
}

impl Bot {
    pub fn new(nick: ~str, channel: ~str, addr: ~str) -> Option<Bot> {
        let addr = match from_str(addr) {
            Some(addr) => addr,
            None => {
                let error = IoError {
                    kind: OtherIoError,
                    desc: "Malformed address",
                    detail: Some(addr),
                };
                io_error::cond.raise(error);
                return None;
            }
        };

        do connect(addr).map_move |conn| {
            Bot {
                nick: nick.clone(),
                channel: channel.clone(),
                conn: conn,
                buf: [0, ..1024],
                unread: DList::new(),
                joined: false,
                rng: rand::rng(),
            }
        }
    }

    pub fn init(&mut self) {
        let nick = self.nick.clone();
        self.writeln(format!("USER {0:s} 8 * :{0:s}", nick));
        self.writeln(format!("NICK {:s}", nick));

        let (port, chan) = stream();

        //----- Timer for "spontaneous" declarations
        do spawn {
            let mut timer = Timer::new().unwrap();
            loop {
                timer.sleep(60000 * 5); // 5 min
                if !chan.try_send(()) { break; }
            }
        }
        self.interact(port);
    }

    pub fn writeln(&mut self, msg: ~str) {
        println!("me         : {}", msg.as_slice());
        write!(&mut self.conn as &mut Writer, "{}\r\n", msg);
    }

    pub fn say(&mut self, msg: ~str) {
        let channel = self.channel.clone();
        let msg = format!("PRIVMSG {:s} :{:s}", channel, msg);
        self.writeln(msg.clone());
    }

    pub fn read_line(&mut self) -> Option<~str> {
        let mut next_line = match self.unread.pop_front() {
            None => ~"",
            Some(line) => line,
        };
        while !next_line.ends_with("\n") {
            match self.unread.pop_front() {
                None => { // read more
                    match self.conn.read(self.buf) {
                        None => return None, // conn closed
                        Some(bits) => {
                            let read = from_utf8(self.buf.slice_to(bits));
                            for line in read.line_iter().map(|s| s + "\n") {
                                self.unread.push_back(line)
                            }
                            if !read.ends_with("\n") {
                                let last = self.unread.pop_back().unwrap();
                                self.unread.push_back(last.trim().to_owned())
                            }
                        }
                    }
                }
                Some(line) => next_line = next_line + line,
            }
        }
        Some(next_line)
    }

    pub fn interact(&mut self, port: Port<()>) {
        loop {
            if (port.peek()) {
                port.recv();
                let say = self.rng.choose(BORED).to_owned();
                self.say(say);
            }

            match self.read_line() {
                Some(line) => {
                    print!("from server: {}", line);

                    let line_split: ~[&str] = line.splitn_iter(' ', 1).collect();

                    // Possible keys: PING | :[<nick>!.* | concrete.mozilla.org | 
                    let key = line_split[0];
                    let content = line_split[1].trim();

                    match self.parse_key(key) {
                        Server => (), // formalities
                        Ping => self.writeln("PONG " + content),
                        Me => if content.starts_with("MODE") && !self.joined {
                            self.join_chan();
                            let me = self.nick.clone();
                            self.converse(me);
                        },
                        Nick(name) => self.respond_to(name, content),
                    }
                }
                None => return,
            }
        }
    }
    
    pub fn converse(&mut self, name: &str) {
        let say = self.rng.choose(SAYS);
        self.say(say(name));
    }

    pub fn join_chan(&mut self) {
        let join_expr = "JOIN " + self.channel;
        self.writeln(join_expr);
        self.joined = true;
    }

    pub fn respond_to(&mut self, name: &str, content: &str) {
        let content: ~[&str] = content.splitn_iter(' ', 2).collect();
        let (content_key, content) = match content {
            [key, .. content] => (key, content),
            _ => unreachable!(),
        };

        match content_key {
            "JOIN" => if name != self.nick {
                self.say(format!("wow hi {}", name))
            },
            "QUIT" => self.say(format!("wow bye {}", name)),
            "PRIVMSG" => self.respond_to_privmsg(name, content[1].slice_from(1)),
            _ => unreachable!(),
        }
    }

    fn respond_to_privmsg(&mut self, name: &str, content: &str) {
        let content_lower = content.to_ascii_lower();
        let content_spaceless = content_lower.replace(" ", "");
        let mut possibilities = ~[];

        if content_spaceless.contains(self.nick) {
            possibilities.push(MyName);
        }

        if BIG_LAFFS.iter().any(|&laff| content.contains(laff)) {
            possibilities.push(BigLaff);
        } else if LAFFS.iter().any(|&laff| content.contains(laff)) {
            possibilities.push(Laff);
        }

        if content.to_ascii_lower().contains("wow") {
            possibilities.push(Wow);
        }

        if possibilities.is_empty() {
            if self.rng.gen_weighted_bool(25) {
                self.say(~"lol i don't even know what to say lol bc i'm a bot :-(");
            }
            return;
        }

        match self.rng.choose(possibilities) {
            MyName => {
                if self.rng.gen_weighted_bool(10) {
                    self.say(format!("and then {} was all like", name));
                    self.say(format!("\"{}\"", content));
                    self.say(~"like i even GAF");
                } else {
                    self.converse(name);
                }
            }
            Laff => {
                let laff = self.rng.choose(LAFFS).to_owned();
                self.say(laff);
            }
            BigLaff => self.say(~"LOLOLOLLLLLL"),
            Wow => self.say(~"wowowow doogie hauser"),
        }
    }

    pub fn parse_key(&self, key: &str) -> Key {
        if key == ":concrete.mozilla.org" {
            Server
        } else if key == "PING" {
            Ping
        } else if key == ":" + self.nick {
            Me
        } else {
            let name_end = key.find('!')
                .expect(format!("unrecognized key: {}", key));
            let nick = key.slice(1, name_end).to_owned();
            Nick(nick)
        }
    }
}

enum Key {
    Me,
    Nick(~str),
    Ping,
    Server,
}

#[deriving(Clone)]
enum ResponseTo {
    MyName,
    Laff,
    BigLaff,
    Wow,
}

//----- Responses to messages ----->
fn ohai(_name: &str) -> ~str { ~"Ohaiii" }
fn doge(_name: &str) -> ~str { ~"good dogge :]" }
fn wow(_name: &str) -> ~str { ~"wow" }
fn such_chaz(_name: &str) -> ~str { ~"such chaz" }
fn very_chaz(name: &str) -> ~str { format!("very chaz, {}", name) }
fn pls(name: &str) -> ~str { format!("{} pls", name) }
fn dolan(name: &str) -> ~str { format!("{}                       :\\}", name) }
fn doing_it(name: &str) -> ~str { format!("you're doing it, {}", name) }

static SAYS: [extern fn(&str) -> ~str, ..8] = [
    ohai,
    doge,
    wow,
    such_chaz,
    very_chaz,
    pls,
    dolan,
    doing_it,
];

static LAFFS: [&'static str, ..5] = [
    "lol",
    "haha",
    "hehe",
    "jaja",
    "hoho",
];

static BIG_LAFFS: [&'static str, ..2] = [
    "rofl",
    "LOL",
];

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
