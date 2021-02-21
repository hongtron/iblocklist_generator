use std::fmt::Write;
use std::sync::Arc;
use async_std::sync::Mutex;
use async_std::task;
use flate2::bufread::GzDecoder;
use regex::Regex;
use std::io::Read;

struct Blocklist<'a> {
    id: &'a str,
    name: &'a str,
}

impl<'a> Blocklist<'a> {
    async fn fetch(&self) -> Vec<u8> {
        surf::get(self.uri()).recv_bytes().await.unwrap()
    }

    fn uri(&self) -> String {
        format!("https://list.iblocklist.com/?list={}&fileformat=p2p&archiveformat=gz", self.id)
    }
}

fn main() {
    let blocklists = vec![
        Blocklist { name: "level1", id: "ydxerpxkpcfqjaybcssw" },
        Blocklist { name: "level2", id: "gyisgnzbhppbvsphucsw" },
        Blocklist { name: "level3", id: "uwnukjqktoggdknzrhgh" },
    ];

    let mut tasks = Vec::with_capacity(blocklists.len());
    let raw_contents = Arc::new(Mutex::new(Vec::with_capacity(blocklists.len())));

    for blocklist in blocklists.into_iter() {
        let rc = Arc::clone(&raw_contents);
        tasks.push(
            task::spawn(
                async move {
                    rc.lock().await.push(blocklist.fetch().await)
                }
            )
        )
    };

    task::block_on(async {
        for t in tasks {
            t.await;
        }
    });

    let empty_line_or_comment: Regex = Regex::new(r"(^$|^#.*$)").unwrap();
    let mut combined_contents = String::new();
    let raw_contents = task::block_on(async { raw_contents.lock().await });
    raw_contents.iter()
        .map(|bytes| GzDecoder::new(&bytes[..])
            .read_to_string(&mut combined_contents)
            .unwrap()
        )
        .for_each(drop);

    let final_list: Vec<&str> = combined_contents
        .lines()
        .filter(|l| !empty_line_or_comment.is_match(l))
        .collect();

    for entry in final_list {
        writeln!(String::from(entry)).unwrap()
    }
}
