#![cfg(test)]

use std::path::Path;

#[test]
fn test_warn_and_error_output() {
    let out_dir = Path::new(env!("OUT_DIR"));
    let warn_output_file = out_dir.join("../output");
    let warn_output = std::fs::read_to_string(warn_output_file).unwrap();

    assert_eq!(
        warn_output,
        "cargo::warning= WARN build_script_build: simple message\n\
        cargo::warning=ERROR build_script_build: simple message\n\
        cargo::warning= WARN build_script_build: with\\nnew\\nline\n\
        cargo::warning=ERROR build_script_build: with\\nnew\\nline\n\
        cargo::warning= WARN build_script_build: with\\nnew\\nlineend\\n\n\
        cargo::warning=ERROR build_script_build: with\\nnew\\nlineend\\n\n\
        cargo::warning= WARN build_script_build: \\nwith\\nnew\\nlinestart\n\
        cargo::warning=ERROR build_script_build: \\nwith\\nnew\\nlinestart\n\
        cargo::warning= WARN build_script_build: two\\n\\nnewlines\n\
        cargo::warning=ERROR build_script_build: two\\n\\nnewlines\n\
        cargo::warning= WARN build_script_build: other\\rspecial\tchar\0a\tb\"c\\\n\
        cargo::warning=ERROR build_script_build: other\\rspecial\tchar\0a\tb\"c\\\n"
    );
}

#[test]
fn test_informational_output() {
    let out_dir = Path::new(env!("OUT_DIR"));
    let info_output_file = out_dir.join("../stderr");
    let info_output = std::fs::read_to_string(info_output_file).unwrap();

    assert_eq!(
        info_output,
        "TRACE build_script_build: simple message\n\
        DEBUG build_script_build: simple message\n \
        INFO build_script_build: simple message\n\
        TRACE build_script_build: with\nnew\nline\n\
        DEBUG build_script_build: with\nnew\nline\n \
        INFO build_script_build: with\nnew\nline\n\
        TRACE build_script_build: with\nnew\nlineend\n\n\
        DEBUG build_script_build: with\nnew\nlineend\n\n \
        INFO build_script_build: with\nnew\nlineend\n\n\
        TRACE build_script_build: \nwith\nnew\nlinestart\n\
        DEBUG build_script_build: \nwith\nnew\nlinestart\n \
        INFO build_script_build: \nwith\nnew\nlinestart\n\
        TRACE build_script_build: two\n\nnewlines\n\
        DEBUG build_script_build: two\n\nnewlines\n \
        INFO build_script_build: two\n\nnewlines\n\
        TRACE build_script_build: other\rspecial\tchar\0a\tb\"c\\\n\
        DEBUG build_script_build: other\rspecial\tchar\0a\tb\"c\\\n \
        INFO build_script_build: other\rspecial\tchar\0a\tb\"c\\\n"
    );
}
