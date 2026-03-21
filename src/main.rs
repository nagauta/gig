mod complete;
mod shell;
mod spec;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gig", version, about = "Terminal autocomplete for any CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize gig for your shell (prints hook script to stdout)
    Init {
        /// Shell to initialize (bash, zsh)
        #[arg(value_enum)]
        shell: Option<Shell>,
    },
    /// Install gig into your shell config
    Install,
    /// Remove gig from your shell config
    Uninstall,
    /// Generate completions for a command (used internally by shell hook)
    Complete {
        /// The command to complete for
        command: String,
        /// The arguments typed so far
        args: Vec<String>,
    },
}

#[derive(Clone, clap::ValueEnum)]
enum Shell {
    Bash,
    Zsh,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init { shell }) => {
            let shell_name = match shell {
                Some(Shell::Bash) => "bash",
                Some(Shell::Zsh) => "zsh",
                None => detect_shell(),
            };
            print!("{}", shell::init_script(shell_name));
        }
        Some(Commands::Complete { command, args }) => {
            let specs_dir = complete::default_specs_dir();
            let specs = complete::load_specs(&specs_dir);
            if let Some(spec) = complete::find_spec(&specs, &command) {
                let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let output = complete::generate_completions(spec, &arg_refs);
                if !output.is_empty() {
                    println!("{}", output);
                }
            }
        }
        Some(Commands::Install) => {
            println!("Installing gig...");
            // TODO: Add hook to shell config
        }
        Some(Commands::Uninstall) => {
            println!("Uninstalling gig...");
            // TODO: Remove hook from shell config
        }
        None => {
            println!("gig v{}", env!("CARGO_PKG_VERSION"));
            println!("Run `gig --help` for usage.");
        }
    }
}

fn detect_shell() -> &'static str {
    std::env::var("SHELL")
        .ok()
        .and_then(|s| {
            if s.contains("zsh") {
                Some("zsh")
            } else if s.contains("bash") {
                Some("bash")
            } else {
                None
            }
        })
        .unwrap_or("zsh")
}
