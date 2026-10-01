# tracing-build-script

A [`tracing-subscriber`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/) compatible writer for logging events in build scripts.

## Getting Started
### How to install
Add `tracing-build-script` to your dependencies

```toml
[package]
# ...
build = "build.rs"

[build-dependencies]
# ...
tracing-build-script = "0.2.0"
tracing-subscriber = "0.3.19"
```

### Quickstart
> build.rs
> ```rust
> fn main() {
>    tracing_subscriber::fmt()
>        .with_writer(tracing_build_script::BuildScriptMakeWriter::new())
>        .init();
> 
>    ...
> }
> ```

By default, error and warn events are emitted as `cargo::warning=`, all other levels go to stderr (visible with `cargo build -vv`).
To fail the build on error events instead, use `BuildScriptMakeWriter::new().fail_on_error(true)`.
