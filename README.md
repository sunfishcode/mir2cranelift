# mir2cretonne
Rust MIR to Cretonne IL translator

**This is an early experiment and doesn't do anything useful yet.**

## Hacking notes

```sh
rustup default nightly

cargo run -q -- rust-examples/nocore-hello-world.rs

RUST_LOG=mir2cretonne cargo run -q -- rust-examples/nocore-hello-world.rs

rustc -Z unstable-options --unpretty=mir rust-examples/nocore-hello-world.rs
```

## Resources

* [mir2wasm](https://github.com/brson/mir2wasm/), which this code is derived from, though it is heavily modified
* [MIR docs](https://github.com/rust-lang/rfcs/blob/master/text/1211-mir.md)
* [rustc docs](https://manishearth.github.io/rust-internals-docs/rustc/index.html)
* [rustc_trans::mir](https://github.com/rust-lang/rust/tree/master/src/librustc_trans/mir)
* [miri](https://github.com/solson/miri) is a MIR interpreter
