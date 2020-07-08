use std::collections::HashMap;

use crate::acting;

pub struct DynActor {
    pub actor_fns:
        HashMap<acting::Atom, Box<dyn Fn(acting::MessageContent, acting::Context)>>,
}

impl acting::Actor for DynActor {
    fn handle_message(&mut self, msg: acting::MessageContent, ctx: acting::Context) {
        let actor_fn = if let Some(af) = self.actor_fns.get(&msg.atom) {
            af
        } else {
            return;
        };

        actor_fn(msg, ctx);
    }
}
