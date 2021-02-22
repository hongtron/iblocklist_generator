use std::fmt::Write;
use std::sync::Arc;
use async_std::sync::Mutex;
use async_std::task;
use flate2::bufread::GzDecoder;
use futures::future::join_all;
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

fn store_from_blocklists(vec: &Vec<Blocklist>) -> Arc<Mutex<Vec<Vec<u8>>>> {
    Arc::new(Mutex::new(Vec::with_capacity(vec.len())))
}

async fn push_to_store(store: Arc<Mutex<Vec<Vec<u8>>>>, value: Vec<u8>) -> () {
    store.lock().await.push(value);
}

fn unwrap_store(store: Arc<Mutex<Vec<Vec<u8>>>>) -> Vec<Vec<u8>> {
    Arc::try_unwrap(store).unwrap().into_inner()
}


fn main() {
    let blocklists = vec![
        Blocklist { name: "level1", id: "ydxerpxkpcfqjaybcssw" },
        Blocklist { name: "level2", id: "gyisgnzbhppbvsphucsw" },
        Blocklist { name: "level3", id: "uwnukjqktoggdknzrhgh" },
    ];

    let store = store_from_blocklists(&blocklists);
    let mut fetches = Vec::with_capacity(blocklists.len());

    for bl in blocklists {
        let store_ref = store.clone();
        fetches.push(task::spawn(async move { push_to_store(store_ref, bl.fetch().await) }))
    }

    let fetch_all = join_all(fetches);
    task::block_on(async { fetch_all.await });
    let raw_contents = unwrap_store(store);

    let empty_line_or_comment: Regex = Regex::new(r"(^$|^#.*$)").unwrap();

    let mut combined_contents = String::new();
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
