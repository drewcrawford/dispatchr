# dispatchr

This binds libdispatch, and is an alternative to the [dispatch](https://crates.io/crates/dispatch/0.2.0) crate.

In design, this crate is similar to my [objr](https://github.com/drewcrawford/objr) crate.  That is,
* It does "what clang would do".  Blocks are memory-compatible with Clang/objc
* It exploits a variety of low-level objc tricks
* It assumes modern apple platforms
* It is small and has few dependencies

In practice, dispatchr is more immature, currently supporting only `dispatch_read` and a few queue APIs.
