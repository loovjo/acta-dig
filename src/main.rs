mod acting;
mod dyn_actor;
mod io_actor;

fn main() {
    let mut worker = acting::Worker::new();

    let io_addr = worker.add_actor(Box::new(io_actor::IOActor));

    let dyn_actor = construct_dyn(io_addr);

    let dyn_addr = worker.add_actor(Box::new(dyn_actor));

    worker
        .msg_send
        .send(acting::Message {
            to: dyn_addr,
            cont: acting::MessageContent {
                atom: acting::Atom::DynActorStart,
                data: vec![acting::Argument::Atom(acting::Atom::PrintHello)],
            },
        })
        .expect("Could not send");

    while worker.step_once() {}

    println!("IOActor at {:?}", io_addr);
}

fn construct_dyn(io_addr: acting::ActorAddr) -> dyn_actor::DynActor {
    let mut actor_fns = std::collections::HashMap::new();

    let fn_1 =
        move |msg_content: acting::MessageContent, ctx: acting::Context| match &*msg_content.data {
            &[acting::Argument::ActorAddr(io_actor), acting::Argument::Atom(ref atom)] => {
                ctx.msg_sender
                    .send(acting::Message {
                        to: io_addr,
                        cont: acting::MessageContent {
                            atom: acting::Atom::PrintMsg,
                            data: vec![acting::Argument::String("Hello, world".to_string())],
                        },
                    })
                    .expect("Could not send");
            }
            _ => {}
        };

    let fn_1_boxed: Box<dyn Fn(acting::MessageContent, acting::Context)> = Box::new(fn_1);

    actor_fns.insert(acting::Atom::DynActorStart, fn_1_boxed);

    dyn_actor::DynActor { actor_fns }
}
