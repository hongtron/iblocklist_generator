use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Write;
use tempfile::NamedTempFile;
use flate2::read::GzDecoder;

fn blocklist_uri(id: &str) -> String {
    format!("https://list.iblocklist.com/?list={}&fileformat=p2p&archiveformat=gz", id)
}

fn get_blocklist(id: &str) -> Result<File, io::Error> {
    let resource = blocklist_uri(id);
    let body = reqwest::blocking::get(&resource).unwrap().bytes().unwrap();
    let mut dest = NamedTempFile::new()?;
    dest.write_all(&body)?;
    let reader = dest.reopen()?;
    Ok(reader)
}

fn main() -> Result<(), io::Error> {
    let blocklists: HashMap<&str, &str> = [
        ("level1", "ydxerpxkpcfqjaybcssw"),
        ("level2", "gyisgnzbhppbvsphucsw"),
        ("level3", "uwnukjqktoggdknzrhgh"),
    ].iter().cloned().collect();

    let local_blocklists: Vec<(&&str, File)> = blocklists.iter().map(|(list_name, id)| {
        println!("Downloading I-Blocklist {}...", list_name);
        (list_name, get_blocklist(&id).unwrap())
    }).collect();

    for (list_name, f) in local_blocklists {
        let mut gz = GzDecoder::new(f);
        let mut list_out = File::create(format!("/tmp/{}.txt", list_name)).unwrap();
        io::copy(&mut gz, &mut list_out).unwrap();
    };

    Ok(())
}
