# Stream-Video
This project demonstrates concurrent live video caputure, analysis, re-encoding, and streaming. It uses a Rust server, WebSockets, FFmpeg, and MPEG-DASH.


## Dependencies
* [Rust](https://rustup.rs/)
* [FFmpeg](https://ffmpeg.org/)
* Python
* Web browser
* Webcam


## Use

1. Clone the repo:
```sh
    git clone https://github.com/notanaccent/stream-video.git
    cd stream-video
```

2. Execute the example script:
```sh
    chmod +x ./example.sh
    ./example.sh
```

3. Wait:
* The server will build and begin running
* A webcam capture webpage will open and ask for access
* The stream will begin re-encoding
* 15 seconds later, another webpage will open
* Click the button to begin viewing the live stream
* Kill the example with any key


## How it works

There are 5 seperate processes:
* Webpage capturing the webcam stream
* Rust server receiving the video via websocket
* FFmpeg converting the video MPEG-DASH
* Python serving the MPEG-DASH files 
* Webpage viewing the converted stream

### Capture
[capture.html](./static/capture.html)
is a client web page that is sending their video the server. In this case, the user's webcam is captured with MediaCapture, encoded with h264, and sent to the websocket at localhost:8080 once every second.

### Rust Server
[main.rs](./src/main.rs) is a Rust server listening to the websocket at localhost:8080. When a connection is made, it starts ffmpeg as a subprocess. When messages come in, the binary data of the video stream is passed to several outputs:
* ffmpeg via stdin to reencode as MPEG-DASH
* streamed in the original encoding to ./output/received.webm
* decoded frame-by-frame for analysis within the rust server 

### FFmpeg
FFmpeg receives the original video stream from Rust via stdin and converts the stream to MPEG-DASH. The result is saved to a series of files in ./output/dash. The equivalent command is ``ffmpeg -i video.mp4 -f dash manifest.mpd``.

### Python Server
Python's http server is used to serve this project's files on localhost:8000. This lets the viewer access the MPEG-DASH files. This is for the purpose of this example only.

### Viewer
[viewer.html](./static/viewer.html)
is a client web page that displays the live video stream. It simply gives [dash.js](https://dashjs.org/) the location of the MPEG-DASH manifest file and dash.js handles the rest.


## Performance
The main performance concern is the websocket server keeping up with reading and processing the incoming stream. The slowest part in doing so is passing the data to FFmpeg via stdin. With a some approaches, this can take so long that the websocket server begins to lag behind the incoming stream.

There is about a 10 second delay between capture and display. This can be halved by adding a bunch of flags to the ffmpeg pipe and reducing the quality of the video capture. This type of video streaming is likely not suitable for applications requiring sub-second latency. Even without a network, consider the sum of these delays:
* Video capture on the client-side is encoded in chunks and is currently configured to send once per second
* Websocket data received by the server is passed to FFmpeg through stdin, currently taking 200ms per 1s of video
* FFmpeg reencoding time
* The resulting format, MPEG-DASH, is encoded in segments of several seconds


## Python instead of Rust
It works too.

```python
import asyncio
import websockets

ffmpeg_cmd = [
    "ffmpeg",
    "-i", "-",
    "-f", "dash", 
    "./output/dash/manifest.mpd",
]

async def handle_stream(websocket):
    ffmpeg_process = await asyncio.create_subprocess_exec(
        *ffmpeg_cmd, stdin=asyncio.subprocess.PIPE
    )
    
    async for message in websocket:
        ffmpeg_process.stdin.write(message)
        await ffmpeg_process.stdin.drain()

start_server = websockets.serve(handle_stream, 'localhost', 8080)

asyncio.get_event_loop().run_until_complete(start_server)
asyncio.get_event_loop().run_forever()
```