// ╔════════════════════════════════════════════════════╗
// ║                  engine/updates                  ║
// ╚════════════════════════════════════════════════════╝

use std::fs::{self, File};
use std::io::{copy, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::engine::github::{
    compare_semver, fetch_flowseal_latest, fetch_zapp_latest, FLOWSEAL_REPO, ZAPP_REPO,
};
use crate::engine::managed::EngineLayout;
use crate::error::{AppError, AppResult};
use crate::model::{UpdateChannel, UpdateChannelId, UpdateChannelStatus};
use crate::util::now_iso;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub struct UpdateOutcome {
    pub channel: UpdateChannel,
    pub layout: Option<EngineLayout>,
    /// Application installer launched — UI should quit so files can be replaced.
    pub restart_required: bool,
}

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
        let flowseal = fetch_flowseal_latest().ok();
        let zapp = fetch_zapp_latest().ok();
        let mut channels = self.channels.lock().expect("updates lock");
        let now = now_iso();

        for channel in channels.iter_mut() {
            channel.last_checked_at = Some(now.clone());
            match channel.id {
                UpdateChannelId::Engine | UpdateChannelId::Strategies => {
                    if let Some(ref rel) = flowseal {
                        let latest = rel.version().to_string();
                        channel.latest_version = latest.clone();
                        channel.release_notes = rel.body.clone().unwrap_or_else(|| {
                            format!("Release {latest} from {FLOWSEAL_REPO}")
                        });
                        let missing = channel.current_version == "0.0.0"
                            || channel.current_version == "unknown";
                        channel.status = if missing
                            || compare_semver(&latest, &channel.current_version).is_gt()
                        {
                            UpdateChannelStatus::Available
                        } else {
                            UpdateChannelStatus::UpToDate
                        };
                    }
                }
                UpdateChannelId::Application => {
                    if let Some(ref rel) = zapp {
                        let latest = rel.version().to_string();
                        channel.latest_version = latest.clone();
                        channel.release_notes = rel
                            .body
                            .clone()
                            .unwrap_or_else(|| format!("Release {latest} from {ZAPP_REPO}"));
                        channel.status =
                            if compare_semver(&latest, &channel.current_version).is_gt() {
                                UpdateChannelStatus::Available
                            } else {
                                UpdateChannelStatus::UpToDate
                            };
                    }
                }
            }
        }

        Ok(channels.clone())
    }

    pub fn update_channel(&self, id: UpdateChannelId) -> AppResult<UpdateOutcome> {
        match id {
            UpdateChannelId::Engine | UpdateChannelId::Strategies => {
                self.update_flowseal_pack(id)
            }
            UpdateChannelId::Application => self.update_application(),
        }
    }

    /// First-run / repair: download Flowseal pack even if channel looks up-to-date.
    pub fn ensure_engine_installed(
        &self,
        current: &EngineLayout,
    ) -> AppResult<Option<EngineLayout>> {
        let _ = crate::engine::pack::ensure_flowseal_sidecars(&current.root);
        if current.is_installed() && crate::engine::pack::pack_payloads_present(&current.root) {
            return Ok(None);
        }

        log::warn!(
            "engine pack incomplete at {} — forcing Flowseal install",
            current.root.display()
        );

        {
            let mut channels = self.channels.lock().expect("updates lock");
            if let Some(channel) = channels
                .iter_mut()
                .find(|c| c.id == UpdateChannelId::Engine)
            {
                channel.status = UpdateChannelStatus::Available;
                if channel.current_version == "0.0.0" || channel.current_version == "unknown" {
                    channel.latest_version = "pending".into();
                }
            }
        }

        let outcome = self.update_flowseal_pack(UpdateChannelId::Engine)?;
        Ok(outcome.layout)
    }

    /// Install every Available channel. Engine pack once; Application last.
    pub fn auto_install_available(&self) -> AppResult<Vec<UpdateOutcome>> {
        let pending: Vec<UpdateChannelId> = self
            .list()
            .into_iter()
            .filter(|c| c.status == UpdateChannelStatus::Available)
            .map(|c| c.id)
            .collect();

        let mut outcomes = Vec::new();
        let mut engine_done = false;

        for id in pending {
            match id {
                UpdateChannelId::Engine | UpdateChannelId::Strategies => {
                    if engine_done {
                        continue;
                    }
                    outcomes.push(self.update_channel(UpdateChannelId::Engine)?);
                    engine_done = true;
                }
                UpdateChannelId::Application => {
                    outcomes.push(self.update_channel(UpdateChannelId::Application)?);
                }
            }
        }

        Ok(outcomes)
    }

    fn update_flowseal_pack(&self, id: UpdateChannelId) -> AppResult<UpdateOutcome> {
        {
            let mut channels = self.channels.lock().expect("updates lock");
            let channel = channels
                .iter_mut()
                .find(|c| c.id == id)
                .ok_or_else(|| AppError::from("Update channel not found"))?;
            if channel.status == UpdateChannelStatus::UpToDate {
                return Ok(UpdateOutcome {
                    channel: channel.clone(),
                    layout: None,
                    restart_required: false,
                });
            }
            channel.status = UpdateChannelStatus::Downloading;
            channel.download_progress = Some(0);
        }

        let release = fetch_flowseal_latest()?;
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

        let nested = normalize_extracted_pack(&dest)?;
        let layout_root = flatten_pack_to_version_dir(&dest, &nested)?;
        crate::engine::pack::ensure_flowseal_sidecars(&layout_root)?;
        if !crate::engine::pack::pack_payloads_present(&layout_root) {
            return Err(AppError::from(
                "Engine pack installed but bin/*.bin payloads are missing — re-download Engine",
            ));
        }
        let layout = EngineLayout::from_pack(layout_root);
        link_engine_current(&self.app_data, &layout.root)?;

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

        Ok(UpdateOutcome {
            channel,
            layout: Some(layout),
            restart_required: false,
        })
    }

    fn update_application(&self) -> AppResult<UpdateOutcome> {
        {
            let mut channels = self.channels.lock().expect("updates lock");
            let channel = channels
                .iter_mut()
                .find(|c| c.id == UpdateChannelId::Application)
                .ok_or_else(|| AppError::from("Update channel not found"))?;
            if channel.status == UpdateChannelStatus::UpToDate {
                return Ok(UpdateOutcome {
                    channel: channel.clone(),
                    layout: None,
                    restart_required: false,
                });
            }
            channel.status = UpdateChannelStatus::Downloading;
            channel.download_progress = Some(0);
        }

        let release = fetch_zapp_latest()?;
        let asset = release
            .preferred_app_asset()
            .ok_or_else(|| {
                AppError::from(
                    "No installer asset in krwg/zapp release (.msi / .exe / .zip)",
                )
            })?
            .clone();
        let version = release.version().to_string();

        let downloads = self.app_data.join("downloads");
        fs::create_dir_all(&downloads).map_err(|e| AppError::from(e.to_string()))?;
        let dest = downloads.join(&asset.name);

        download_with_progress(&asset.browser_download_url, &dest, |pct| {
            let mut channels = self.channels.lock().expect("updates lock");
            if let Some(channel) = channels
                .iter_mut()
                .find(|c| c.id == UpdateChannelId::Application)
            {
                channel.download_progress = Some(pct);
                channel.status = UpdateChannelStatus::Downloading;
            }
        })?;

        {
            let mut channels = self.channels.lock().expect("updates lock");
            if let Some(channel) = channels
                .iter_mut()
                .find(|c| c.id == UpdateChannelId::Application)
            {
                channel.status = UpdateChannelStatus::Installing;
                channel.download_progress = Some(100);
            }
        }

        if let Err(err) = launch_app_installer(&dest) {
            let mut channels = self.channels.lock().expect("updates lock");
            if let Some(channel) = channels
                .iter_mut()
                .find(|c| c.id == UpdateChannelId::Application)
            {
                channel.status = UpdateChannelStatus::Available;
                channel.download_progress = None;
            }
            return Err(err);
        }

        let mut channels = self.channels.lock().expect("updates lock");
        let channel = channels
            .iter_mut()
            .find(|c| c.id == UpdateChannelId::Application)
            .ok_or_else(|| AppError::from("Update channel not found"))?;
        // Version flips after installer finishes; keep latest as target until restart.
        channel.latest_version = version.clone();
        channel.release_notes = format!(
            "Installer launched ({}). Confirm UAC / finish setup, then reopen zapp.",
            asset.name
        );
        channel.download_progress = None;
        channel.last_checked_at = Some(now_iso());
        channel.status = UpdateChannelStatus::Available;

        Ok(UpdateOutcome {
            channel: channel.clone(),
            layout: None,
            restart_required: true,
        })
    }
}

