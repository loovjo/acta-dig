use crate::acting;

pub const ATOM_HELLO: &str = "hello";

pub struct IOActor;

impl acting::Actor for IOActor {
    fn handle_message(&self, msg: acting::MessageContent, _ctx: acting::Context) {
        match (&*msg.atom.name, &*msg.data) {
            (ATOM_HELLO, &[]) => {
                println!("Hello");
            }
            _ => {}
        }
    }
}
