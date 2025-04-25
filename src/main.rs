use std::cmp::min;
use std::fs::File;
use std::io::Write;

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::redirect::{Attempt, Policy};
use reqwest::Client;

pub async fn download_file(client: &Client, url: &str, path: &str) -> Result<(), String> {
    // Reqwest setup
    let res = client
        .get(url)
        .send()
        .await
        .or(Err(format!("Failed to GET from '{}'", &url)))?;
    let total_size = res
        .content_length()
        .ok_or(format!("Failed to get content length from '{}'", &url))?;

    // Indicatif setup
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap()
        .progress_chars("#>-"));
    let download_message = format!("Downloading {}", url);
    pb.set_message(download_message);

    // download chunks
    let mut file = File::create(path).or(Err(format!("Failed to create file '{}'", path)))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(format!("Error while downloading file")))?;
        file.write_all(&chunk)
            .or(Err(format!("Error while writing to file")))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    let finish_message = format!("Downloaded {} to {}", url, path);
    pb.finish_with_message(finish_message);
    return Ok(());
}

#[tokio::main]
async fn main() {
    let redirect_rules: Policy = Policy::custom(|attempt: Attempt<'_>| {
        if attempt.previous().len() > 5 {
            attempt.stop()
        } else {
            attempt.follow()
        }
    });
    let client: Client = Client::builder().redirect(redirect_rules).build().unwrap();

    download_file(
        &client,
        "https://download.sublimetext.com/sublime_text_build_4196_x64.zip",
        "sublime_text_build_4196_x64.zip",
    )
    .await
    .expect("Could not download file");
}
