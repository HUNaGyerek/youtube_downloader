use std::path::Path;
use std::process::Command;

#[cfg(target_os = "linux")]
pub fn detect_linux_distro() -> Option<String> {
    // Try reading /etc/os-release first (most modern distros)

    use std::fs;
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("ID=") {
                return Some(line[3..].trim_matches('"').to_lowercase());
            }
        }
    }

    // Try reading /etc/lsb-release (Ubuntu and some others)
    if let Ok(content) = fs::read_to_string("/etc/lsb-release") {
        for line in content.lines() {
            if line.starts_with("DISTRIB_ID=") {
                return Some(line[11..].trim_matches('"').to_lowercase());
            }
        }
    }

    // Check for specific files that indicate certain distributions
    if Path::new("/etc/arch-release").exists() {
        return Some("arch".to_string());
    }

    if Path::new("/etc/fedora-release").exists() {
        return Some("fedora".to_string());
    }

    if Path::new("/etc/redhat-release").exists() {
        return Some("rhel".to_string());
    }

    if Path::new("/etc/debian_version").exists() {
        return Some("debian".to_string());
    }

    // Try using command line tools as a last resort
    if Command::new("lsb_release")
        .arg("-is")
        .output()
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_lowercase()
        })
        .ok()
        .filter(|s| !s.is_empty())
        .is_some()
    {
        return Command::new("lsb_release")
            .arg("-is")
            .output()
            .ok()
            .and_then(|output| {
                let dist = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_lowercase();
                if !dist.is_empty() {
                    Some(dist)
                } else {
                    None
                }
            });
    }

    None
}

pub fn check_ffmpeg() -> bool {
    Command::new("ffmpeg").arg("-version").output().is_ok()
}

