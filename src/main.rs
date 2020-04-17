use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use http::Uri;
use tempfile::TempDir;
use flate2::read::GzDecoder;

fn blocklist_uri(id: &str) -> String {
    format!("https://list.iblocklist.com/?list={}&fileformat=p2p&archiveformat=gz", id)
}

fn main() -> Result<(), std::io::Error> {
    let blocklists: HashMap<&str, &str> = [
        ("level1", "ydxerpxkpcfqjaybcssw"),
        ("level2", "gyisgnzbhppbvsphucsw"),
        ("level3", "uwnukjqktoggdknzrhgh"),
    ].iter().cloned().collect();

    let tmp_dir = TempDir::new()?;
    for (list_name, id) in &blocklists {
        println!("Downloading I-Blocklist {}...", list_name);
        let uri = blocklist_uri(id);
        let mut response = reqwest::get(&uri);
    }

    Ok(())
}
