use std::time::Instant;

use rand::random;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct ActorAddr {
    pub addr: u32,
}

impl std::fmt::Debug for ActorAddr {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "@{:x}", self.addr)
    }
}

impl ActorAddr {
    pub fn random() -> ActorAddr {
        ActorAddr { addr: random() }
    }
}
#[derive(PartialEq, Debug, Clone)]
pub struct Message {
    pub to: ActorAddr,
    pub cont: MessageContent,
    pub arrive_after: Option<Instant>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct MessageContent {
    pub atom: Atom,
    pub data: Vec<Value>,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Value {
    ActorAddr(ActorAddr),
    Number(u64), // TODO: add another for i32
    Float(f64),
    String(String),
    Atom(Atom),
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct Atom(pub u64);

impl Atom {
    pub fn random() -> Atom {
        Atom(random())
    }
}
