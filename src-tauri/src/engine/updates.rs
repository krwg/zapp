// ╔════════════════════════════════════════════════════╗
// ║                  engine/updates                  ║
// ╚════════════════════════════════════════════════════╝

use std::fs::{self, File};
use std::io::{copy, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use crate::engine::github::{compare_semver, fetch_latest_release, FLOWSEAL_REPO};
use crate::engine::managed::EngineLayout;
use crate::error::{AppError, AppResult};
use crate::model::{UpdateChannel, UpdateChannelId, UpdateChannelStatus};
use crate::util::now_iso;

pub struct UpdateManager {
    channels: Mutex<Vec<UpdateChannel>>,
    app_data: PathBuf,
}

impl UpdateManager {
    pub fn new(channels: Vec<UpdateChannel>, app_data: PathBuf) -> Self {
        Self {
            channels: Mutex::new(channels),
            app_data,
        }
    }

    pub fn list(&self) -> Vec<UpdateChannel> {
        self.channels.lock().expect("updates lock").clone()
    }

    pub fn check_all(&self) -> AppResult<Vec<UpdateChannel>> {
        let release = fetch_latest_release().ok();
        let mut channels = self.channels.lock().expect("updates lock");
        let now = now_iso();

        for channel in channels.iter_mut() {
            channel.last_checked_at = Some(now.clone());
            match channel.id {
                UpdateChannelId::Engine | UpdateChannelId::Strategies => {
                    if let Some(ref rel) = release {
                        let latest = rel.version().to_string();
                        channel.latest_version = latest.clone();
                        channel.release_notes = rel
                            .body
                            .clone()
                            .unwrap_or_else(|| format!("Release {latest} from {FLOWSEAL_REPO}"));
                        channel.status =
                            if compare_semver(&latest, &channel.current_version).is_gt() {
                                UpdateChannelStatus::Available
                            } else {
                                UpdateChannelStatus::UpToDate
                            };
                    }
                }
                UpdateChannelId::Application => {
                    
                    channel.status = if channel.latest_version != channel.current_version {
                        UpdateChannelStatus::Available
                    } else {
                        UpdateChannelStatus::UpToDate
                    };
                }
            }
        }

        Ok(channels.clone())
    }

    pub fn update_channel(&self, id: UpdateChannelId) -> AppResult<(UpdateChannel, Option<EngineLayout>)> {
        match id {
            UpdateChannelId::Engine | UpdateChannelId::Strategies => self.update_flowseal_pack(id),
            UpdateChannelId::Application => {
                let mut channels = self.channels.lock().expect("updates lock");
                let channel = channels
                    .iter_mut()
                    .find(|c| c.id == id)
                    .ok_or_else(|| AppError::from("Update channel not found"))?;
                channel.status = UpdateChannelStatus::UpToDate;
                channel.current_version = channel.latest_version.clone();
                channel.download_progress = None;
                channel.last_checked_at = Some(now_iso());
                Ok((channel.clone(), None))
            }
        }
    }

    fn update_flowseal_pack(
        &self,
        id: UpdateChannelId,
    ) -> AppResult<(UpdateChannel, Option<EngineLayout>)> {
        {
            let mut channels = self.channels.lock().expect("updates lock");
            let channel = channels
                .iter_mut()
                .find(|c| c.id == id)
                .ok_or_else(|| AppError::from("Update channel not found"))?;
            if channel.status == UpdateChannelStatus::UpToDate {
                return Ok((channel.clone(), None));
            }
            channel.status = UpdateChannelStatus::Downloading;
            channel.download_progress = Some(0);
        }

        let release = fetch_latest_release()?;
        let asset = release
            .preferred_zip_asset()
            .ok_or_else(|| AppError::from("No zip asset in latest Flowseal release"))?
            .clone();
        let version = release.version().to_string();

        let downloads = self.app_data.join("downloads");
        fs::create_dir_all(&downloads).map_err(|e| AppError::from(e.to_string()))?;
        let archive_path = downloads.join(&asset.name);

        download_with_progress(&asset.browser_download_url, &archive_path, |pct| {
            let mut channels = self.channels.lock().expect("updates lock");
            if let Some(channel) = channels.iter_mut().find(|c| c.id == id) {
                channel.download_progress = Some(pct);
                channel.status = UpdateChannelStatus::Downloading;
            }
        })?;

        {
            let mut channels = self.channels.lock().expect("updates lock");
            if let Some(channel) = channels.iter_mut().find(|c| c.id == id) {
                channel.status = UpdateChannelStatus::Installing;
                channel.download_progress = Some(100);
            }
        }

        let dest = self.app_data.join("engine").join(&version);
        if dest.exists() {
            fs::remove_dir_all(&dest).map_err(|e| AppError::from(e.to_string()))?;
        }
        fs::create_dir_all(&dest).map_err(|e| AppError::from(e.to_string()))?;

        extract_zip(&archive_path, &dest)?;

        
        let layout_root = normalize_extracted_pack(&dest)?;
        let layout = EngineLayout::from_pack(layout_root);

        
        let current = self.app_data.join("engine").join("current");
        let _ = fs::remove_dir_all(&current);
        let _ = fs::remove_file(&current);
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("cmd")
                .args([
                    "/C",
                    "mklink",
                    "/J",
                    current.to_str().unwrap_or_default(),
                    layout.root.to_str().unwrap_or_default(),
                ])
                .status();
        }
        #[cfg(not(windows))]
        {
            let _ = std::os::unix::fs::symlink(&layout.root, &current);
        }

        let mut channels = self.channels.lock().expect("updates lock");
        for channel in channels.iter_mut() {
            if matches!(
                channel.id,
                UpdateChannelId::Engine | UpdateChannelId::Strategies
            ) {
                channel.status = UpdateChannelStatus::UpToDate;
                channel.current_version = version.clone();
                channel.latest_version = version.clone();
                channel.download_progress = None;
                channel.last_checked_at = Some(now_iso());
            }
        }

        let channel = channels
            .iter()
            .find(|c| c.id == id)
            .cloned()
            .ok_or_else(|| AppError::from("Update channel not found"))?;

        Ok((channel, Some(layout)))
    }

    #[allow(dead_code)]
    pub fn set_engine_version(&self, version: &str) {
        let mut channels = self.channels.lock().expect("updates lock");
        for channel in channels.iter_mut() {
            if matches!(
                channel.id,
                UpdateChannelId::Engine | UpdateChannelId::Strategies
            ) {
                channel.current_version = version.to_string();
            }
        }
    }
}