fn link_engine_current(app_data: &Path, root: &Path) -> AppResult<()> {
    let current = app_data.join("engine").join("current");
    let _ = fs::remove_dir_all(&current);
    let _ = fs::remove_file(&current);
    #[cfg(windows)]
    {
        let status = Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                current.to_str().unwrap_or_default(),
                root.to_str().unwrap_or_default(),
            ])
            .status()
            .map_err(|e| AppError::from(e.to_string()))?;
        if !status.success() {
            // Fallback: copy tree when junction fails (no admin).
            copy_dir_recursive(root, &current)?;
        }
    }
    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(root, &current).map_err(|e| AppError::from(e.to_string()))?;
    }
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> AppResult<()> {
    fs::create_dir_all(dst).map_err(|e| AppError::from(e.to_string()))?;
    for entry in fs::read_dir(src).map_err(|e| AppError::from(e.to_string()))? {
        let entry = entry.map_err(|e| AppError::from(e.to_string()))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| AppError::from(e.to_string()))?;
        }
    }
    Ok(())
}

/// Ask Windows for administrator rights. `Command::spawn` cannot do this and
/// returns os error 740 for an installer marked requireAdministrator.
#[cfg(windows)]
fn launch_elevated(file: &Path, args: &[String]) -> AppResult<()> {
    let quote = |s: &str| format!("'{}'", s.replace('\'', "''"));
    let file_arg = quote(&file.to_string_lossy());
    let arg_list = if args.is_empty() {
        String::new()
    } else {
        let parts: Vec<String> = args.iter().map(|a| quote(a)).collect();
        format!(" -ArgumentList @({})", parts.join(","))
    };
    let script = format!(
        "$ErrorActionPreference='Stop'; try {{ Start-Process -FilePath {file_arg}{arg_list} -Verb RunAs; exit 0 }} catch {{ if ($_.Exception.Message -match 'canceled|cancelled|1223') {{ exit 1223 }}; Write-Error $_; exit 1 }}"
    );

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| AppError::from(format!("Failed to launch installer: {e}")))?;

    let code = output.status.code().unwrap_or(1);
    if code == 0 {
        return Ok(());
    }
    if code == 1223 {
        return Err(AppError::from(
            "Installer was not started: the UAC prompt was declined.",
        ));
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(AppError::from(format!(
        "Failed to launch installer: {stderr}"
    )))
}

