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

fn blocklist_uri(id: &str) -> String {
    format!("https://list.iblocklist.com/?list={}&fileformat=p2p&archiveformat=gz", id)
}

fn download_list(blocklist: &Blocklist) -> Result<File> {
    println!("Downloading list: {}", blocklist.name);
    let resource = blocklist_uri(blocklist.id);
    let body = reqwest::blocking::get(&resource)?.bytes()?;
    let mut dest = NamedTempFile::new()?;
    dest.write_all(&body)?;
    let reader = dest.reopen()?;
    Ok(reader)
}

fn decompress_list(f: File) -> Result<File> {
    let mut gz = GzDecoder::new(f);
    let mut list_out = NamedTempFile::new()?;
    io::copy(&mut gz, &mut list_out).unwrap();
    let reader = list_out.reopen()?;
    Ok(reader)
}

fn valid_lines(f: File) -> Vec<String> {
    let empty_line_or_comment: Regex = Regex::new(r"(^$|^#.*$)").unwrap();
    let r = BufReader::new(f);
    let valid_lines = r.lines()
        .map(|l| l.unwrap())
        .filter(|line| !empty_line_or_comment.is_match(line))
        .collect();
    valid_lines
}

fn main() -> Result<()> {
    let blocklists = vec![
        Blocklist { name: "level1", id: "ydxerpxkpcfqjaybcssw" },
        Blocklist { name: "level2", id: "gyisgnzbhppbvsphucsw" },
        Blocklist { name: "level3", id: "uwnukjqktoggdknzrhgh" },
    ];

    let local_blocklists: Vec<Result<File>> = blocklists.iter()
        .map(|blocklist| { download_list(&blocklist).unwrap() })
        .map(|f| decompress_list(f))
        .collect();

    let mut combined_list = File::create("blocklist.txt")?;

    for f in local_blocklists {
        let f = f.unwrap();
        let valid_lines = valid_lines(f);
        for line in valid_lines {
            writeln!(combined_list, "{}", line)?;
        }
    }

    Ok(())
}
