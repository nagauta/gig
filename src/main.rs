use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fig", version, about = "Terminal autocomplete for any CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize fig for your shell
    Init {
        /// Shell to initialize (bash, zsh)
        #[arg(value_enum)]
        shell: Option<Shell>,
    },
    /// Install fig into your shell config
    Install,
    /// Remove fig from your shell config
    Uninstall,
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
            println!("Initializing fig for {}...", shell_name);
            // TODO: Output shell hook script
        }
        Some(Commands::Install) => {
            println!("Installing fig...");
            // TODO: Add hook to shell config
        }
        Some(Commands::Uninstall) => {
            println!("Uninstalling fig...");
            // TODO: Remove hook from shell config
        }
        None => {
            println!("fig v{}", env!("CARGO_PKG_VERSION"));
            println!("Run `fig --help` for usage.");
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