fn launch_app_installer(path: &Path) -> AppResult<()> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    #[cfg(windows)]
    {
        if name.ends_with(".msi") {
            let msiexec = std::env::var("SystemRoot")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from(r"C:\Windows"))
                .join("System32")
                .join("msiexec.exe");
            return launch_elevated(
                &msiexec,
                &[
                    "/i".into(),
                    path.to_string_lossy().into_owned(),
                    "/qb".into(),
                ],
            );
        }

        if name.ends_with(".exe") {
            return launch_elevated(path, &[]);
        }

        if name.ends_with(".zip") {
            let staging = path
                .parent()
                .unwrap_or(Path::new("."))
                .join("zapp-app-update");
            if staging.exists() {
                let _ = fs::remove_dir_all(&staging);
            }
            fs::create_dir_all(&staging).map_err(|e| AppError::from(e.to_string()))?;
            extract_zip(path, &staging)?;

            let _exe = find_file(&staging, "zapp.exe")
                .ok_or_else(|| AppError::from("zapp.exe not found in release zip"))?;
            let current = std::env::current_exe()
                .map_err(|e| AppError::from(e.to_string()))?;
            let target_dir = current
                .parent()
                .ok_or_else(|| AppError::from("Cannot resolve install directory"))?
                .to_path_buf();

            let script = path
                .parent()
                .unwrap_or(Path::new("."))
                .join("apply-zapp-update.cmd");
            let body = format!(
                "@echo off\r\n\
                 timeout /t 2 /nobreak >nul\r\n\
                 xcopy /y /e /i \"{src}\\*\" \"{dst}\\\"\r\n\
                 start \"\" \"{dst}\\zapp.exe\"\r\n\
                 del \"%~f0\"\r\n",
                src = staging.display(),
                dst = target_dir.display(),
            );
            fs::write(&script, body).map_err(|e| AppError::from(e.to_string()))?;

            return launch_elevated(&script, &[]);
        }
    }

    #[cfg(not(windows))]
    {
        let _ = path;
        return Err(AppError::from("Application auto-update is Windows-only for now"));
    }

    Err(AppError::from(format!(
        "Unsupported application asset: {name}"
    )))
}

fn find_file(root: &Path, name: &str) -> Option<PathBuf> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.eq_ignore_ascii_case(name))
            {
                return Some(path);
            }
        }
    }
    None
}

fn download_with_progress(
    url: &str,
    dest: &Path,
    mut on_progress: impl FnMut(u32),
) -> AppResult<()> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("zapp/0.2")
        .timeout(Duration::from_secs(300))
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

    thread::sleep(Duration::from_millis(100));
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

/// Move `engine/1.10.4/zapret-discord-youtube-1.10.4/*` → `engine/1.10.4/*`.
fn flatten_pack_to_version_dir(dest: &Path, nested: &Path) -> AppResult<PathBuf> {
    if nested == dest {
        return Ok(dest.to_path_buf());
    }

    let entries: Vec<_> = fs::read_dir(nested)
        .map_err(|e| AppError::from(e.to_string()))?
        .flatten()
        .map(|e| e.path())
        .collect();

    for entry in entries {
        let name = entry
            .file_name()
            .ok_or_else(|| AppError::from("Invalid pack entry"))?;
        let target = dest.join(name);
        if target.exists() {
            if target.is_dir() {
                let _ = fs::remove_dir_all(&target);
            } else {
                let _ = fs::remove_file(&target);
            }
        }
        if fs::rename(&entry, &target).is_err() {
            if entry.is_dir() {
                copy_dir_recursive(&entry, &target)?;
                let _ = fs::remove_dir_all(&entry);
            } else {
                fs::copy(&entry, &target).map_err(|e| AppError::from(e.to_string()))?;
                let _ = fs::remove_file(&entry);
            }
        }
    }

    let _ = fs::remove_dir_all(nested);

    if !dest.join("bin").join("winws.exe").is_file() {
        return Err(AppError::from(
            "Flattened engine pack is missing bin/winws.exe",
        ));
    }

    Ok(dest.to_path_buf())
}
