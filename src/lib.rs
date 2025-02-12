use std::io;
use tracing::{Level, Metadata};
use tracing_subscriber::fmt::MakeWriter;

enum BuildScriptWriterInner {
    Informational(io::Stderr),
    ErrorsAndWarnings { first_write: bool, writer: io::Stdout },
}

/// A writer intended to support the [output capturing of build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script).
/// `BuildScriptWriter` can be used by [`tracing_subscriber::fmt::Subscriber`](tracing_subscriber::fmt::Subscriber) or [`tracing_subscriber::fmt::Layer`](tracing_subscriber::fmt::Layer)
/// to enable capturing output in build scripts.
///
/// # Logging Behaviour
/// Events for Levels Error and Warn are printed to stdout with [`cargo::warning=`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargo-warning) prepended.
/// All other levels are sent to stderr, where they are only visible when running with verbose build output (`cargo build -vv`).
///
/// Note: this writer explicitly does **not** use the [`cargo::error=`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargo-error) instruction
/// because it aborts the build with an error, which is not always desired.
///
/// # Example
/// ```
/// tracing_subscriber::fmt()
///     .with_writer(tracing_build_script::BuildScriptMakeWriter)
///     .init();
/// ```
pub struct BuildScriptWriter(BuildScriptWriterInner);

impl BuildScriptWriter {
    /// Create a writer for informational events.
    /// Events will be written to stderr.
    pub fn informational() -> Self {
        Self(BuildScriptWriterInner::Informational(io::stderr()))
    }

    /// Create a writer for warning and error events.
    /// Events will be written to stdout after having `cargo::warning=` prepended.
    pub fn errors_and_warnings() -> Self {
        Self(BuildScriptWriterInner::ErrorsAndWarnings { first_write: true, writer: io::stdout() })
    }
}

impl io::Write for BuildScriptWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match &mut self.0 {
            BuildScriptWriterInner::Informational(writer) => writer.write(buf),
            BuildScriptWriterInner::ErrorsAndWarnings { first_write, writer } => {
                if *first_write {
                    writer.write(b"cargo::warning=")?;
                    *first_write = false;
                }

                writer.write(buf)
            },
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.0 {
            BuildScriptWriterInner::Informational(writer) => writer.flush(),
            BuildScriptWriterInner::ErrorsAndWarnings { writer, first_write: _ } => writer.flush(),
        }
    }
}

pub struct BuildScriptMakeWriter;

impl<'a> MakeWriter<'a> for BuildScriptMakeWriter {
    type Writer = BuildScriptWriter;

    fn make_writer(&'a self) -> Self::Writer {
        BuildScriptWriter::informational()
    }

    fn make_writer_for(&'a self, meta: &Metadata) -> Self::Writer {
        if meta.level() == &Level::ERROR || meta.level() == &Level::WARN {
            BuildScriptWriter::errors_and_warnings()
        } else {
            BuildScriptWriter::informational()
        }
    }
}
