# AI agent instructions

Whenever an AI agent autonomously formats Rust code in this crate, it must use `cargo +nightly fmt`. This is required by
the unstable settings in `rustfmt.toml`, including `skip_macro_invocations`. VS Code already uses nightly rustfmt
through the override in `.vscode/settings.json`.

The line-wrap limit is defined by `max_width` in `rustfmt.toml` and is currently 120 characters. Generated text must not
be hard-wrapped until the configured limit would be exceeded; wrap at a word boundary to keep lines within that limit.

Write tests in Arrange, Act, Assert (AAA) format, marking each section with the following lowercase comments:

```rust
// arrange

// act

// assert
```
