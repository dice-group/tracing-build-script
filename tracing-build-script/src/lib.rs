use std::{io, io::Write};
use tracing::{Level, Metadata};
use tracing_subscriber::fmt::MakeWriter;

enum ErrorAndWarnState {
    /// Initial state, no output has been written yet
    /// cargo::warning= needs to be written next
    Init,
    /// Normal operation, cargo::warning= was already written
    /// and user input did not end with newline
    Normal,
    /// Normal operation, cargo::warning= was already written
    /// but user input ended with a newline or another char that needs escaping.
    /// This either means the log message is done, or there just happens to be a newline at the end of this write
    /// request
    LastCharWasSpecial(u8),
}

fn char_is_special(ch: u8) -> bool {
    ch == b'\n' || ch == b'\r'
}

fn escape_special(ch: u8) -> &'static [u8] {
    match ch {
        b'\n' => b"\\n",
        b'\r' => b"\\r",
        _ => unreachable!(),
    }
}

enum BuildScriptWriterInner {
    Informational(io::Stderr),
    ErrorsAndWarnings {
        state: ErrorAndWarnState,
        /// `cargo::warning=` or `cargo::error=`
        prefix: &'static [u8],
        writer: io::Stdout,
    },
}

/// A writer intended to support the [output capturing of build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script).
/// `BuildScriptWriter` can be used by [`tracing_subscriber::fmt::Subscriber`](tracing_subscriber::fmt::Subscriber) or [`tracing_subscriber::fmt::Layer`](tracing_subscriber::fmt::Layer)
/// to enable capturing output in build scripts.
pub struct BuildScriptWriter(BuildScriptWriterInner);

impl BuildScriptWriter {
    /// Create a writer for informational events.
    /// Events will be written to stderr.
    pub fn informational() -> Self {
        Self(BuildScriptWriterInner::Informational(io::stderr()))
    }

    /// Create a writer for error events.
    /// Events will be written to stdout after having `cargo::error=` prepended,
    /// which makes cargo fail the build once the build script has finished.
    pub fn error() -> Self {
        Self::prefixed(b"cargo::error=")
    }

    /// Create a writer for warning events.
    /// Events will be written to stdout after having `cargo::warning=` prepended.
    pub fn warning() -> Self {
        Self::prefixed(b"cargo::warning=")
    }

    fn prefixed(prefix: &'static [u8]) -> Self {
        Self(BuildScriptWriterInner::ErrorsAndWarnings { state: ErrorAndWarnState::Init, prefix, writer: io::stdout() })
    }
}

impl Drop for BuildScriptWriter {
    fn drop(&mut self) {
        if let BuildScriptWriterInner::ErrorsAndWarnings {
            state: ErrorAndWarnState::LastCharWasSpecial(ch),
            writer,
            ..
        } = &mut self.0
        {
            let _ = writer.write(&[*ch]);
        }
    }
}

impl Write for BuildScriptWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        match &mut self.0 {
            BuildScriptWriterInner::Informational(writer) => writer.flush(),
            BuildScriptWriterInner::ErrorsAndWarnings { writer, .. } => writer.flush(),
        }
    }

    fn write_all(&mut self, mut buf: &[u8]) -> io::Result<()> {
        match &mut self.0 {
            BuildScriptWriterInner::Informational(writer) => writer.write_all(buf),
            BuildScriptWriterInner::ErrorsAndWarnings { state, writer, prefix } => {
                // We will need to issue multiple write calls to the writer (to avoid heap allocation)
                // so we need to lock it to prevent other threads from clobbering our output.
                let mut writer = writer.lock();

                // depending on the current state we may need to prefix the output
                match *state {
                    ErrorAndWarnState::Init => {
                        writer.write_all(prefix)?;
                    },
                    ErrorAndWarnState::LastCharWasSpecial(ch) => {
                        writer.write_all(escape_special(ch))?;
                    },
                    ErrorAndWarnState::Normal => {},
                }

                // If the last char is a newline we need to remember that but cannot immediately
                // write it out. This is because we cannot know yet if its needs to be escaped, there are two cases:
                //
                // 1. this call to write is not actually the last call to write that will happen it just happens to end with a newline
                //    => we need to escape the newline
                //
                // 2. this call to write is actually the last call to write that will happen, and it ends with a newline
                //    => we need to keep the newline as is, because it is the newline terminator of the log message
                //       (tracing automatically appends a newline at the end of each message, like println!)
                //
                // Since we cannot decide which of these cases we are in at the moment, we need to delay writing the last character (if it is a newline) until we know that.
                // We know which case we are in
                //  either when we enter this function the next time (case 1)
                //  or the next time or when we enter the destructor (case 2).
                match buf.last().copied() {
                    Some(ch) if char_is_special(ch) => {
                        buf = &buf[..buf.len() - 1];
                        *state = ErrorAndWarnState::LastCharWasSpecial(ch);
                    },
                    _ => {
                        *state = ErrorAndWarnState::Normal;
                    },
                }

                let mut last_special_char = match buf.iter().position(|ch| char_is_special(*ch)) {
                    Some(pos) => {
                        writer.write_all(&buf[..pos])?;

                        let ret = buf[pos];
                        buf = &buf[pos + 1..];
                        ret
                    },
                    None => {
                        // fast path for messages without any special chars
                        writer.write_all(buf)?;
                        return Ok(());
                    },
                };

                loop {
                    writer.write_all(escape_special(last_special_char))?;

                    match buf.iter().position(|ch| char_is_special(*ch)) {
                        Some(pos) => {
                            writer.write_all(&buf[..pos])?;

                            last_special_char = buf[pos];
                            buf = &buf[pos + 1..];
                        },
                        None => {
                            writer.write_all(buf)?;
                            break;
                        },
                    }
                }

                Ok(())
            },
        }
    }
}

