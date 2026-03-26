use serde::Deserialize;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Template {
    Filepaths,
    Folders,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Spec {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub options: Vec<Opt>,
    #[serde(default)]
    pub subcommands: Vec<Subcommand>,
    pub template: Option<Template>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Opt {
    pub name: String,
    pub description: Option<String>,
    pub short: Option<String>,
    pub arg: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Subcommand {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub options: Vec<Opt>,
    pub generator: Option<String>,
    #[serde(default)]
    pub generator_kind: GeneratorKind,
    pub template: Option<Template>,
}

impl Spec {
    pub fn from_toml(input: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(input)
    }

    pub fn from_file(path: &Path) -> Result<Self, SpecError> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::from_toml(&content)?)
    }

    /// Return completions matching a partial input for the top-level command.
    pub fn completions(&self, args: &[&str]) -> Vec<Completion> {
        match args {
            [] | [""] => {
                // Show all subcommands and options
                let mut result: Vec<Completion> = self
                    .subcommands
                    .iter()
                    .map(|s| Completion {
                        value: s.name.clone(),
                        display_name: None,
                        description: s.description.clone(),
                        kind: CompletionKind::Subcommand,
                    })
                    .collect();
                result.extend(self.options.iter().flat_map(opt_completions));
                if let Some(template) = self.template {
                    let (_, candidates) = run_template(template, "");
                    result.extend(candidates);
                }
                result
            }
            [partial] if partial.starts_with('-') => {
                // Filter options by prefix
                self.options
                    .iter()
                    .flat_map(opt_completions)
                    .filter(|c| c.value.starts_with(partial))
                    .collect()
            }
            [partial] => {
                // Filter subcommands and options by prefix
                let mut result: Vec<Completion> = self
                    .subcommands
                    .iter()
                    .filter(|s| s.name.starts_with(partial))
                    .map(|s| Completion {
                        value: s.name.clone(),
                        display_name: None,
                        description: s.description.clone(),
                        kind: CompletionKind::Subcommand,
                    })
                    .collect();
                result.extend(
                    self.options
                        .iter()
                        .flat_map(opt_completions)
                        .filter(|c| c.value.starts_with(partial)),
                );
                if let Some(template) = self.template {
                    let (_, candidates) = run_template(template, partial);
                    result.extend(candidates);
                }
                result
            }
            [subcmd, rest @ ..] => {
                // Delegate to subcommand
                if let Some(sub) = self.subcommands.iter().find(|s| s.name == *subcmd) {
                    sub_completions(sub, rest)
                } else if let Some(template) = self.template {
                    // No matching subcommand, but spec has a template — join all tokens as a path
                    let mut path_parts: Vec<&str> = vec![subcmd];
                    path_parts.extend_from_slice(rest);
                    let combined = path_parts.join("");
                    let (_, candidates) = run_template(template, &combined);
                    candidates
                } else {
                    vec![]
                }
            }
        }
    }
}

fn opt_completions(opt: &Opt) -> Vec<Completion> {
    let mut result = vec![Completion {
        value: opt.name.clone(),
        display_name: None,
        description: opt.description.clone(),
        kind: CompletionKind::Option,
    }];
    if let Some(short) = &opt.short {
        result.push(Completion {
            value: short.clone(),
            display_name: None,
            description: opt.description.clone(),
            kind: CompletionKind::Option,
        });
    }
    result
}

fn run_generator(command: &str, generator_kind: GeneratorKind) -> Vec<Completion> {
    let kind = match generator_kind {
        GeneratorKind::Branch => CompletionKind::Branch,
        GeneratorKind::File => CompletionKind::File,
        GeneratorKind::Command => CompletionKind::Command,
    };
    let output = Command::new("sh").arg("-c").arg(command).output().ok();
    match output {
        Some(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| Completion {
                value: l.trim().to_string(),
                display_name: None,
                description: None,
                kind,
            })
            .collect(),
        _ => vec![],
    }
}

/// Run a template completion with an optional partial input.
/// If `partial` contains a `/` (e.g. "src/ma"), the directory part is used
/// as the base path and results are prefixed with it.
fn run_template(template: Template, partial: &str) -> (String, Vec<Completion>) {
    let (dir, filter) = if let Some(pos) = partial.rfind('/') {
        let dir_part = &partial[..=pos]; // e.g. "src/"
        let file_part = &partial[pos + 1..]; // e.g. "ma"
        (dir_part.to_string(), file_part.to_string())
    } else {
        (".".to_string(), partial.to_string())
    };

    let prefix = if dir == "." {
        String::new()
    } else {
        dir.clone()
    };

    let read_dir = match std::fs::read_dir(&dir) {
        Ok(rd) => rd,
        Err(_) => return (filter, vec![]),
    };

    let mut candidates: Vec<Completion> = Vec::new();

    // When browsing inside a directory, add an entry to confirm the current directory
    if !prefix.is_empty() && filter.is_empty() {
        candidates.push(Completion {
            value: prefix.clone(),
            display_name: Some("↵".to_string()),
            description: Some("Enter the current directory".to_string()),
            kind: CompletionKind::Command,
        });
    }

    candidates.extend(
        read_dir
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let file_type = entry.file_type().ok()?;
                match template {
                    Template::Filepaths => Some(entry),
                    Template::Folders if file_type.is_dir() => Some(entry),
                    Template::Folders => None,
                }
            })
            .map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                let suffix = if is_dir { "/" } else { "" };
                let display = format!("{}{}", name, suffix);
                Completion {
                    value: format!("{}{}{}", prefix, name, suffix),
                    display_name: if prefix.is_empty() {
                        None
                    } else {
                        Some(display)
                    },
                    description: None,
                    kind: CompletionKind::File,
                }
            })
            .filter(|c| filter.is_empty() || c.value[prefix.len()..].starts_with(&filter)),
    );

    (filter, candidates)
}

