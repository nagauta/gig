use serde::Deserialize;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Spec {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub options: Vec<Opt>,
    #[serde(default)]
    pub subcommands: Vec<Subcommand>,
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
                        description: s.description.clone(),
                        kind: CompletionKind::Subcommand,
                    })
                    .collect();
                result.extend(self.options.iter().flat_map(opt_completions));
                result
            }
            [partial] => {
                // Filter subcommands and options by prefix
                let mut result: Vec<Completion> = self
                    .subcommands
                    .iter()
                    .filter(|s| s.name.starts_with(partial))
                    .map(|s| Completion {
                        value: s.name.clone(),
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
                result
            }
            [subcmd, rest @ ..] => {
                // Delegate to subcommand
                if let Some(sub) = self.subcommands.iter().find(|s| s.name == *subcmd) {
                    sub_completions(sub, rest)
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
        description: opt.description.clone(),
        kind: CompletionKind::Option,
    }];
    if let Some(short) = &opt.short {
        result.push(Completion {
            value: short.clone(),
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
    };
    let output = Command::new("sh").arg("-c").arg(command).output().ok();
    match output {
        Some(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| Completion {
                value: l.trim().to_string(),
                description: None,
                kind,
            })
            .collect(),
        _ => vec![],
    }
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

    // Otherwise, try generator for positional arguments
    if let Some(generator) = &sub.generator {
        let mut candidates = run_generator(generator, sub.generator_kind);
        if !partial.is_empty() {
            candidates.retain(|c| c.value.starts_with(partial));
        }
        // Also include options
        candidates.extend(
            sub.options
                .iter()
                .flat_map(opt_completions)
                .filter(|c| c.value.starts_with(partial)),
        );
        return candidates;
    }

    // Fallback: show options
    sub.options
        .iter()
        .flat_map(opt_completions)
        .filter(|c| c.value.starts_with(partial))
        .collect()
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum CompletionKind {
    #[default]
    Subcommand,
    Option,
    Branch,
    File,
}

#[derive(Debug, Deserialize, PartialEq, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum GeneratorKind {
    #[default]
    Branch,
    File,
}

#[derive(Debug, PartialEq)]
pub struct Completion {
    pub value: String,
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
}
