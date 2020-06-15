use lvchat_core::*;

fn foo() {
    let foo = Message {
        source: Source::Client,
        kind: MessageKind::Join,
        time: std::time::Instant::now(),
        data: b"avonarret",
    };


    println!("{:#?}", foo);
    println!("{:?}", bytemuck::bytes_of(&foo));
}

fn main() {
    foo();
}