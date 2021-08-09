# dispatchr

This binds libdispatch, and is an alternative to the [dispatch](https://crates.io/crates/dispatch/0.2.0) crate.

In design, this crate is similar to my [objc](https://github.com/drewcrawford/objr) crate.  That is,
* It does "what clang would do".  Blocks are memory-compatible
