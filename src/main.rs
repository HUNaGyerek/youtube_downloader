use sevenz_rust::decompress;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::Path;
use youtube_dl::downloader::YoutubeDlFetcher;
use youtube_dl::YoutubeDl;

async fn download_ffmpeg() -> Result<(), Box<dyn Error>> {
    let ffmpeg_path = ".\\ffmpeg\\bin\\ffmpeg.exe";
    if !Path::new(ffmpeg_path).exists() {
        // URL for a 7z file
        let url = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-git-essentials.7z";
        let response = reqwest::get(url).await?;
        if !response.status().is_success() {
            eprintln!("Failed to fetch the file: HTTP {}", response.status());
            return Err("Download failed".into());
        }
        let bytes = response.bytes().await?;

        // Save the 7z file locally
        let archive_path = ".\\ffmpeg.7z";
        let mut file = std::fs::File::create(archive_path)?;
        file.write_all(&bytes)?;

        // Decompress the 7z archive
        let extract_path = ".\\";
        fs::create_dir_all(extract_path)?;
        let archive_file = fs::File::open(archive_path)?; // Open the 7z file
        decompress(archive_file, extract_path)?;

        // Remove the .7z file
        if Path::new(archive_path).exists() {
            std::fs::remove_file(archive_path)?;
            println!("Removed archive file: {:?}", archive_path);
        }

        // Find the extracted folder with a specific name pattern
        let extracted_folder = std::fs::read_dir(extract_path)?
            .find_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_dir() && path.file_name()?.to_string_lossy().contains("ffmpeg") {
                    Some(path)
                } else {
                    None
                }
            })
            .ok_or("Failed to locate the extracted folder")?;

        let target_folder = Path::new(extract_path).join("ffmpeg");
        if extracted_folder != target_folder {
            std::fs::rename(&extracted_folder, &target_folder)?;
            println!("Renamed {:?} to {:?}", extracted_folder, target_folder);
        }
    }

    Ok(())
}

async fn download_yt_dlp() -> Result<(), Box<dyn Error>> {
    // Download yt-dlp if not exists
    let fetcher = YoutubeDlFetcher::default();
    let _ = fetcher.download(".\\ytdlp\\").await;
    Ok(())
}

async fn download_music(url: &str) -> Result<(), Box<dyn Error>> {
    YoutubeDl::new(url)
        .extract_audio(true)
        .extra_arg("--audio-format")
        .extra_arg("mp3")
        .youtube_dl_path(".\\ytdlp\\yt-dlp.exe")
        .extra_arg("--ffmpeg-location")
        .extra_arg(".\\ffmpeg\\bin\\ffmpeg.exe")
        .extra_arg("--embed-metadata")
        // .extra_arg("--list-thumbnails")
        .extra_arg("--embed-thumbnail")
        .output_template("%(title)s.%(ext)s")
        .download_to_async(".\\songs\\")
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(e) = download_ffmpeg().await {
        eprintln!("Failed to download and extract ffmpeg: {}", e)
    }
    if let Err(e) = download_yt_dlp().await {
        eprintln!("Failed to download and extract yt-dlp: {}", e)
    }

    let args = std::env::args().collect::<Vec<_>>();
    if args.len() == 1 {
        let url = "https://www.youtube.com/watch?v=bAMbi0E3AkI&list=RDbAMbi0E3AkI";
        download_music(url).await?;
    } else {
        let url = &args[1];
        download_music(url).await?;
    }
    Ok(())
}
