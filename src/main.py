import asyncio
import websockets

ffmpeg_cmd = [
    "ffmpeg",
    "-i", "-",
    "-f", "dash", 
    "-seg_duration", "2",
    "./output/dash/manifest.mpd",
]

async def handle_stream(websocket):
    print("Connection opened")
    ffmpeg_process = await asyncio.create_subprocess_exec(
        *ffmpeg_cmd, stdin=asyncio.subprocess.PIPE
    )
    
    async for message in websocket:
        # Output #1 MPEG-DASH
        ffmpeg_process.stdin.write(message)
        await ffmpeg_process.stdin.drain()


        # Output #2 local file
        with open('./output/received.webm', 'ab') as f:
            f.write(message)

start_server = websockets.serve(handle_stream, 'localhost', 8080)

asyncio.get_event_loop().run_until_complete(start_server)
asyncio.get_event_loop().run_forever()