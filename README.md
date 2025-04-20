# YouTube Downloader

A command-line tool for downloading YouTube videos and playlists as MP3 files.

## Features

- Download single videos or entire playlists
- Convert videos to MP3 format with metadata and thumbnails
- Queue multiple downloads
- Track download history
- Parallel downloading for faster processing
- Automatic dependency installation (ffmpeg, yt-dlp)

## Requirements

- Rust and Cargo
- Internet connection
- The application will attempt to automatically install:
  - ffmpeg
  - yt-dlp

## Installation

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/youtube-downloader.git
   cd youtube-downloader
   ```

2. Build with Cargo:
   ```
   cargo build --release
   ```

3. Run the application:
   ```
   cargo run --release
   ```

## Usage

The application provides a simple menu interface:

1. **Add URL to download queue** - Add a YouTube video or playlist URL
2. **List queued downloads** - See what's in your download queue
3. **Start downloads** - Begin downloading all queued videos
4. **Set download directory** - Choose where to save the downloaded files
5. **View download history** - See previously downloaded videos
6. **Clear download queue** - Remove all items from the queue
7. **Exit** - Close the application

## License

MIT
