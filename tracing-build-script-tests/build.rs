use tracing::metadata::LevelFilter;

fn write_message(msg: &str) {
    tracing::trace!("{msg}");
    tracing::debug!("{msg}");
    tracing::info!("{msg}");
    tracing::warn!("{msg}");
    tracing::error!("{msg}");
}

fn main() {
    tracing_subscriber::fmt()
        .with_writer(tracing_build_script::BuildScriptMakeWriter)
        .with_ansi(false)
        .without_time()
        .with_max_level(LevelFilter::TRACE)
        .init();

    write_message("simple message");
    write_message("with\nnew\nline");
    write_message("with\nnew\nlineend\n");
    write_message("\nwith\nnew\nlinestart");
    write_message("two\n\nnewlines");
    write_message("other\rspecial\tchar\0a\tb\"c\\");
    write_message("two\nnewlines\natend\n\n");
}
