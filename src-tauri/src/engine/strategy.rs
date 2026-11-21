// ╔════════════════════════════════════════════════════╗
// ║                 engine/strategy                  ║
// ╚════════════════════════════════════════════════════╝

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct StrategyLaunch {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub bat_path: PathBuf,
    pub winws: PathBuf,
    pub args: Vec<String>,
    pub work_dir: PathBuf,
}

fn game_filter_vars(root: &Path) -> (String, String, String) {
    let flag = root.join("utils").join("game_filter.enabled");
    if !flag.is_file() {
        return ("12".into(), "12".into(), "12".into());
    }

    let Ok(text) = fs::read_to_string(&flag) else {
        return ("12".into(), "12".into(), "12".into());
    };

    let mut mode = String::from("disabled");
    let mut tcp_range = String::from("1024-65535");
    let mut udp_range = String::from("1024-65535");

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            match k.trim().to_ascii_lowercase().as_str() {
                "mode" | "all" => mode = v.trim().to_string(),
                "tcp" if !v.trim().is_empty() => tcp_range = v.trim().to_string(),
                "udp" if !v.trim().is_empty() => udp_range = v.trim().to_string(),
                "tcp" => mode = "tcp".into(),
                "udp" => mode = "udp".into(),
                _ => {}
            }
        } else if line.eq_ignore_ascii_case("all") {
            mode = "all".into();
        }
    }

    match mode.to_ascii_lowercase().as_str() {
        "all" => (tcp_range.clone(), tcp_range, udp_range),
        "tcp" => (tcp_range.clone(), tcp_range, "12".into()),
        "udp" => (udp_range.clone(), "12".into(), udp_range),
        _ => ("12".into(), "12".into(), "12".into()),
    }
}

fn expand_vars(raw: &str, root: &Path) -> String {
    let bin = root.join("bin");
    let lists = root.join("lists");
    let (gf, gf_tcp, gf_udp) = game_filter_vars(root);

    let mut s = raw.to_string();

    s = s.replace("^!", "!");
    s = s.replace("%GameFilterTCP%", &gf_tcp);
    s = s.replace("%GameFilterUDP%", &gf_udp);
    s = s.replace("%GameFilter%", &gf);
    s = s.replace("%BIN%", &format!("{}\\", bin.display()));
    s = s.replace("%LISTS%", &format!("{}\\", lists.display()));
    s = s.replace("%~dp0", &format!("{}\\", root.display()));
    s
}

fn tokenize(input: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => in_quotes = !in_quotes,
            ' ' | '\t' if !in_quotes => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn join_continuations(text: &str) -> String {
    let mut joined = String::new();
    for line in text.lines() {
        let trimmed_end = line.trim_end();
        if trimmed_end.ends_with('^') {
            joined.push_str(&trimmed_end[..trimmed_end.len() - 1]);
            joined.push(' ');
        } else {
            joined.push_str(trimmed_end);
            joined.push('\n');
        }
    }
    joined
}

pub fn parse_strategy_bat(root: &Path, strategy_name: &str) -> AppResult<StrategyLaunch> {
    let bat_path = root.join(format!("{strategy_name}.bat"));
    if !bat_path.is_file() {
        return Err(AppError::from(format!(
            "Strategy not found: {}",
            bat_path.display()
        )));
    }

    let text = fs::read_to_string(&bat_path)
        .map_err(|e| AppError::from(format!("Cannot read {}: {e}", bat_path.display())))?;
    let joined = join_continuations(&text);

    let mut capture = String::new();
    for line in joined.lines() {
        let lower = line.to_ascii_lowercase();
        if let Some(idx) = lower.find("winws.exe") {
            let after = &line[idx + "winws.exe".len()..];

            let after = after.strip_prefix('"').unwrap_or(after);
            capture.push_str(after);
            capture.push(' ');
        } else if !capture.is_empty() && line.trim_start().starts_with("--") {
            capture.push_str(line);
            capture.push(' ');
        }
    }

    if capture.trim().is_empty() {
        return Err(AppError::from(format!(
            "No winws.exe arguments found in {}",
            bat_path.display()
        )));
    }

    let root = dunce_canonicalize(root);
    let expanded = expand_vars(capture.trim(), &root);
    let args = tokenize(&expanded);
    let winws = root.join("bin").join("winws.exe");
    if !winws.is_file() {
        return Err(AppError::from(format!(
            "winws.exe not found at {}",
            winws.display()
        )));
    }

    Ok(StrategyLaunch {
        name: strategy_name.to_string(),
        bat_path: dunce_canonicalize(&bat_path),
        winws: dunce_canonicalize(&winws),
        args,
        work_dir: dunce_canonicalize(&root.join("bin")),
    })
}

fn dunce_canonicalize(path: &Path) -> PathBuf {
    path.canonicalize()
        .map(|p| {

            let s = p.to_string_lossy();
            if let Some(stripped) = s.strip_prefix(r"\\?\") {
                PathBuf::from(stripped)
            } else {
                p
            }
        })
        .unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_quotes() {
        let t = tokenize(r#"--hostlist="C:\lists\a.txt" --dpi-desync=fake"#);
        assert_eq!(t[0], "--hostlist=C:\\lists\\a.txt");
        assert_eq!(t[1], "--dpi-desync=fake");
    }

    #[test]
    fn expand_bang_escape() {
        let root = Path::new("C:\\zapret");
        let s = expand_vars("--dpi-desync-fake-tls=^!", root);
        assert_eq!(s, "--dpi-desync-fake-tls=!");
    }

    #[test]
    fn join_caret_continuations() {
        let text = "--a=1 ^\r\n--b=2";
        let j = join_continuations(text);
        assert!(j.contains("--a=1"));
        assert!(j.contains("--b=2"));
        assert!(!j.contains('^'));
    }

    #[test]
    fn parse_real_fake_tls_auto() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../vendor/zapret");
        if !root.join("bin").join("winws.exe").is_file() {
            return;
        }
        let launch = parse_strategy_bat(&root, "general (FAKE TLS AUTO)").expect("parse");
        assert!(
            launch.args.len() > 40,
            "too few args: {}",
            launch.args.len()
        );
        assert!(launch.args.iter().any(|a| a == "--dpi-desync-fake-tls=!"));
        assert!(launch.args.iter().any(|a| a.starts_with("--wf-tcp=")));
        assert!(launch.winws.is_absolute());
        assert!(!launch.winws.to_string_lossy().contains(".."));
        eprintln!("OK args={} winws={}", launch.args.len(), launch.winws.display());
    }
}
