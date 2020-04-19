use std::fs::File;
use std::io;
use std::io::{Write, BufReader, BufRead};

use tempfile::NamedTempFile;
use flate2::read::GzDecoder;
use anyhow::Result;
use regex::Regex;

struct Blocklist<'a> {
    id: &'a str,
    name: &'a str,
}

impl<'a> Blocklist<'a> {
    fn uri(&self) -> String {
        format!("https://list.iblocklist.com/?list={}&fileformat=p2p&archiveformat=gz", self.id)
    }

    fn download(&self) -> Result<File> {
        println!("Downloading list: {}", self.name);
        let body = reqwest::blocking::get(&self.uri())?.bytes()?;
        let mut dest = NamedTempFile::new()?;
        dest.write_all(&body)?;
        let reader = dest.reopen()?;
        Ok(reader)
    }
}

fn decompress(f: Result<File>) -> Result<File> {
    let mut gz = GzDecoder::new(f?);
    let mut out = NamedTempFile::new()?;
    io::copy(&mut gz, &mut out)?;
    let reader = out.reopen()?;
    Ok(reader)
}

fn valid_lines(f: Result<File>) -> Result<Vec<Result<String, std::io::Error>>> {
    let empty_line_or_comment: Regex = Regex::new(r"(^$|^#.*$)")?;
    let r = BufReader::new(f?);
    let valid_lines = r.lines()
        .filter(|l| !empty_line_or_comment.is_match(l.as_ref().unwrap()))
        .collect();
    Ok(valid_lines)
}

fn main() {
    let blocklists = vec![
        Blocklist { name: "level1", id: "ydxerpxkpcfqjaybcssw" },
        Blocklist { name: "level2", id: "gyisgnzbhppbvsphucsw" },
        Blocklist { name: "level3", id: "uwnukjqktoggdknzrhgh" },
    ];

    let mut combined_list = File::create("blocklist.txt").unwrap();
    blocklists.iter()
        .map(|b| b.download())
        .map(|f| decompress(f))
        .flat_map(|f| valid_lines(f).unwrap())
        .for_each(|l| {
            writeln!(combined_list, "{}", l.as_ref().unwrap())
                .unwrap();
        })
}
