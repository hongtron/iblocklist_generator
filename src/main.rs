use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{Write, BufReader, BufRead};
use tempfile::NamedTempFile;
use flate2::read::GzDecoder;
use anyhow::Result;
use regex::Regex;

fn blocklist_uri(id: &str) -> String {
    format!("https://list.iblocklist.com/?list={}&fileformat=p2p&archiveformat=gz", id)
}

fn get_blocklist(id: &str) -> Result<File> {
    let resource = blocklist_uri(id);
    let body = reqwest::blocking::get(&resource)?.bytes()?;
    let mut dest = NamedTempFile::new()?;
    dest.write_all(&body)?;
    let reader = dest.reopen()?;
    Ok(reader)
}

fn main() -> Result<()> {
    let blocklists: HashMap<&str, &str> = [
        ("level1", "ydxerpxkpcfqjaybcssw"),
        ("level2", "gyisgnzbhppbvsphucsw"),
        ("level3", "uwnukjqktoggdknzrhgh"),
    ].iter().cloned().collect();

    let local_blocklists: Vec<File> = blocklists.iter().map(|(list_name, id)| {
        println!("Downloading I-Blocklist {}...", list_name);
        get_blocklist(&id).unwrap()
    }).collect();

    let decompressed_blocklists: Vec<Result<File>> = local_blocklists.iter().map(|f| {
        let mut gz = GzDecoder::new(f);
        let mut list_out = NamedTempFile::new()?;
        io::copy(&mut gz, &mut list_out).unwrap();
        let reader = list_out.reopen()?;
        Ok(reader)
    }).collect();

    let mut combined_list = File::create("blocklist.txt")?;
    let empty_line_or_comment: Regex = Regex::new(r"(^$|^#.*$)").unwrap();

    for f in decompressed_blocklists {
        let f = f.unwrap();
        let r = BufReader::new(f);
        let valid_lines = r.lines()
            .map(|l| l.unwrap())
            .filter(|line| !empty_line_or_comment.is_match(line));
        for line in valid_lines {
            writeln!(combined_list, "{}", line)?;
        }
    }

    Ok(())
}
