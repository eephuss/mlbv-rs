use crate::cli::Cli;
use std::io;
use std::path::PathBuf;
use std::process::Command;

fn find_in_path(command: &str) -> anyhow::Result<PathBuf> {
    #[cfg(target_os = "windows")]
    let output = Command::new("where").arg(command).output()?;

    #[cfg(not(target_os = "windows"))]
    let output = Command::new("which").arg(command).output()?;

    if !output.status.success() {
        tracing::warn!(%command, "Command not found in PATH");
        anyhow::bail!("Command '{}' not found in PATH", command);
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<_> = stdout_str.lines().map(str::trim).collect();

    let path = lines
        .iter()
        .find(|s| s.ends_with(".exe"))
        .or_else(|| lines.first())
        .ok_or_else(|| {
            anyhow::anyhow!("Found entries for '{}' but no valid executable", command)
        })?;

    tracing::debug!("Found {command} at {path}");
    Ok(PathBuf::from(path))
}

fn resolve_media_player(media_player: Option<&str>) -> io::Result<(String, Vec<String>)> {
    // Use specified player if found in PATH
    if let Some(m_player) = media_player
        && let Ok(path) = find_in_path(m_player)
    {
        return Ok((path.to_string_lossy().into_owned(), Vec::new()));
    }

    tracing::warn!("No valid media_player provided; falling back to system default player");

    #[cfg(target_os = "windows")]
    {
        Ok(("cmd".into(), vec!["/C".into(), "start".into()]))
    }

    #[cfg(target_os = "macos")]
    {
        Ok(("open".into(), Vec::new()))
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Ok(("xdg-open".into(), Vec::new()))
    }
}

pub fn play_stream_url(url: String, media_player: Option<&str>) -> anyhow::Result<()> {
    let (command, mut args) = resolve_media_player(media_player)?;
    args.push(url);

    let mut child = Command::new(command).args(args).spawn()?;

    let status = child.wait()?;
    match status.success() {
        true => Ok(()),
        false => anyhow::bail!("Media player exited with status: {}", status),
    }
}

pub fn handle_playback_url(url: String, cli: &Cli, media_player: Option<&str>) -> anyhow::Result<()> {
    match cli.url {
        true => {
            println!("{url}");
            Ok(())
        }
        false => play_stream_url(url, media_player),
    }
}