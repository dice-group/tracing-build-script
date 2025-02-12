use std::io;
use tracing::{Level, Metadata};
use tracing_subscriber::fmt::MakeWriter;

enum BuildScriptWriterInner {
    Informational(io::Stderr),
    ErrorsAndWarnings { first_write: bool, writer: io::Stdout },
}

pub struct BuildScriptWriter(BuildScriptWriterInner);

impl BuildScriptWriter {
    pub fn informational() -> Self {
        Self(BuildScriptWriterInner::Informational(io::stderr()))
    }

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
