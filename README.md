# demon-core

Unsafe, undocumented, radioactive types for fun and profit.
Incomplete and likely full of lurking bugs and evils, **use another library for production**. This is for fun and learning.

Provides small vector optimized versions of `String` and `Vec`, named `Str` and `Vector` for your inconvenience.
Both types offer 23 bytes (on 64-bit architectures, 11 bytes for 32-bit) of stack storage before spilling onto the heap.

`#![no-std]` and zero dependencies.

## Usage

This crate is currently only available from it's git repository. It's a good idea to pin it to a specific commit.

Add the following to your project's `Cargo.toml` to use it.

```toml
[dependencies]
demon-core = { git = "https://github.com/divinejadie/demon-core" }
```

## Safety

There aren't any safety comments, however all tests are routinely validated with miri on 64 and 32 bit targets.

## Contributing

Keep in mind this is only a personal project, but you are welcome to contribute if you like.

## Benchmarks

Benchmarks are managed with [criterion](https://github.com/bheisler/criterion.rs).
Run `cargo bench` to see benchmarks. Results are roughly the same as the `std` counterparts.

## License

`MPL-2.0`, see [LICENSE](https://github.com/divinejadie/demon-core/blob/main/LICENSE.md) for details.
