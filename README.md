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
tracing-build-script = "0.1.0"
tracing-subscriber = "0.3.19"
```

### Quickstart
> build.rs
> ```rust
> fn main() {
>    tracing_subscriber::fmt()
>        .with_writer(tracing_build_script::BuildScriptMakeWriter)
>        .init();
> 
>    ...
> }
> ```
