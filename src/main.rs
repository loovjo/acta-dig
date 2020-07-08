mod acting;

mod actors;
use actors::{dyn_actor, io_actor};

const ATOM_START: acting::Atom = acting::Atom(0);

fn main() {
    let mut worker = acting::Worker::new();

    let io_addr = worker.add_actor(Box::new(io_actor::IOActor));

    let dyn_actor = construct_dyn();

    let dyn_addr = worker.add_actor(Box::new(dyn_actor));

    eprintln!("Registered io actor at {:?}", io_addr);
    eprintln!("Registered dyn actor at {:?}", dyn_addr);

    worker
        .msg_queue
        .make_ctx()
        .push_msg(acting::Message {
            to: dyn_addr,
            cont: acting::MessageContent {
                atom: ATOM_START,
                data: vec![
                    acting::Argument::ActorAddr(io_addr),
                    acting::Argument::Atom(io_actor::IOActor::PRINT_MSG),
                ],
            },
        });

    while worker.step_once() {}
}

fn construct_dyn() -> dyn_actor::DynActor {
    let mut actor_fns = std::collections::HashMap::new();

    let fn_1 =
        move |msg_content: acting::MessageContent, mut ctx: acting::Context| match &*msg_content.data {
            &[acting::Argument::ActorAddr(io_addr), acting::Argument::Atom(ref atom)] => {
                ctx.push_msg(acting::Message {
                    to: io_addr,
                    cont: acting::MessageContent {
                        atom: *atom,
                        data: vec![acting::Argument::String("Hello, world".to_string())],
                    },
                });
            }
            _ => {}
        };

    let fn_1_boxed: Box<dyn Fn(acting::MessageContent, acting::Context)> = Box::new(fn_1);

    actor_fns.insert(ATOM_START, fn_1_boxed);

    dyn_actor::DynActor { actor_fns }
}
