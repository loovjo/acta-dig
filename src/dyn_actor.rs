use std::collections::HashMap;

use crate::acting;

pub struct DynActor {
    actor_fns:
        HashMap<acting::Atom, Box<dyn for<'a> Fn(acting::MessageContent, acting::Context<'a>)>>,
}

impl acting::Actor for DynActor {
    fn handle_message(&self, msg: acting::MessageContent, ctx: acting::Context) {}
}