fn download_with_progress(
    url: &str,
    dest: &Path,
    mut on_progress: impl FnMut(u32),
) -> AppResult<()> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("zapp/0.2")
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|e| AppError::from(e.to_string()))?;

    let mut response = client
        .get(url)
        .send()
        .map_err(|e| AppError::from(format!("Download failed: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::from(format!(
            "Download HTTP {}",
            response.status()
        )));
    }

    let total = response.content_length().unwrap_or(0);
    let mut file = File::create(dest).map_err(|e| AppError::from(e.to_string()))?;
    let mut downloaded: u64 = 0;
    let mut buffer = [0u8; 64 * 1024];

    loop {
        let n = {
            use std::io::Read;
            response
                .read(&mut buffer)
                .map_err(|e| AppError::from(e.to_string()))?
        };
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n])
            .map_err(|e| AppError::from(e.to_string()))?;
        downloaded += n as u64;
        if total > 0 {
            on_progress(((downloaded * 100) / total).min(99) as u32);
        } else {
            on_progress(((downloaded / (256 * 1024)) % 90 + 5) as u32);
        }
    }

    on_progress(100);
    Ok(())
}

fn extract_zip(archive: &Path, dest: &Path) -> AppResult<()> {
    let file = File::open(archive).map_err(|e| AppError::from(e.to_string()))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| AppError::from(e.to_string()))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| AppError::from(e.to_string()))?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|e| AppError::from(e.to_string()))?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent).map_err(|e| AppError::from(e.to_string()))?;
            }
            let mut outfile = File::create(&outpath).map_err(|e| AppError::from(e.to_string()))?;
            copy(&mut file, &mut outfile).map_err(|e| AppError::from(e.to_string()))?;
        }
    }

    
    thread::sleep(Duration::from_millis(200));
    Ok(())
}

fn normalize_extracted_pack(dest: &Path) -> AppResult<PathBuf> {
    if dest.join("bin").join("winws.exe").is_file() {
        return Ok(dest.to_path_buf());
    }

    let entries: Vec<_> = fs::read_dir(dest)
        .map_err(|e| AppError::from(e.to_string()))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();

    for entry in entries {
        if entry.join("bin").join("winws.exe").is_file() {
            return Ok(entry);
        }
    }

    Err(AppError::from(
        "Extracted archive does not contain bin/winws.exe",
    ))
}
