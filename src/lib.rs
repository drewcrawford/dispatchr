/*!
# dispatchr

My Rust bindings for libdispatch, a.k.a. [GCD](https://en.wikipedia.org/wiki/Grand_Central_Dispatch).  This is an alternative to the [dispatch](https://crates.io/crates/dispatch/0.2.0) crate.

This crate is part of the [objr expanded universe universe](https://github.com/drewcrawford/objr#objr-expanded-universe) which provide low-level, zero-cost Rust abstractions
for Apple platform features that mimic code from first-party compilers.  Distinctive features of this library include:

* Leverages [blocksr](https://github.com/drewcrawford/blocksr) technology for fast, low-overhead, static compile-time optimizations of dispatch calls.
* Exposes a rich set of datatypes for `dispatch_data`, including managed, unmanaged, contiguous, and zero-copy-bridged flavors of data
* Binds `dispatch_read`/write, the defacto API for nonblocking IO on macOS.
    * Notably, the rest of the Rust ecosystem uses some cross-platform API to cover macOS, like `poll` or `kevent`.  These
      lack various features and optimizations of the preferred API.
    * In general, Apple implements the cross-platform APIs with about as much care as the developers using them to port cross-platform apps: *not enough*.
* Binds `QoS`, which is *the* solution for task priority and responsive GUI apps on macOS

# Status
dispatchr covers large but incomplete portions of the libdispatch API.

* global queues, dispatch_sync
* qos
* popular portions of io: `dispatch_read`, `dispatch_write`, `dispatch_io_create_with_path`
* data
* semaphore
* source (timers only)

*/

extern crate core;

pub mod queue;
pub mod qos;
pub mod io;
pub mod data;
pub mod block_impl;
pub mod external_data;
pub mod semaphore;
pub mod time;
pub mod source;

pub use qos::QoS;
