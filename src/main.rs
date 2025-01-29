use rayon::prelude::*;
use serde::Deserialize;
use youtube_dl::YoutubeDl;

#[derive(Debug, Deserialize)]
struct Music {
    url: String,
}

fn fetch_playlist_videos(url: &str) -> Result<Vec<Music>, Box<dyn std::error::Error>> {
    let output = YoutubeDl::new(url)
        .flat_playlist(true) // Get the full playlist info
        .socket_timeout("15")
        .run()?;

    let mut videos = Vec::new();

    if let Some(playlist) = output.clone().into_playlist() {
        if let Some(entries) = playlist.entries {
            for video in entries {
                videos.push(Music {
                    url: video.url.unwrap_or_else(|| "Unknown".to_string()),
                });
            }
        } else {
            return Err("No videos found".into());
        }
    } else {
        videos.push(Music {
            url: url.to_string(),
        });
    }
    
    Ok(videos)
}

fn download_video(url: &str) {
    let status = YoutubeDl::new(url)
        .extract_audio(true)
        .extra_arg("--audio-format")
        .extra_arg("mp3")
        .extra_arg("--embed-metadata")
        // .extra_arg("--list-thumbnails")
        .extra_arg("--embed-thumbnail")
        .output_template("%(title)s.%(ext)s")
        .download_to("./Music");

    match status {
        Ok(_) => {
            println!("Downloaded {}", url);
        }
        Err(e) => {
            println!("Error downloading {}: {}", url, e);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let playlist_url = "https://www.youtube.com/watch?v=bAMbi0E3AkI"; // &list=RDbAMbi0E3AkI
    match fetch_playlist_videos(playlist_url) {
        Ok(videos) => {
            videos
                .par_iter()
                .map(|video| download_video(&video.url))
                .count();
        }
        Err(e) => eprintln!("Error fetching playlist: {}", e),
    }
    Ok(())
}
