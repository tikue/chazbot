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

/// Informally called "chaz," Bot is an IRC bot
/// for irc.mozilla.org.
pub struct Bot {
    /// IRC Nick to register.
    priv nick: ~str,
    /// IRC Channel to join.
    priv channel: ~str,
    /// Connection to the IRC server.
    priv conn: TcpStream,
    /// Whether or not Bot has joined <channel> yet.
    priv joined: bool,
    /// RNG for randomly choosing things to say.
    priv rng: IsaacRng,
    /// Buffer for reading data from the server.
    priv buf: [u8, ..1024],
    /// Processed but unread lines sent from server.
    priv unread: DList<~str>,
}

impl Bot {
    /// Construct a new bot. Automatically connects to the server
    /// at the given address.
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

    /// Initialize user name and nick, spawn the event timer,
    /// and enter main interaction routine.
    pub fn init(&mut self) {
        let me = self.nick.clone();
        self.writeln(format!("USER {0:s} 8 * :{0:s}", me));
        self.writeln(format!("NICK {:s}", me));

        //----- Timer for "spontaneous" declarations
        let (port, chan) = stream();
        do spawn {
            let mut timer = Timer::new().unwrap();
            loop {
                timer.sleep(60000 * 5); // 5 min
                if !chan.try_send(()) { break; }
            }
        }
        self.interact(port);
    }

    /// Main routine in which chazbot responds to a channel
    /// in various ways.
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
                    let split = line.splitn_iter(is_ws, 1).to_owned_vec();
                    let (key, content) = (split[0], split[1]);

                    match self.parse_key(key) {
                        Server => (), // formalities
                        Ping => self.writeln("PONG " + content),
                        Me => if content.starts_with("MODE") && !self.joined {
                            self.join();
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
    
    /// Read next line from server. Returns None if connection
    /// is closed.
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

    /// Sends a message to the server, appending the proper carriage return.
    pub fn writeln(&mut self, msg: ~str) {
        println!("me         : {}", msg.as_slice());
        write!(&mut self.conn as &mut Writer, "{}\r\n", msg);
    }

    /// Say something in the channel currently residing in.
    /// Does nothing if a channel has not yet been joined.
    pub fn say(&mut self, msg: ~str) {
        if !self.joined { return; }
        let channel = self.channel.clone();
        let msg = format!("PRIVMSG {:s} :{:s}", channel, msg);
        self.writeln(msg.clone());
    }

    /// Direct a message at the specified nick.
    pub fn converse(&mut self, name: &str) {
        let say = self.rng.choose(SAYS);
        self.say(say(name));
    }

    /// Join the bot's channel if it hasn't already.
    pub fn join(&mut self) {
        if self.joined { return; }
        let join_expr = "JOIN " + self.channel;
        self.writeln(join_expr);
        self.joined = true;
    }

    /// Respond to a message's content appropriately given the
    /// type of message.
    pub fn respond_to(&mut self, name: &str, content: &str) {
        match content.splitn_iter(is_ws, 2).to_owned_vec() {
            ["JOIN", .. _content] => if name != self.nick {
                self.say(format!("and then {0} made the stupidest fa-- \
                oh, hey {0}, didn't see you there buddy", name))
            },
            ["PRIVMSG", _chan, content] => {
                self.respond_to_privmsg(name, content.trim_left_chars(&':'))
            },
            ["QUIT", .. _content] => {
                self.say(format!("did you see the way {} left like that?", name));
                self.say(~"that's just, like, classic him.");
            },
            ["TOPIC", .. _content] => {
                self.say(format!("what was wrong with the old topic, {}?", name));
            }
            _ => unreachable!(),
        }
    }

    /// Respond to a message from the given nick.
    fn respond_to_privmsg(&mut self, name: &str, content: &str) {
        let content_lower = content.to_ascii_lower();
        let content_spaceless: ~str = content_lower.word_iter()
            .to_owned_vec().concat();
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
                self.say(~"lol i don't even know what to say lol \
                            bc i'm a bot :-(");
            }
            return;
        }

        match self.rng.choose(possibilities) {
            MyName => {
                if self.rng.gen_weighted_bool(10) {
                    self.say(format!("and then {} was all like", name));
                    self.say(format!("\"{}\"", content.trim_right()));
                    self.say(format!("you're better than that, {0} -- that's \
                            what your mom would always say: \"You're better \
                            than that, {0}.\"", name));
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

    /// Parse a message's key.
    pub fn parse_key(&self, key: &str) -> Key {
        match key {
            ":concrete.mozilla.org" => Server,
            "PING" => Ping,
            key if key == ":" + self.nick => Me,
            _ => Nick(key.split_iter('!').next().unwrap()
                .trim_left_chars(&':').to_owned()),
        }
    }
}

/// The various types of message keys.
/// Possible keys: PING | :[<nick>!.* | concrete.mozilla.org]
enum Key {
    Me,
    Nick(~str),
    Ping,
    Server,
}

#[deriving(Clone)]
/// Different possibilities for types of responses.
enum ResponseTo {
    MyName,
    Laff,
    BigLaff,
    Wow,
}

/// Convenience function because |c: char| c.is_whitespace()
/// is too verbose.
fn is_ws(c: char) -> bool {
    c.is_whitespace()
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

static BIG_LAFFS: [&'static str, ..3] = [
    "rofl",
    "lmao",
    "LOL",
];

static BORED: [&'static str, ..10] = [
    "are we playing the silence game",
    "hey guys? question",
    "bananas, you there? I got a question about my implementation",
    "wait I have to think about the phrasing of this give me a sec.",
    "and then he was like 'oh chaz you so funny'",
    "ok, HERE IT IS:",
    "but for real guys",
    "i just had the craziest idea",
    "do i make any of you uncomfortable?",
    "so before I tell you my idea, there's one thing I need to know...",
];
