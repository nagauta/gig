use std::path::{Path, PathBuf};

use crate::spec::Spec;

/// Load all specs from a directory.
pub fn load_specs(dir: &Path) -> Vec<Spec> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
        .filter_map(|e| Spec::from_file(&e.path()).ok())
        .collect()
}

/// Find the spec matching the command name.
pub fn find_spec<'a>(specs: &'a [Spec], command: &str) -> Option<&'a Spec> {
    specs.iter().find(|s| s.name == command)
}

/// Generate completion output (one completion per line: value\tdescription).
pub fn generate_completions(spec: &Spec, args: &[&str]) -> String {
    spec.completions(args)
        .into_iter()
        .map(|c| {
            if let Some(desc) = &c.description {
                format!("{}\t{}", c.value, desc)
            } else {
                c.value
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Default specs directory (relative to executable or config).
pub fn default_specs_dir() -> PathBuf {
    // Check XDG_CONFIG_HOME or ~/.config/gig/specs
    if let Ok(config) = std::env::var("GIG_SPECS_DIR") {
        return PathBuf::from(config);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("gig").join("specs")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_specs_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("git.toml"),
            r#"
name = "git"
description = "Version control"

[[subcommands]]
name = "commit"
description = "Record changes"

[[subcommands.options]]
name = "--message"
description = "Commit message"
short = "-m"

[[subcommands]]
name = "push"
description = "Update remote"
"#,
        )
        .unwrap();

        fs::write(
            dir.path().join("docker.toml"),
            r#"
name = "docker"
description = "Container runtime"

[[subcommands]]
name = "run"
description = "Run a container"

[[subcommands]]
name = "build"
description = "Build an image"
"#,
        )
        .unwrap();

        // Non-toml file should be ignored
        fs::write(dir.path().join("readme.txt"), "ignore me").unwrap();

        dir
    }

    #[test]
    fn load_specs_from_directory() {
        let dir = setup_test_specs_dir();
        let specs = load_specs(dir.path());
        assert_eq!(specs.len(), 2);
        let names: Vec<&str> = specs.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"git"));
        assert!(names.contains(&"docker"));
    }

    #[test]
    fn load_specs_ignores_non_toml() {
        let dir = setup_test_specs_dir();
        let specs = load_specs(dir.path());
        // Only git.toml and docker.toml
        assert_eq!(specs.len(), 2);
    }

    #[test]
    fn load_specs_nonexistent_dir_returns_empty() {
        let specs = load_specs(Path::new("/nonexistent/path"));
        assert!(specs.is_empty());
    }

    #[test]
    fn find_spec_by_name() {
        let dir = setup_test_specs_dir();
        let specs = load_specs(dir.path());
        assert!(find_spec(&specs, "git").is_some());
        assert!(find_spec(&specs, "docker").is_some());
        assert!(find_spec(&specs, "npm").is_none());
    }

    #[test]
    fn generate_completions_with_descriptions() {
        let dir = setup_test_specs_dir();
        let specs = load_specs(dir.path());
        let git = find_spec(&specs, "git").unwrap();
        let output = generate_completions(git, &[""]);
        assert!(output.contains("commit\tRecord changes"));
        assert!(output.contains("push\tUpdate remote"));
    }

    #[test]
    fn generate_completions_subcommand_options() {
        let dir = setup_test_specs_dir();
        let specs = load_specs(dir.path());
        let git = find_spec(&specs, "git").unwrap();
        let output = generate_completions(git, &["commit", ""]);
        assert!(output.contains("--message\tCommit message"));
        assert!(output.contains("-m\tCommit message"));
    }

    #[test]
    fn generate_completions_no_match() {
        let dir = setup_test_specs_dir();
        let specs = load_specs(dir.path());
        let git = find_spec(&specs, "git").unwrap();
        let output = generate_completions(git, &["xyz"]);
        assert!(output.is_empty());
    }
}
