use std::collections::HashMap;
use std::collections::VecDeque;
use std::time::Instant;

use crate::messages::{ActorAddr, Message, MessageContent, Atom};

pub struct MessageQueue {
    inner: VecDeque<Message>,
}

// TODO: Use the delegate crate
impl MessageQueue {
    pub fn new() -> MessageQueue {
        MessageQueue {
            inner: VecDeque::new(),
        }
    }

    pub fn push_msg(&mut self, msg: Message) {
        self.inner.push_back(msg);
    }

    pub fn pop_msg(&mut self) -> Option<Message> {
        self.inner.pop_back()
    }

    pub fn make_ctx<'a>(&'a mut self) -> Context<'a> {
        Context { msg_queue: self }
    }
}

pub struct Worker {
    actors: HashMap<ActorAddr, Box<dyn Actor>>,

    pub msg_queue: MessageQueue,
}

impl Worker {
    pub fn new() -> Worker {
        Worker {
            actors: HashMap::new(),
            msg_queue: MessageQueue::new(),
        }
    }

    pub fn add_actor(&mut self, actor: Box<dyn Actor>) -> ActorAddr {
        let addr = ActorAddr::random();
        self.actors.insert(addr, actor);
        addr
    }

    pub fn step_once(&mut self) -> bool {
        match self.msg_queue.pop_msg() {
            Some(Message {
                arrive_after: Some(arrive_after),
                to,
                cont,
            }) if Instant::now() < arrive_after => {
                self.msg_queue.push_msg(Message {
                    arrive_after: Some(arrive_after),
                    to,
                    cont,
                });
                true
            }
            Some(Message { to, cont, .. }) => {
                if let Some(act) = self.actors.get_mut(&to) {
                    {
                        // Debug information
                        let rec_debug_info = act.debug_info();
                        let actor_name = rec_debug_info.get_actor_name();
                        let atom_name = rec_debug_info.get_atom_name(cont.atom);
                        eprintln!(
                            "Sending {}({:?}) to {:?}/{:?}",
                            atom_name
                                .map(|x| format!("{:?}", x))
                                .unwrap_or("UNK".to_string()),
                            cont.data,
                            actor_name,
                            to,
                        );
                    }
                    let ctx = self.msg_queue.make_ctx();
                    act.handle_message(cont, ctx);
                    true
                } else {
                    eprintln!(
                        "Tried sending message to nonexistant/dead actor with address {:?}. Message content = {:?}",
                        to, cont
                    );
                    true
                }
            }
            None => false,
        }
    }
}


// TODO: Properly abstract this to handle multiple workers
pub struct Context<'a> {
    msg_queue: &'a mut MessageQueue,
}

impl<'a> Context<'a> {
    pub fn push_msg(&mut self, msg: Message) {
        self.msg_queue.push_msg(msg);
    }
}

pub trait Actor {
    fn handle_message<'a>(&'a mut self, message: MessageContent, ctx: Context<'a>);
    fn debug_info<'a>(&'a self) -> Box<dyn DebugInfo + 'a>;
}

// TODO: We could probably use a more concise and less borrowed interface. &str should be fine instead of using Strings
pub trait DebugInfo {
    fn get_actor_name(&self) -> String;
    fn get_atom_name(&self, atom: Atom) -> Option<String>;
}
