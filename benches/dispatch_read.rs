use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dispatchr::io::dispatch_fd_t;
use dispatchr::data::{Unmanaged,Contiguous};
use dispatchr::dispatch_read_block;
use dispatchr::queue;
use dispatchr::qos::QoS;
use std::os::raw::c_int;

fn dispatch_read_inline() {

}


fn criterion_benchmark(c: &mut Criterion) {
    let path = std::path::Path::new("src/io.rs");
    let file = std::fs::File::open(path).unwrap();
    let fd = dispatch_fd_t::new(file);
    const inline_len: usize = 64;
    type inline_ty = [u8; inline_len];
    let context = [2; inline_len];
    use std::sync::mpsc::{Sender,channel};
    struct Context {
        payload: inline_ty,
        sender: Sender<()>
    }

    fn read_fn(context: Context, u: Unmanaged, err: c_int) {
        for item in context.payload {
            black_box(item);
        }
        context.sender.send(()).unwrap();
    }

    fn read_boxed(context:Box<Context>, u: Unmanaged, err: c_int) {
        for item in context.payload {
            black_box(item);
        }
        context.sender.send(()).unwrap();
    }
    let mut group = c.benchmark_group("dispatch_read");

    group.bench_function("dispatch_read_inline", |b| b.iter(|| {
        let (sender,receiver) = channel();

        let context = Context {
            payload: context,
            sender: sender
        };
        let block = dispatch_read_block!(read_fn, context, Context );
        unsafe { dispatchr::io::dispatch_read(fd.clone(), 20, queue::global(QoS::UserInitiated), block.as_raw()) };
        std::mem::forget(block);
        receiver.recv().unwrap();
    }));
    group.bench_function("dispatch_read_boxed", |b| b.iter(|| {
        let (sender,receiver) = channel();

        let context = Context {
            payload: context,
            sender: sender
        };
        let block = dispatch_read_block!(read_boxed, Box::new(context), Box<Context>);
        unsafe { dispatchr::io::dispatch_read(fd.clone(), 20, queue::global(QoS::UserInitiated), block.as_raw()) };
        std::mem::forget(block);
        receiver.recv().unwrap();
    }));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);