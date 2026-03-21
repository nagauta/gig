use std::fs;
use std::path::{Path, PathBuf};

const HOOK_MARKER_BEGIN: &str = "# >>> gig >>>";
const HOOK_MARKER_END: &str = "# <<< gig <<<";

fn hook_block(shell: &str) -> String {
    format!(
        "{}\neval \"$(gig init {})\"\n{}",
        HOOK_MARKER_BEGIN, shell, HOOK_MARKER_END
    )
}

/// Return the shell rc file path for the given shell.
pub fn rc_path(shell: &str) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let file = match shell {
        "zsh" => ".zshrc",
        "bash" => ".bashrc",
        _ => return None,
    };
    Some(PathBuf::from(home).join(file))
}

/// Check if the hook is already installed in the given file content.
pub fn is_installed(content: &str) -> bool {
    content.contains(HOOK_MARKER_BEGIN)
}

/// Add the gig hook to a shell config file. Returns true if modified.
pub fn install(rc_file: &Path, shell: &str) -> Result<bool, std::io::Error> {
    let content = if rc_file.exists() {
        fs::read_to_string(rc_file)?
    } else {
        String::new()
    };

    if is_installed(&content) {
        return Ok(false);
    }

    let block = hook_block(shell);
    let new_content = if content.is_empty() {
        format!("{}\n", block)
    } else {
        format!("{}\n\n{}\n", content.trim_end(), block)
    };

    fs::write(rc_file, new_content)?;
    Ok(true)
}

/// Remove the gig hook from a shell config file. Returns true if modified.
pub fn uninstall(rc_file: &Path) -> Result<bool, std::io::Error> {
    if !rc_file.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(rc_file)?;
    if !is_installed(&content) {
        return Ok(false);
    }

    let mut lines: Vec<&str> = content.lines().collect();
    let begin = lines.iter().position(|l| l.contains(HOOK_MARKER_BEGIN));
    let end = lines.iter().position(|l| l.contains(HOOK_MARKER_END));

    if let (Some(b), Some(e)) = (begin, end) {
        lines.drain(b..=e);
        // Remove trailing blank line left behind
        while lines.last().is_some_and(|l| l.is_empty()) {
            lines.pop();
        }
        let new_content = if lines.is_empty() {
            String::new()
        } else {
            format!("{}\n", lines.join("\n"))
        };
        fs::write(rc_file, new_content)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Copy bundled specs to the user config directory.
pub fn install_specs(bundled_dir: &Path, target_dir: &Path) -> Result<u32, std::io::Error> {
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
    }

    let mut count = 0;
    for entry in fs::read_dir(bundled_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            let dest = target_dir.join(entry.file_name());
            fs::copy(&path, &dest)?;
            count += 1;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn temp_rc(dir: &TempDir, name: &str, content: &str) -> PathBuf {
        let path = dir.path().join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn install_adds_hook_to_empty_file() {
        let dir = TempDir::new().unwrap();
        let rc = dir.path().join(".zshrc");
        let modified = install(&rc, "zsh").unwrap();
        assert!(modified);
        let content = fs::read_to_string(&rc).unwrap();
        assert!(content.contains(HOOK_MARKER_BEGIN));
        assert!(content.contains("gig init zsh"));
        assert!(content.contains(HOOK_MARKER_END));
    }

    #[test]
    fn install_appends_to_existing_file() {
        let dir = TempDir::new().unwrap();
        let rc = temp_rc(&dir, ".zshrc", "export PATH=/usr/bin\n");
        let modified = install(&rc, "zsh").unwrap();
        assert!(modified);
        let content = fs::read_to_string(&rc).unwrap();
        assert!(content.starts_with("export PATH=/usr/bin"));
        assert!(content.contains(HOOK_MARKER_BEGIN));
    }

    #[test]
    fn install_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let rc = dir.path().join(".bashrc");
        install(&rc, "bash").unwrap();
        let modified = install(&rc, "bash").unwrap();
        assert!(!modified);
    }

    #[test]
    fn uninstall_removes_hook() {
        let dir = TempDir::new().unwrap();
        let rc = dir.path().join(".zshrc");
        install(&rc, "zsh").unwrap();
        let modified = uninstall(&rc).unwrap();
        assert!(modified);
        let content = fs::read_to_string(&rc).unwrap();
        assert!(!content.contains(HOOK_MARKER_BEGIN));
        assert!(!content.contains("gig init"));
    }

    #[test]
    fn uninstall_preserves_other_content() {
        let dir = TempDir::new().unwrap();
        let rc = temp_rc(&dir, ".zshrc", "export FOO=bar\n");
        install(&rc, "zsh").unwrap();
        uninstall(&rc).unwrap();
        let content = fs::read_to_string(&rc).unwrap();
        assert!(content.contains("export FOO=bar"));
        assert!(!content.contains(HOOK_MARKER_BEGIN));
    }

    #[test]
    fn uninstall_noop_when_not_installed() {
        let dir = TempDir::new().unwrap();
        let rc = temp_rc(&dir, ".bashrc", "# just a comment\n");
        let modified = uninstall(&rc).unwrap();
        assert!(!modified);
    }

    #[test]
    fn uninstall_noop_when_file_missing() {
        let dir = TempDir::new().unwrap();
        let rc = dir.path().join(".zshrc");
        let modified = uninstall(&rc).unwrap();
        assert!(!modified);
    }

    #[test]
    fn is_installed_detects_marker() {
        assert!(is_installed("stuff\n# >>> gig >>>\neval\n# <<< gig <<<\n"));
        assert!(!is_installed("just normal shell config\n"));
    }

    #[test]
    fn install_specs_copies_toml_files() {
        let src = TempDir::new().unwrap();
        fs::write(src.path().join("git.toml"), "name = \"git\"").unwrap();
        fs::write(src.path().join("readme.md"), "ignore").unwrap();

        let dst = TempDir::new().unwrap();
        let target = dst.path().join("specs");
        let count = install_specs(src.path(), &target).unwrap();
        assert_eq!(count, 1);
        assert!(target.join("git.toml").exists());
        assert!(!target.join("readme.md").exists());
    }

    #[test]
    fn install_specs_creates_target_dir() {
        let src = TempDir::new().unwrap();
        fs::write(src.path().join("a.toml"), "name = \"a\"").unwrap();

        let dst = TempDir::new().unwrap();
        let target = dst.path().join("deep").join("nested").join("specs");
        let count = install_specs(src.path(), &target).unwrap();
        assert_eq!(count, 1);
    }
}
