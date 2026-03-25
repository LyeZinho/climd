use std::process::Command;

const REPO_OWNER: &str = "LyeZinho";
const REPO_NAME: &str = "climd";

pub fn run_update() -> Result<(), String> {
    println!("Checking for updates...");

    let current_version = env!("CARGO_PKG_VERSION");
    println!("Current version: {}", current_version);

    let latest_release = fetch_latest_release()?;
    let latest_version = latest_release.tag_name.trim_start_matches('v');

    println!("Latest version: {}", latest_version);

    if latest_version > current_version {
        println!("Newer version available! Updating...");

        let asset = find_appropriate_asset(&latest_release.assets)?;
        download_and_replace(&asset.browser_download_url)?;

        println!("Update complete! Please restart climd.");
    } else {
        println!("Already on the latest version.");
    }

    Ok(())
}

fn fetch_latest_release() -> Result<GitHubRelease, String> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        REPO_OWNER, REPO_NAME
    );

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "climd-update")
        .header("Accept", "application/vnd.github+json")
        .send()
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("GitHub API error: {}", response.status()));
    }

    let text = response.text().map_err(|e| e.to_string())?;
    let release: GitHubRelease = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    Ok(release)
}

fn find_appropriate_asset(assets: &[GitHubAsset]) -> Result<&GitHubAsset, String> {
    let target = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let (os, arch_str) = match (target, arch) {
        ("windows", "x86_64") => ("windows", "x86_64-pc-windows-msvc"),
        ("linux", "x86_64") => ("linux", "x86_64-unknown-linux-gnu"),
        ("linux", "aarch64") => ("linux", "aarch64-unknown-linux-gnu"),
        ("macos", "x86_64") => ("darwin", "x86_64-apple-darwin"),
        ("macos", "aarch64") => ("darwin", "aarch64-apple-darwin"),
        _ => return Err(format!("Unsupported platform: {}-{}", target, arch)),
    };

    let expected_name = format!("climd-{}.tar.gz", arch_str);

    assets
        .iter()
        .find(|a| a.name.contains(arch_str) || a.name == expected_name)
        .ok_or_else(|| format!("No asset found for {}-{}", os, arch_str))
}

fn download_and_replace(url: &str) -> Result<(), String> {
    println!("Downloading from: {}", url);

    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send().map_err(|e| e.to_string())?;

    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("climd_update.tar.gz");

    let mut file = std::fs::File::create(&temp_file).map_err(|e| e.to_string())?;
    let mut bytes = response.bytes().map_err(|e| e.to_string())?;
    std::io::Write::write_all(&mut file, &mut bytes).map_err(|e| e.to_string())?;

    let extract_dir = temp_dir.join("climd_update");
    std::fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;

    let output = Command::new("tar")
        .arg("xzf")
        .arg(&temp_file)
        .arg("-C")
        .arg(&extract_dir)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("Failed to extract archive".to_string());
    }

    let binary_path = extract_dir.join(if cfg!(windows) { "climd.exe" } else { "climd" });

    if let Ok(current_exe) = std::env::current_exe() {
        let backup_path = current_exe.with_extension("exe.bak");
        if std::fs::copy(&current_exe, &backup_path).is_ok() {
            std::fs::copy(&binary_path, &current_exe).map_err(|e| e.to_string())?;
            println!("Updated binary at: {:?}", current_exe);
            let _ = std::fs::remove_file(backup_path);
        }
    }

    let _ = std::fs::remove_file(temp_file);
    let _ = std::fs::remove_dir_all(extract_dir);

    Ok(())
}

#[derive(serde::Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(serde::Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}
