use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt::{self, Display};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

enum Error {
    ReadFile(io::Error),
    ParseFile {
        error: syn::Error,
        filepath: PathBuf,
        source_code: String,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match self {
            ReadFile(error) => write!(f, "Unable to read file: {}", error),
            ParseFile {
                error,
                filepath,
                source_code,
            } => render_location(f, error, filepath, source_code),
        }
    }
}

// Render a rustc-style error message, including colors.
//
//     error: Syn unable to parse file
//       --> main.rs:40:17
//        |
//     40 |     fn fmt(&self formatter: &mut fmt::Formatter) -> fmt::Result {
//        |                  ^^^^^^^^^ expected `,`
//
fn render_location(
    formatter: &mut fmt::Formatter,
    err: &syn::Error,
    filepath: &Path,
    code: &str,
) -> fmt::Result {
    let start = err.span().start();
    let mut end = err.span().end();

    if start.line == end.line && start.column == end.column {
        return render_fallback(formatter, err);
    }

    let code_line = match code.lines().nth(start.line - 1) {
        Some(line) => line,
        None => return render_fallback(formatter, err),
    };

    if end.line > start.line {
        end.line = start.line;
        end.column = code_line.len();
    }

    let filename = filepath
        .file_name()
        .map(OsStr::to_string_lossy)
        .unwrap_or(Cow::Borrowed("main.rs"));

    write!(
        formatter,
        "\n\
         {error}{header}\n\
         {indent}{arrow} {filename}:{linenum}:{colnum}\n\
         {indent} {pipe}\n\
         {label} {pipe} {code}\n\
         {indent} {pipe} {offset}{underline} {message}\n\
         ",
        error = "error",
        header = ": Syn unable to parse file",
        indent = " ".repeat(start.line.to_string().len()),
        arrow = "-->",
        filename = filename,
        linenum = start.line,
        colnum = start.column,
        pipe = "|",
        label = start.line.to_string(),
        code = code_line.trim_end(),
        offset = " ".repeat(start.column),
        underline = "^".repeat(end.column - start.column),
        message = err.to_string(),
    )
}

fn render_fallback(formatter: &mut fmt::Formatter, err: &syn::Error) -> fmt::Result {
    write!(formatter, "Unable to parse file: {}", err)
}

fn try_main() -> Result<(), Error> {
    // TODO: Change this by a configurable path
    let filepath = PathBuf::from("/root/test.rs");

    let code = fs::read_to_string(&filepath).map_err(Error::ReadFile)?;
    let syntax = syn::parse_file(&code).map_err({
        |error| Error::ParseFile {
            error,
            filepath,
            source_code: code,
        }
    })?;
    println!("{:#?}", syntax);

    Ok(())
}

#[no_mangle]
pub extern "C" fn dump_ast() {
    if let Err(error) = try_main() {
        let _ = writeln!(io::stderr(), "{}", error);
    }
}