fn sub_completions(sub: &Subcommand, args: &[&str]) -> Vec<Completion> {
    let partial = args.last().copied().unwrap_or("");

    // If the partial input starts with "-", show options
    if partial.starts_with('-') {
        return sub
            .options
            .iter()
            .flat_map(opt_completions)
            .filter(|c| c.value.starts_with(partial))
            .collect();
    }

    let mut candidates = Vec::new();

    // Run generator if present
    if let Some(generator) = &sub.generator {
        candidates.extend(run_generator(generator, sub.generator_kind));
    }

    // Run template if present
    if let Some(template) = sub.template {
        let (_, template_candidates) = run_template(template, partial);
        candidates.extend(template_candidates);
    }

    // Filter generator results by partial input
    if !partial.is_empty() {
        candidates.retain(|c| c.kind == CompletionKind::File || c.value.starts_with(partial));
    }

    // Also include options
    candidates.extend(
        sub.options
            .iter()
            .flat_map(opt_completions)
            .filter(|c| c.value.starts_with(partial)),
    );

    candidates
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum CompletionKind {
    #[default]
    Subcommand,
    Option,
    Branch,
    File,
    Command,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum GeneratorKind {
    #[default]
    Branch,
    File,
    Command,
}

#[derive(Debug, PartialEq)]
pub struct Completion {
    pub value: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub kind: CompletionKind,
}

#[derive(Debug)]
pub enum SpecError {
    Io(std::io::Error),
    Parse(toml::de::Error),
}

impl From<std::io::Error> for SpecError {
    fn from(e: std::io::Error) -> Self {
        SpecError::Io(e)
    }
}

impl From<toml::de::Error> for SpecError {
    fn from(e: toml::de::Error) -> Self {
        SpecError::Parse(e)
    }
}

impl std::fmt::Display for SpecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecError::Io(e) => write!(f, "IO error: {}", e),
            SpecError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for SpecError {}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_SPEC: &str = r#"
name = "mycli"
"#;

    const FULL_SPEC: &str = r#"
name = "git"
description = "The stupid content tracker"

[[options]]
name = "--version"
description = "Prints the Git suite version"

[[options]]
name = "-C"
description = "Run as if git was started in <path>"
arg = "path"

[[subcommands]]
name = "clone"
description = "Clone a repository"

[[subcommands.options]]
name = "--bare"
description = "Make a bare Git repository"

[[subcommands.options]]
name = "--branch"
description = "Point to a specific branch"
arg = "branch"
short = "-b"

[[subcommands]]
name = "commit"
description = "Record changes"

[[subcommands.options]]
name = "--message"
description = "Commit message"
arg = "message"
short = "-m"

[[subcommands.options]]
name = "--amend"
description = "Amend the previous commit"
"#;

    #[test]
    fn parse_minimal_spec() {
        let spec = Spec::from_toml(MINIMAL_SPEC).unwrap();
        assert_eq!(spec.name, "mycli");
        assert_eq!(spec.description, None);
        assert!(spec.options.is_empty());
        assert!(spec.subcommands.is_empty());
    }

    #[test]
    fn parse_full_spec() {
        let spec = Spec::from_toml(FULL_SPEC).unwrap();
        assert_eq!(spec.name, "git");
        assert_eq!(spec.description, Some("The stupid content tracker".into()));
        assert_eq!(spec.options.len(), 2);
        assert_eq!(spec.subcommands.len(), 2);

        let clone = &spec.subcommands[0];
        assert_eq!(clone.name, "clone");
        assert_eq!(clone.options.len(), 2);
        assert_eq!(clone.options[1].short, Some("-b".into()));

        let commit = &spec.subcommands[1];
        assert_eq!(commit.name, "commit");
        assert_eq!(commit.options.len(), 2);
    }

    #[test]
    fn parse_option_with_arg() {
        let spec = Spec::from_toml(FULL_SPEC).unwrap();
        let opt = &spec.options[1];
        assert_eq!(opt.name, "-C");
        assert_eq!(opt.arg, Some("path".into()));
    }

    #[test]
    fn parse_invalid_toml_returns_error() {
        let result = Spec::from_toml("not valid {{{{");
        assert!(result.is_err());
    }

    #[test]
    fn completions_empty_args_returns_all() {
        let spec = Spec::from_toml(FULL_SPEC).unwrap();
        let completions = spec.completions(&[""]);
        let values: Vec<&str> = completions.iter().map(|c| c.value.as_str()).collect();
        assert!(values.contains(&"clone"));
        assert!(values.contains(&"commit"));
        assert!(values.contains(&"--version"));
        assert!(values.contains(&"-C"));
    }

    #[test]
    fn completions_partial_filters_subcommands() {
        let spec = Spec::from_toml(FULL_SPEC).unwrap();
        let completions = spec.completions(&["cl"]);
        let values: Vec<&str> = completions.iter().map(|c| c.value.as_str()).collect();
        assert_eq!(values, vec!["clone"]);
    }

    #[test]
    fn completions_subcommand_shows_its_options() {
        let spec = Spec::from_toml(FULL_SPEC).unwrap();
        let completions = spec.completions(&["commit", ""]);
        let values: Vec<&str> = completions.iter().map(|c| c.value.as_str()).collect();
        assert!(values.contains(&"--message"));
        assert!(values.contains(&"-m"));
        assert!(values.contains(&"--amend"));
    }

    #[test]
    fn completions_subcommand_filters_options() {
        let spec = Spec::from_toml(FULL_SPEC).unwrap();
        let completions = spec.completions(&["commit", "--a"]);
        let values: Vec<&str> = completions.iter().map(|c| c.value.as_str()).collect();
        assert_eq!(values, vec!["--amend"]);
    }

    #[test]
    fn completions_unknown_subcommand_returns_empty() {
        let spec = Spec::from_toml(FULL_SPEC).unwrap();
        let completions = spec.completions(&["nonexistent", ""]);
        assert!(completions.is_empty());
    }

    #[test]
    fn from_file_reads_spec() {
        let spec = Spec::from_file(Path::new("specs/git.toml")).unwrap();
        assert_eq!(spec.name, "git");
        assert!(!spec.subcommands.is_empty());
    }

    #[test]
    fn parse_template_filepaths() {
        let spec = Spec::from_toml(
            r#"
name = "mycli"

[[subcommands]]
name = "open"
description = "Open a file"
template = "filepaths"
"#,
        )
        .unwrap();
        assert_eq!(spec.subcommands[0].template, Some(Template::Filepaths));
    }

    #[test]
    fn parse_template_folders() {
        let spec = Spec::from_toml(
            r#"
name = "mycli"

[[subcommands]]
name = "cd"
description = "Change directory"
template = "folders"
"#,
        )
        .unwrap();
        assert_eq!(spec.subcommands[0].template, Some(Template::Folders));
    }

    #[test]
    fn parse_no_template() {
        let spec = Spec::from_toml(FULL_SPEC).unwrap();
        assert_eq!(spec.subcommands[0].template, None);
    }

    #[test]
    fn template_filepaths_returns_files_and_dirs() {
        let (_, completions) = run_template(Template::Filepaths, "");
        // We're running from the project root, so Cargo.toml should be present
        let values: Vec<&str> = completions.iter().map(|c| c.value.as_str()).collect();
        assert!(values.contains(&"Cargo.toml"));
        assert!(values.contains(&"src/"));
        assert!(completions.iter().all(|c| c.kind == CompletionKind::File));
    }

    #[test]
    fn template_folders_returns_only_dirs() {
        let (_, completions) = run_template(Template::Folders, "");
        let values: Vec<&str> = completions.iter().map(|c| c.value.as_str()).collect();
        assert!(values.contains(&"src/"));
        assert!(!values.contains(&"Cargo.toml"));
    }

    #[test]
    fn template_folders_have_trailing_slash() {
        let (_, completions) = run_template(Template::Folders, "");
        assert!(completions.iter().all(|c| c.value.ends_with('/')));
    }

    #[test]
    fn template_subdirectory_traversal() {
        // Simulates typing "src/" — should list contents of src/
        let (_, completions) = run_template(Template::Filepaths, "src/");
        let values: Vec<&str> = completions.iter().map(|c| c.value.as_str()).collect();
        assert!(values.contains(&"src/main.rs"));
        assert!(values.contains(&"src/spec.rs"));
    }

    #[test]
    fn template_subdirectory_with_partial() {
        // Simulates typing "src/sp" — should filter within src/
        let (_, completions) = run_template(Template::Filepaths, "src/sp");
        let values: Vec<&str> = completions.iter().map(|c| c.value.as_str()).collect();
        assert!(values.contains(&"src/spec.rs"));
        assert!(!values.contains(&"src/main.rs"));
    }

    #[test]
    fn completions_with_template_filepaths() {
        let spec = Spec::from_toml(
            r#"
name = "mycli"

[[subcommands]]
name = "open"
template = "filepaths"
"#,
        )
        .unwrap();
        let completions = spec.completions(&["open", ""]);
        let values: Vec<&str> = completions.iter().map(|c| c.value.as_str()).collect();
        assert!(values.contains(&"Cargo.toml"));
        assert!(values.contains(&"src/"));
    }

    #[test]
    fn completions_with_template_filters_by_partial() {
        let spec = Spec::from_toml(
            r#"
name = "mycli"

[[subcommands]]
name = "open"
template = "filepaths"
"#,
        )
        .unwrap();
        let completions = spec.completions(&["open", "Car"]);
        let values: Vec<&str> = completions.iter().map(|c| c.value.as_str()).collect();
        assert!(values.contains(&"Cargo.toml"));
        assert!(!values.contains(&"src/"));
    }

    #[test]
    fn spec_level_template_folders_empty_args() {
        let spec = Spec::from_toml(
            r#"
name = "cd"
description = "Change directory"
template = "folders"
"#,
        )
        .unwrap();
        let completions = spec.completions(&[""]);
        let values: Vec<&str> = completions.iter().map(|c| c.value.as_str()).collect();
        assert!(values.contains(&"src/"));
        assert!(!values.contains(&"Cargo.toml"));
    }

    #[test]
    fn spec_level_template_filters_by_partial() {
        let spec = Spec::from_toml(
            r#"
name = "cd"
template = "folders"
"#,
        )
        .unwrap();
        let completions = spec.completions(&["sr"]);
        let values: Vec<&str> = completions.iter().map(|c| c.value.as_str()).collect();
        assert!(values.contains(&"src/"));
    }

    #[test]
    fn spec_level_template_subdirectory() {
        let spec = Spec::from_toml(
            r#"
name = "mycli"
template = "filepaths"
"#,
        )
        .unwrap();
        // Simulates: mycli src/ <tab> — "src/" is passed as partial
        let completions = spec.completions(&["src/"]);
        let values: Vec<&str> = completions.iter().map(|c| c.value.as_str()).collect();
        // src/ contains files like main.rs, spec.rs
        assert!(values.contains(&"src/main.rs"));
        assert!(values.contains(&"src/spec.rs"));
    }

    #[test]
    fn from_file_reads_cd_spec() {
        let spec = Spec::from_file(Path::new("specs/cd.toml")).unwrap();
        assert_eq!(spec.name, "cd");
        assert_eq!(spec.template, Some(Template::Filepaths));
    }
}
