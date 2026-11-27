#[cfg(windows)]
use std::process::Command;

pub fn open_url(url: &str) {
    #[cfg(windows)]
    let result = Command::new("rundll32")
        .arg("url.dll,FileProtocolHandler")
        .arg(url)
        .spawn();
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(url).spawn();
    #[cfg(all(not(windows), not(target_os = "macos")))]
    let result = std::process::Command::new("xdg-open").arg(url).spawn();
    if let Err(err) = result {
        log::warn!("failed to open {url}: {err}");
    }
}
