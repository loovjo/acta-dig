use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};

use rand::random;

pub struct Worker {
    actors: HashMap<ActorAddr, Box<dyn Actor>>,

    msg_recv: Receiver<Message>,
    pub msg_send: Sender<Message>,
}

impl Worker {
    pub fn new() -> Worker {
        let (msg_send, msg_recv) = channel();
        Worker {
            actors: HashMap::new(),
            msg_recv,
            msg_send,
        }
    }

    pub fn add_actor(&mut self, actor: Box<dyn Actor>) -> ActorAddr {
        let addr = ActorAddr::random();
        self.actors.insert(addr, actor);
        addr
    }

    pub fn step_once(&mut self) -> bool {
        match self.msg_recv.try_recv() {
            Ok(Message { to, cont }) => {
                eprintln!("Sending {:?} to {:?}", cont, to);
                if let Some(act) = self.actors.get_mut(&to) {
                    let ctx = Context {
                        msg_sender: &mut self.msg_send,
                    };
                    act.handle_message(cont, ctx);
                    true
                } else {
                    false
                }
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => false,
            Err(e) => {
                eprintln!("Message channel disconnected: {:?}", e);
                false
            }
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct ActorAddr {
    pub addr: u32,
}

impl ActorAddr {
    fn random() -> ActorAddr {
        ActorAddr { addr: random() }
    }
}
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Message {
    pub to: ActorAddr,
    pub cont: MessageContent,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct MessageContent {
    pub atom: Atom,
    pub data: Vec<Argument>,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum Argument {
    ActorAddr(ActorAddr),
    Number(i32),
    String(String),
    Atom(Atom),
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct Atom(pub u32);

pub struct Context<'a> {
    pub msg_sender: &'a mut Sender<Message>,
}

pub trait Actor {
    fn handle_message<'a>(&'a mut self, message: MessageContent, ctx: Context<'a>);
}
