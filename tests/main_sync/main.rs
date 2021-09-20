use dispatchr::queue::DispatchSyncBlock;
extern "C" {
    fn dispatch_main();
}
fn child_task() {
    use core::pin::Pin;
    let queue = dispatchr::queue::main();
    use std::mem::MaybeUninit;
    let mut block_value = MaybeUninit::uninit();
    let block_value = unsafe{ Pin::new_unchecked(&mut block_value) };
    let mut run = false;
    let block_value = unsafe{DispatchSyncBlock::new(block_value, || {
        println!("hello from child_task");
        run = true;
    }) };
    queue.sync(&block_value);
    assert!(run);
    std::process::exit(0);
}
fn main() {
    std::thread::spawn(|| {
         child_task();
    });
    unsafe {
        dispatch_main()
    }
}