pub fn check_yt_dlp() -> bool {
    // Check for yt-dlp in various possible locations
    let check_default = Command::new("yt-dlp").arg("--version").output().is_ok();

    if check_default {
        return true;
    }

    // On some systems, it might be installed but not in PATH
    #[cfg(target_os = "windows")]
    {
        let home_dir = dirs::home_dir();
        if let Some(home) = home_dir {
            let local_path = home
                .join("AppData")
                .join("Local")
                .join("yt-dlp")
                .join("yt-dlp.exe");
            if local_path.exists() {
                return true;
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Check in common Linux locations
        for path in ["/usr/local/bin/yt-dlp", "/usr/bin/yt-dlp", "/bin/yt-dlp"] {
            if Path::new(path).exists() {
                return true;
            }
        }
    }

    false
}

pub fn get_yt_dlp_path() -> Option<String> {
    // First try the command directly
    if Command::new("yt-dlp").arg("--version").output().is_ok() {
        return Some("yt-dlp".to_string());
    }

    #[cfg(target_os = "linux")]
    {
        // Check in common Linux locations
        for path in ["/usr/local/bin/yt-dlp", "/usr/bin/yt-dlp", "/bin/yt-dlp"] {
            if Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let home_dir = dirs::home_dir();
        if let Some(home) = home_dir {
            let local_path = home
                .join("AppData")
                .join("Local")
                .join("yt-dlp")
                .join("yt-dlp.exe");
            if local_path.exists() {
                return Some(local_path.to_string_lossy().to_string());
            }
        }
    }

    None
}

pub fn install_ffmpeg() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        println!("Installing ffmpeg via winget...");
        let status = Command::new("winget")
            .args(&["install", "ffmpeg"])
            .status()?;

        if !status.success() {
            return Err("Failed to install ffmpeg".into());
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        println!("Installing ffmpeg...");

        // Detect distribution
        if let Some(distro) = detect_linux_distro() {
            println!("Detected Linux distribution: {}", distro);

            // Install based on distribution
            match distro.as_str() {
                "arch" | "manjaro" | "endeavouros" => {
                    println!("Using pacman for installation...");
                    if Command::new("sudo")
                        .args(&["pacman", "-S", "--noconfirm", "ffmpeg"])
                        .status()
                        .is_ok()
                    {
                        return Ok(());
                    }
                }
                "ubuntu" | "debian" | "linuxmint" | "pop" => {
                    println!("Using apt for installation...");
                    if Command::new("sudo")
                        .args(&["apt", "install", "-y", "ffmpeg"])
                        .status()
                        .is_ok()
                    {
                        return Ok(());
                    }
                }
                "fedora" | "rhel" | "centos" | "rocky" | "alma" => {
                    println!("Using dnf/yum for installation...");
                    // Try dnf first (newer)
                    if Command::new("sudo")
                        .args(&["dnf", "install", "-y", "ffmpeg"])
                        .status()
                        .is_ok()
                    {
                        return Ok(());
                    }

                    // Fall back to yum
                    if Command::new("sudo")
                        .args(&["yum", "install", "-y", "ffmpeg"])
                        .status()
                        .is_ok()
                    {
                        return Ok(());
                    }
                }
                "opensuse" | "suse" => {
                    println!("Using zypper for installation...");
                    if Command::new("sudo")
                        .args(&["zypper", "install", "-y", "ffmpeg"])
                        .status()
                        .is_ok()
                    {
                        return Ok(());
                    }
                }
                _ => {
                    println!("Using generic installation methods for unknown distribution...");
                    // Continue to generic methods below
                }
            }
        } else {
            println!("Could not detect Linux distribution, trying common package managers...");
        }

        return Err("Could not install ffmpeg with any known package manager".into());
    }
}

pub fn install_yt_dlp() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "windows")]
    {
        println!("Installing yt-dlp via winget...");
        let status = Command::new("winget").args(&["install", "yt-dlp"]).status();
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        println!("Installing yt-dlp...");

        // Detect distribution
        if let Some(distro) = detect_linux_distro() {
            println!("Detected Linux distribution: {}", distro);

            // Install based on distribution
            match distro.as_str() {
                "arch" | "manjaro" | "endeavouros" => {
                    println!("Using pacman for installation...");
                    if Command::new("sudo")
                        .args(&["pacman", "-S", "--noconfirm", "yt-dlp"])
                        .status()
                        .is_ok()
                    {
                        return Ok(());
                    }
                }
                "ubuntu" | "debian" | "linuxmint" | "pop" => {
                    println!("Using apt for installation...");
                    if Command::new("sudo")
                        .args(&["apt", "install", "-y", "yt-dlp"])
                        .status()
                        .is_ok()
                    {
                        return Ok(());
                    }
                }
                "fedora" | "rhel" | "centos" | "rocky" | "alma" => {
                    println!("Using dnf/yum for installation...");
                    // Try dnf first (newer)
                    if Command::new("sudo")
                        .args(&["dnf", "install", "-y", "yt-dlp"])
                        .status()
                        .is_ok()
                    {
                        return Ok(());
                    }

                    // Fall back to yum
                    if Command::new("sudo")
                        .args(&["yum", "install", "-y", "yt-dlp"])
                        .status()
                        .is_ok()
                    {
                        return Ok(());
                    }
                }
                "opensuse" | "suse" => {
                    println!("Using zypper for installation...");
                    if Command::new("sudo")
                        .args(&["zypper", "install", "-y", "yt-dlp"])
                        .status()
                        .is_ok()
                    {
                        return Ok(());
                    }
                }
                _ => {
                    println!("Using generic installation methods for unknown distribution...");
                    // Continue to generic methods below
                }
            }
        } else {
            println!("Could not detect Linux distribution, trying common package managers...");
        }

        // If both methods failed, suggest manual installation
        println!("Could not install yt-dlp automatically.");
        println!("Please try manually with one of these commands:");
        println!("  sudo pacman -S yt-dlp        # Arch Linux");
        println!("  sudo apt install yt-dlp      # Debian/Ubuntu");
        println!("  sudo dnf install yt-dlp      # Fedora");
        println!("  sudo yum install yt-dlp      # CentOS/RHEL");
        println!("  pip install --user yt-dlp    # Any Linux");

        return Err("Could not install yt-dlp with any known method".into());
    }

    #[cfg(target_os = "macos")]
    {
        println!("Installing yt-dlp via Homebrew...");
        if Command::new("brew")
            .args(&["install", "yt-dlp"])
            .status()
            .is_ok()
        {
            return Ok(());
        }

        // If Homebrew fails, try direct installation
        println!("Installing yt-dlp directly...");
        let download_url = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp";

        if Command::new("curl")
            .args(&["-L", download_url, "-o", "/usr/local/bin/yt-dlp"])
            .status()
            .is_ok()
        {
            // Make executable
            Command::new("chmod")
                .args(&["+x", "/usr/local/bin/yt-dlp"])
                .status()?;

            return Ok(());
        }

        return Err("Failed to install yt-dlp".into());
    }
}