/// [`MakeWriter`](tracing_subscriber::fmt::MakeWriter) implementation for [`BuildScriptWriter`](BuildScriptWriter)
///
/// # Behaviour
/// Events for Levels Error and Warn are printed to stdout with [`cargo::warning=`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargo-warning) prepended.
/// All other levels are sent to stderr, where they are only visible when running with verbose build output (`cargo build -vv`).
///
/// By default the [`cargo::error=`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargo-error) instruction
/// is **not** used because it fails the build, which is not always desired.
/// Opt in with [`fail_on_error`](Self::fail_on_error) to print Error events with `cargo::error=` prepended instead.
///
/// # Example
/// ```
/// tracing_subscriber::fmt()
///     .with_writer(tracing_build_script::BuildScriptMakeWriter::new())
///     .init();
/// ```
#[derive(Default)]
pub struct BuildScriptMakeWriter {
    fail_on_error: bool,
}

impl BuildScriptMakeWriter {
    pub fn new() -> Self {
        Self::default()
    }

    /// If `true`, Error events are printed with `cargo::error=` prepended, which makes cargo
    /// fail the build once the build script has finished. Defaults to `false`.
    pub fn fail_on_error(mut self, fail_on_error: bool) -> Self {
        self.fail_on_error = fail_on_error;
        self
    }

    fn writer_for(&self, level: Level) -> BuildScriptWriter {
        if level == Level::ERROR && self.fail_on_error {
            BuildScriptWriter::error()
        } else if level <= Level::WARN {
            BuildScriptWriter::warning()
        } else {
            BuildScriptWriter::informational()
        }
    }
}

impl<'a> MakeWriter<'a> for BuildScriptMakeWriter {
    type Writer = BuildScriptWriter;

    fn make_writer(&'a self) -> Self::Writer {
        BuildScriptWriter::informational()
    }

    fn make_writer_for(&'a self, meta: &Metadata) -> Self::Writer {
        self.writer_for(*meta.level())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prefix(writer: BuildScriptWriter) -> Option<&'static [u8]> {
        match &writer.0 {
            BuildScriptWriterInner::ErrorsAndWarnings { prefix, .. } => Some(prefix),
            BuildScriptWriterInner::Informational(_) => None,
        }
    }

    #[test]
    fn selects_prefix_by_level() {
        let default = BuildScriptMakeWriter::new();
        assert_eq!(prefix(default.writer_for(Level::ERROR)), Some(&b"cargo::warning="[..]));
        assert_eq!(prefix(default.writer_for(Level::WARN)), Some(&b"cargo::warning="[..]));
        assert_eq!(prefix(default.writer_for(Level::INFO)), None);

        let failing = BuildScriptMakeWriter::new().fail_on_error(true);
        assert_eq!(prefix(failing.writer_for(Level::ERROR)), Some(&b"cargo::error="[..]));
        assert_eq!(prefix(failing.writer_for(Level::WARN)), Some(&b"cargo::warning="[..]));
        assert_eq!(prefix(default.writer_for(Level::INFO)), None);
    }
}
