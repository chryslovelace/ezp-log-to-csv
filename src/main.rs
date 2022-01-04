use anyhow::{Context, Result};
use std::{
    borrow::Cow,
    fs::File,
    io::{stdin, stdout, BufRead, BufReader, Read, Stdin},
    path::{Path, PathBuf},
};
use structopt::StructOpt;

fn main() -> Result<()> {
    let options = Options::from_args();
    let mut input = match options.file {
        Some(path) => Input::from_file(path)?,
        None => Input::from_stdin(),
    };
    let mut writer = csv::Writer::from_writer(stdout());
    if options.headers {
        writer
            .write_record([
                "IP Address",
                "Username",
                "Session ID",
                "Timestamp",
                "Timezone",
                "HTTP Method",
                "Request URL",
                "HTTP Version",
                "HTTP Status Code",
                "Bytes Transferred",
            ])
            .context("failed to write csv headers")?;
    }
    let mut buf = String::new();
    while input.read_line(&mut buf)? {
        writer
            .write_record(buf.trim().split(' ').map(|field| field.trim_matches(&['"', '[', ']'][..])))
            .with_context(|| format!("failed to write a csv record for line {} of {}", input.line, input.name))?;
        buf.clear();
    }
    Ok(())
}

#[derive(StructOpt)]
struct Options {
    #[structopt(parse(from_os_str))]
    file: Option<PathBuf>,
    #[structopt(short, long = "include-headers", help = "Include CSV headers at the start of the output")]
    headers: bool,
}

/// Struct representing an input source, which can be a file or stdin.
struct Input {
    /// Holds the handle to the input.
    reader: InputReader,
    /// The file name, or the string "<stdin>".
    name: Cow<'static, str>,
    /// Tracks how many lines have been read.
    line: u64,
}

impl Input {
    /// Constructs an Input that reads from a file.
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let name = path.display().to_string();
        let file = File::open(path).with_context(|| format!("failed to open {}", name))?;
        Ok(Self {
            reader: InputReader::File(BufReader::new(file)),
            name: name.into(),
            line: 0,
        })
    }

    /// Constructs an Input that reads from stdin.
    fn from_stdin() -> Self {
        Self {
            reader: InputReader::Stdin(BufReader::new(stdin())),
            name: "<stdin>".into(),
            line: 0,
        }
    }

    /// Read a line from the input and appends it to the provided buffer, including the terminating newline character.
    /// Returns true if there are lines remaining, false for EOF.
    fn read_line(&mut self, buf: &mut String) -> Result<bool> {
        let bytes_read = self
            .reader
            .read_line(buf)
            .with_context(|| format!("failed to read a line from the input after {} lines", self.line))?;
        self.line += 1;
        Ok(bytes_read != 0)
    }
}

/// Wraps the underlying source in a BufReader and dispatches to its Read and BufRead implementations.
enum InputReader {
    Stdin(BufReader<Stdin>),
    File(BufReader<File>),
}

impl Read for InputReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            InputReader::Stdin(stdin) => stdin.read(buf),
            InputReader::File(file) => file.read(buf),
        }
    }
}

impl BufRead for InputReader {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        match self {
            InputReader::Stdin(stdin) => stdin.fill_buf(),
            InputReader::File(file) => file.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match self {
            InputReader::Stdin(stdin) => stdin.consume(amt),
            InputReader::File(file) => file.consume(amt),
        }
    }
}
