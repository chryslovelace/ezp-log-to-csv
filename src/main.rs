use anyhow::{Context, Result};
use std::{
    borrow::Cow,
    fs::File,
    io::{stdin, stdout, BufRead, BufReader, Read, Stdin},
    path::{Path, PathBuf},
};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    #[structopt(parse(from_os_str))]
    file: Option<PathBuf>,
    #[structopt(short, long = "include-headers", help = "Include CSV headers at the start of the output")]
    headers: bool,
}

struct Input {
    reader: InputReader,
    name: Cow<'static, str>,
}

impl Input {
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
        Ok(Self {
            reader: InputReader::File(BufReader::new(file)),
            name: path.to_string_lossy().into_owned().into(),
        })
    }

    fn from_stdin() -> Self {
        Self {
            reader: InputReader::Stdin(BufReader::new(stdin())),
            name: "<stdin>".into(),
        }
    }
}

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
    let mut line = 0u64;
    while input
        .reader
        .read_line(&mut buf)
        .with_context(|| format!("failed to read a line from the input after {} lines", line))?
        != 0
    {
        line += 1;
        writer
            .write_record(buf.trim().split(' ').map(|field| field.trim_matches(&['"', '[', ']'][..])))
            .with_context(|| format!("failed to write a csv record for line {} of {}", line, input.name))?;
        buf.clear();
    }
    Ok(())
}
