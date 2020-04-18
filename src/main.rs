use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use tempfile::tempfile;
use flate2::read::GzDecoder;
use std::fmt::Error;
use std::path::Path;

fn blocklist_uri(id: &str) -> String {
    format!("https://list.iblocklist.com/?list={}&fileformat=p2p&archiveformat=gz", id)
}

fn get_blocklist(id: &str) -> Result<File, io::Error> {
    let pathstr = format!("/tmp/{}.gz", id);
    let path = Path::new(&pathstr);
    // fs::create_dir_all(path).unwrap();
    // let mut dest = File::create(&format!("/tmp/{}.gz", id)).unwrap();
    // let mut dest = tempfile()?;
    let resource = format!("https://list.iblocklist.com/?list={}&fileformat=p2p&archiveformat=gz", id);
    let body = reqwest::blocking::get(&resource).unwrap().bytes().unwrap();
    let mut dest= File::create(path).unwrap();
    println!("{:?}", dest);
    dest.write_all(&body);
    let reader = File::open(path)?;
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
        println!("file for gz {:?}", f);
        let mut gz = GzDecoder::new(f);
        let pathstr = format!("/tmp/{}.txt", list_name);
        let path = Path::new(&pathstr);
        let mut file = File::create(path).unwrap();
        io::copy(&mut gz, &mut file).unwrap();
    };

    Ok(())
}
