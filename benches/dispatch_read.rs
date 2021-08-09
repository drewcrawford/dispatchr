use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dispatchr::io::dispatch_fd_t;
use dispatchr::queue;
use dispatchr::qos::QoS;

fn criterion_benchmark(c: &mut Criterion) {
    let path = std::path::Path::new("src/io.rs");
    let file = std::fs::File::open(path).unwrap();
    let fd = dispatch_fd_t::new(file);
    use std::sync::mpsc::channel;


    c.bench_function("dispatch_read_closure", |b| b.iter(|| {
        let (sender,receiver) = channel();
        dispatchr::io::read(fd.clone(), 20, queue::global(QoS::UserInitiated), |a,_b| {
            black_box(a);
            sender.send(()).unwrap();
        });
        receiver.recv().unwrap();
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);