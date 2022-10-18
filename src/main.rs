use reqwest::Client;
use futures::{stream, future, StreamExt};
use std::fmt::Write;
use flate2::bufread::GzDecoder;
use regex::Regex;
use std::io::Read;


const CONCURRENT_REQUESTS: usize = 2;

struct Blocklist<'a> {
    id: &'a str,
    name: &'a str,
}

impl<'a> Blocklist<'a> {
    fn uri(&self) -> String {
        format!("https://list.iblocklist.com/?list={}&fileformat=p2p&archiveformat=gz", self.id)
    }
}

#[tokio::main]
async fn main() {
    let blocklists = vec![
        Blocklist { name: "level1", id: "ydxerpxkpcfqjaybcssw" },
        Blocklist { name: "level2", id: "gyisgnzbhppbvsphucsw" },
        Blocklist { name: "level3", id: "uwnukjqktoggdknzrhgh" },
    ];

    let client = Client::new();

    let bodies = stream::iter(blocklists.iter().map(|b| { b.uri() }))
        .map(|uri| {
            let client = &client;
            async move {
                let resp = client.get(uri).send().await.unwrap();
                let gz_body = resp.bytes().await.unwrap();
                let mut decoded_contents = String::new();
                GzDecoder::new(&gz_body[..]).read_to_string(&mut decoded_contents).unwrap();
                decoded_contents
            }
        })
    .buffer_unordered(CONCURRENT_REQUESTS);

    let mut combined_contents = vec![];
    bodies.for_each(|b| {
        combined_contents.push(b);
        future::ready(())
    }).await;

    let empty_line_or_comment: Regex = Regex::new(r"(^$|^#.*$)").unwrap();
    let final_list: Vec<&str> = combined_contents.iter().flat_map(|x| x.lines())
        .filter(|l| !empty_line_or_comment.is_match(l))
        .collect();

    for entry in final_list {
        println!("{}", entry)
    }
}
