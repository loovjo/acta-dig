mod messages;
mod acting;

mod actors;
use actors::{dyn_actor, io_actor};

const ATOM_START: messages::Atom = messages::Atom(0);

fn main() {
    let mut worker = acting::Worker::new();

    let io_addr = worker.add_actor(Box::new(io_actor::IOActor));

    let dyn_actor = construct_dyn();

    let dyn_addr = worker.add_actor(Box::new(dyn_actor));

    eprintln!("Registered io actor at {:?}", io_addr);
    eprintln!("Registered dyn actor at {:?}", dyn_addr);

    worker.msg_queue.make_ctx().push_msg(messages::Message {
        to: dyn_addr,
        cont: messages::MessageContent {
            atom: ATOM_START,
            data: vec![
                messages::Argument::ActorAddr(io_addr),
                messages::Argument::Atom(io_actor::IOActor::PRINT_MSG),
            ],
        },
        arrive_after: None,
    });

    while worker.step_once() {}
}

fn construct_dyn() -> dyn_actor::DynActor {
    let mut actor_fns = std::collections::HashMap::new();

    let fn_1 = move |msg_content: messages::MessageContent, mut context: acting::Context| {
        match &*msg_content.data {
            &[messages::Argument::ActorAddr(io_addr), messages::Argument::Atom(ref atom)] => {
                context.push_msg(messages::Message {
                    to: io_addr,
                    cont: messages::MessageContent {
                        atom: *atom,
                        data: vec![messages::Argument::String("Hello, world".to_string())],
                    },
                    arrive_after: Some(
                        std::time::Instant::now() + std::time::Duration::from_secs(1),
                    ),
                });
            }
            _ => {}
        }
    };

    let fn_1_boxed: Box<dyn Fn(messages::MessageContent, acting::Context)> = Box::new(fn_1);

    actor_fns.insert(ATOM_START, ("START".to_string(), fn_1_boxed));

    dyn_actor::DynActor {
        actor_name: "Starter Boi".to_string(),
        actor_fns,
    }
}
