use std::process::Command;

pub fn in_repo() -> bool {
    Command::new("jj")
        .args(["root"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn log() -> String {
    Command::new("jj")
        .args(["log", "--limit", "10", "--color=always"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

pub fn status() -> String {
    Command::new("jj")
        .args(["status", "--color=always"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default()
}

pub fn bookmark_names() -> Vec<String> {
    Command::new("jj")
        .args(["bookmark", "list", "--color=never"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter_map(|l| l.split(':').next())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}
