// ╔════════════════════════════════════════════════════╗
// ║                   helper/paths                   ║
// ╚════════════════════════════════════════════════════╝

use std::path::{Component, Path, PathBuf};


pub const APP_DATA_ID: &str = "app.zapp.desktop";

pub const PROGRAM_DATA_ID: &str = "zapp";

pub fn managed_engine_prefix() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("USERPROFILE")
                .map(|u| PathBuf::from(u).join("AppData").join("Roaming"))
                .unwrap_or_else(std::env::temp_dir)
        });
    base.join(APP_DATA_ID).join("engine")
}

pub fn program_data_engine_prefix() -> PathBuf {
    let base = std::env::var_os("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
    base.join(PROGRAM_DATA_ID).join("engine")
}


pub fn managed_engine_prefixes() -> Vec<PathBuf> {
    vec![program_data_engine_prefix(), managed_engine_prefix()]
}


pub fn validate_winws_paths_any(
    winws: &Path,
    work_dir: &Path,
) -> Result<(PathBuf, PathBuf), String> {
    let mut last = String::from("refused: no managed engine prefix available");
    for prefix in managed_engine_prefixes() {
        match validate_winws_paths(winws, work_dir, &prefix) {
            Ok(ok) => return Ok(ok),
            Err(e) => last = e,
        }
    }
    Err(last)
}












pub fn validate_winws_paths(
    winws: &Path,
    work_dir: &Path,
    allowed_engine_prefix: &Path,
) -> Result<(PathBuf, PathBuf), String> {
    reject_dotdot(winws, "winws")?;
    reject_dotdot(work_dir, "work_dir")?;

    let winws_name = winws
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if !winws_name.eq_ignore_ascii_case("winws.exe") {
        return Err(format!(
            "refused: executable must be winws.exe, got '{winws_name}'"
        ));
    }

    let winws_abs = canonicalize_existing(winws, "winws")?;
    let work_abs = canonicalize_existing(work_dir, "work_dir")?;
    let prefix_abs = canonicalize_or_lex(allowed_engine_prefix)?;

    let bin_dir = winws_abs
        .parent()
        .ok_or_else(|| "refused: winws has no parent directory".to_string())?;
    let bin_name = bin_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if !bin_name.eq_ignore_ascii_case("bin") {
        return Err(format!(
            "refused: winws must live in a 'bin' folder, got '{}'",
            bin_dir.display()
        ));
    }

    let engine_root = bin_dir
        .parent()
        .ok_or_else(|| "refused: engine root missing".to_string())?;


    let cwd_abs = if path_eq(&work_abs, bin_dir) {
        bin_dir.to_path_buf()
    } else if path_eq(&work_abs, engine_root) {
        bin_dir.to_path_buf()
    } else {
        return Err(format!(
            "refused: work_dir must be pack root or bin (pack {}, bin {}, got {})",
            engine_root.display(),
            bin_dir.display(),
            work_abs.display()
        ));
    };

    let engine_parent = engine_root
        .parent()
        .ok_or_else(|| "refused: engine parent missing".to_string())?;
    if !path_eq(engine_parent, &prefix_abs) {
        return Err(format!(
            "refused: engine root must be under {} (got {})",
            prefix_abs.display(),
            engine_root.display()
        ));
    }


    let ver = engine_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if ver.is_empty() || ver == "." || ver == ".." || ver.contains(['/', '\\']) {
        return Err("refused: invalid engine version directory".into());
    }

    Ok((winws_abs, cwd_abs))
}


pub fn managed_app_data_root() -> PathBuf {
    managed_engine_prefix()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(managed_engine_prefix)
}


pub fn program_data_zapp_root() -> PathBuf {
    program_data_engine_prefix()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(program_data_engine_prefix)
}



pub fn apply_program_data_acl(zapp_root: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::process::Command;
        let path = zapp_root.to_string_lossy();
        let status = Command::new("icacls")
            .args([
                path.as_ref(),
                "/inheritance:r",
                "/grant:r",
                "*S-1-5-18:(OI)(CI)F",
                "/grant:r",
                "*S-1-5-32-544:(OI)(CI)F",
                "/grant:r",
                "*S-1-5-32-545:(OI)(CI)RX",
            ])
            .status()
            .map_err(|e| format!("icacls spawn failed: {e}"))?;
        if !status.success() {
            return Err(format!("icacls exited with {status}"));
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = zapp_root;
        Ok(())
    }
}

fn reject_dotdot(path: &Path, label: &str) -> Result<(), String> {
    for c in path.components() {
        if matches!(c, Component::ParentDir) {
            return Err(format!("refused: {label} must not contain '..'"));
        }
    }
    Ok(())
}

fn canonicalize_existing(path: &Path, label: &str) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|e| format!("refused: cannot resolve {label} '{}': {e}", path.display()))
}

fn canonicalize_or_lex(path: &Path) -> Result<PathBuf, String> {
    match path.canonicalize() {
        Ok(p) => Ok(p),
        Err(_) => {

            Ok(normalize_lexically(path))
        }
    }
}

fn normalize_lexically(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn path_eq(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }

    let a = a.to_string_lossy();
    let b = b.to_string_lossy();
    a.eq_ignore_ascii_case(&b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn scratch() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("zapp-path-test-{nanos}"))
    }

    #[test]
    fn accepts_managed_layout_with_bin_cwd() {
        let root = scratch();
        let prefix = root.join("engine");
        let pack = prefix.join("1.10.4");
        let bin = pack.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let winws = bin.join("winws.exe");
        fs::write(&winws, b"fake").unwrap();


        let (w, d) = validate_winws_paths(&winws, &bin, &prefix).unwrap();
        assert!(w.ends_with("winws.exe"));
        assert!(path_eq(&d, &bin.canonicalize().unwrap()));


        let (w2, d2) = validate_winws_paths(&winws, &pack, &prefix).unwrap();
        assert!(w2.ends_with("winws.exe"));
        assert!(path_eq(&d2, &bin.canonicalize().unwrap()));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rejects_evil_exe_name() {
        let root = scratch();
        let prefix = root.join("engine");
        let pack = prefix.join("1.10.4");
        let bin = pack.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let evil = bin.join("evil.exe");
        fs::write(&evil, b"x").unwrap();

        let err = validate_winws_paths(&evil, &pack, &prefix).unwrap_err();
        assert!(err.contains("winws.exe"), "{err}");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rejects_outside_prefix() {
        let root = scratch();
        let prefix = root.join("engine");
        let other = root.join("other").join("1.0.0");
        let bin = other.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let winws = bin.join("winws.exe");
        fs::write(&winws, b"x").unwrap();
        fs::create_dir_all(&prefix).unwrap();

        let err = validate_winws_paths(&winws, &other, &prefix).unwrap_err();
        assert!(err.contains("under"), "{err}");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rejects_dotdot_in_input() {
        let err = validate_winws_paths(
            Path::new(r"C:\app\engine\1.0\bin\..\..\evil\winws.exe"),
            Path::new(r"C:\app\engine\1.0"),
            Path::new(r"C:\app\engine"),
        )
        .unwrap_err();
        assert!(err.contains(".."), "{err}");
    }
}
