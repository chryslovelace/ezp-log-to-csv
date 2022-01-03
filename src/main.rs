use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{stdout, BufRead, BufReader},
    path::PathBuf,
};
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    #[structopt(parse(from_os_str))]
    file: PathBuf,
    #[structopt(short, long = "include-headers", help = "Include CSV headers")]
    headers: bool,
}

fn main() -> Result<()> {
    let options = Options::from_args();
    let file = File::open(&options.file).with_context(|| format!("failed to open {:?}", options.file))?;
    let mut reader = BufReader::new(file);
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
    while reader.read_line(&mut buf).context("failed to read a line from the input file")? != 0 {
        writer
            .write_record(buf.trim().split(' ').map(|field| field.trim_matches(&['"', '[', ']'][..])))
            .context("failed to write a csv record")?;
        buf.clear();
    }
    Ok(())
}
