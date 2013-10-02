#[desc = "A simple IRC bot written in Rust."];
#[license = "MIT"];

extern mod extra;
use extra::container::Deque;
use extra::dlist::DList;
use std::rand;
use std::rand::{IsaacRng, Rng};
use std::rt::io::TcpStream::connect;
use std::rt::io::{io_error, IoError, OtherIoError, TcpStream, Writer};
use std::str::from_utf8;
use std::rt::io::{Reader};

pub struct Bot {
    nick: ~str,
    channel: ~str,
    conn: TcpStream,
    priv buf: [u8, ..1024],
    priv unread: DList<~str>,
    priv rng: IsaacRng,
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
                rng: rand::rng(),
            }
        }
    }

    pub fn init(&mut self) {
        let nick = self.nick.clone();
        self.writeln(format!("USER {0:s} 8 * :{0:s}", nick));
        self.writeln(format!("NICK {:s}", nick));
    }

    pub fn writeln(&mut self, msg: ~str) {
        write!(&mut self.conn as &mut Writer, "{}\r\n", msg);
    }

    pub fn say(&mut self, msg: ~str) {
        let channel = self.channel.clone();
        let msg = format!("PRIVMSG {:s} :{:s}", channel, msg);
        self.writeln(msg.clone());
        println!("me: {}", msg);
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
                                self.unread.push_back(line.to_owned())
                            }
                            if !read.ends_with("\n") {
                                let last = self.unread.pop_back().unwrap();
                                self.unread.push_back(last.trim().to_owned());
                            }
                        }
                    }
                }
                Some(line) => next_line = next_line + line,
            }
        }
        Some(next_line)
    }
    
    pub fn converse(&mut self, guy: &str) {
        let say = says[self.rng.gen_integer_range(0, says.len())].to_owned();
        let say = if say.starts_with(" ") {
            guy + say
        } else if say.ends_with(" ") {
            say + guy
        } else {
            say
        };
        self.say(say);
    }
}

static says: [&'static str, ..7] = [
    "Ohai there",
    "doge",
    "wow",
    "such chaz",
    "very chaz, ",
    " pls",
    "                       :}",
];

