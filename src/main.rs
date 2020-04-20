use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::io::{Write, BufReader, BufRead};

use tempfile::NamedTempFile;
use flate2::read::GzDecoder;
use anyhow::Result;
use regex::Regex;
use structopt::StructOpt;


#[derive(Debug, StructOpt)]
#[structopt(name = "iblocklist_generator", about = "Fetch and combine lists from iblocklist.com")]
struct Opt {
    /// Combined list destination; writes to stdout if not specified.
    #[structopt(short, long, parse(from_os_str))]
    output_file: Option<PathBuf>,
}

struct Blocklist<'a> {
    id: &'a str,
    name: &'a str,
}

impl<'a> Blocklist<'a> {
    fn uri(&self) -> String {
        format!("https://list.iblocklist.com/?list={}&fileformat=p2p&archiveformat=gz", self.id)
    }

    fn download(&self) -> Result<File> {
        eprintln!("Downloading list: {}", self.name);
        let body = reqwest::blocking::get(&self.uri())?.bytes()?;
        let mut dest = NamedTempFile::new()?;
        dest.write_all(&body)?;
        let reader = dest.reopen()?;
        Ok(reader)
    }
}

fn decompress(f: File) -> Result<File> {
    let mut gz = GzDecoder::new(f);
    let mut out = NamedTempFile::new()?;
    io::copy(&mut gz, &mut out)?;
    let reader = out.reopen()?;
    Ok(reader)
}

fn valid_entries(f: File) -> Result<Vec<Result<String, std::io::Error>>> {
    let empty_line_or_comment: Regex = Regex::new(r"(^$|^#.*$)")?;
    let r = BufReader::new(f);
    let valid_entries = r.lines()
        .filter(|l| !empty_line_or_comment.is_match(l.as_ref().unwrap()))
        .collect();
    Ok(valid_entries)
}

fn get_output(output_opt: Option<PathBuf>) -> Box<dyn Write> {
    output_opt
        .map(|o| File::create(o.as_path()).expect("Invalid output path"))
        .map(|f| Box::new(f) as Box<dyn Write>)
        .unwrap_or(Box::new(std::io::stdout()) as Box<dyn Write>)
}

fn main() {
    let opt = Opt::from_args();
    let mut output = get_output(opt.output_file);

    let blocklists = vec![
        Blocklist { name: "level1", id: "ydxerpxkpcfqjaybcssw" },
        Blocklist { name: "level2", id: "gyisgnzbhppbvsphucsw" },
        Blocklist { name: "level3", id: "uwnukjqktoggdknzrhgh" },
    ];

    blocklists.iter()
        .map(|b| b.download().expect("Problem downloading blocklist"))
        .map(|f| decompress(f).expect("Problem decompressing blocklist"))
        .flat_map(|f| valid_entries(f).expect("Problem filtering invalid entries"))
        .for_each(|l| {
            writeln!(output, "{}", l.as_ref().unwrap())
                .expect("Problem writing entry to output file");
        })
}
