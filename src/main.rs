use futures::stream::StreamExt;
use openh264::decoder::{DecodedYUV, Decoder};
use openh264::{nal_units, OpenH264API};
use std::fs;
use std::io::Write;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Command;
use tokio_tungstenite::accept_async;
use tungstenite::Message;

#[tokio::main]
async fn main() {
    // Create TCP listener
    let listener = TcpListener::bind("localhost:8080").await.unwrap();
    println!("WebSocket server listening on port 8080...");

    // Accept one incoming connection
    let mut awaiting_connection = true;
    while let Ok((stream, _)) = listener.accept().await {
        if awaiting_connection {
            awaiting_connection = false;
            tokio::spawn(handle_connection(stream));
        }
    }
}

async fn handle_connection(stream: TcpStream) {
    // Input stream
    let ws_stream = accept_async(stream)
        .await
        .expect("Error during WebSocket handshake");
    let (_, mut ws_read) = ws_stream.split();

    // Output #1 local file
    let _ = fs::remove_dir_all("./output/dash").unwrap();
    fs::create_dir_all("./output/dash").unwrap();
    let mut file = fs::File::create("./output/received.webm").unwrap();

    // Output #2 decoded frames
    let api = OpenH264API::from_source();
    let mut decoder = Decoder::new(api).unwrap();

    // Output #3 ffmpeg mpeg dash
    let mut ffmpeg_process = Command::new("ffmpeg")
        .arg("-i")
        .arg("-")
        .arg("-f")
        .arg("dash")
        // .arg("-x264-params")
        // .arg("keyint=10:min-keyint=10:scenecut=0")
        // .arg("-preset").arg("ultrafast")
        // .arg("-tune").arg("zerolatency")
        .arg("-seg_duration").arg("2")
        .arg("./output/dash/manifest.mpd")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut ffmpeg_stdin = BufWriter::new(ffmpeg_process.stdin.take().unwrap());

    // Handle messages
    while let Some(Ok(msg)) = ws_read.next().await {
        match msg {
            Message::Binary(data) => {
                // Longest task is writing to stdin, start it first
                let dataclone = data.clone();
                let ffmpeg_task = tokio::spawn(async move {
                    let _ = ffmpeg_stdin.flush().await;
                    let _ = ffmpeg_stdin.write_all(&dataclone).await;
                    ffmpeg_stdin
                });

                // Output #1 local file
                if let Err(e) = file.write_all(&data) {
                    eprintln!("Failed to save bytes to file: {}", e);
                }

                // Output #2 decode frame
                let mut frames_decoded = 0;
                for packet in nal_units(&data) {
                    if let Ok(Some(frame)) = decoder.decode(packet) {
                        analyze_frame(frame);
                        frames_decoded += 1;
                    }
                }

                // Output #3 ffmpeg
                ffmpeg_stdin = match ffmpeg_task.await {
                    Ok(s) => s,
                    Err(..) => BufWriter::new(ffmpeg_process.stdin.take().unwrap()),
                };

                println!(
                    "Received: {}KB and analyzed {} frames.",
                    (data.len() as f64) / 1e3,
                    frames_decoded
                );
            }
            _ => eprintln!("Unexpected message type"),
        }
    }
}

fn analyze_frame(_frame: DecodedYUV<'_>) {
    // TODO
}
