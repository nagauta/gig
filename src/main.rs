mod complete;
mod installer;
mod shell;
mod spec;
mod tui;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "gig",
    version,
    about = "Terminal autocomplete for any CLI tool"
)]
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
    /// Interactive completion picker (replaces fzf)
    Pick {
        /// File to write the selected value to (avoids stdout capture issues)
        #[arg(long)]
        output: String,
        /// Column offset for dropdown positioning
        #[arg(long, default_value_t = 0)]
        indent: u16,
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
        Some(Commands::Pick {
            output,
            indent,
            command,
            args,
        }) => {
            let specs_dir = complete::default_specs_dir();
            let specs = complete::load_specs(&specs_dir);
            if let Some(spec) = complete::find_spec(&specs, &command) {
                let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                let completions = spec.completions(&arg_refs);
                match tui::run(completions, indent) {
                    Ok(Some(value)) => {
                        let _ = std::fs::write(&output, &value);
                    }
                    Ok(None) => {}
                    Err(e) => eprintln!("gig: picker error: {:?}", e),
                }
            }
        }
        Some(Commands::Install) => {
            let shell_name = detect_shell();
            match installer::rc_path(shell_name) {
                Some(rc) => {
                    match installer::install(&rc, shell_name) {
                        Ok(true) => println!("Added gig hook to {}", rc.display()),
                        Ok(false) => println!("gig is already installed in {}", rc.display()),
                        Err(e) => eprintln!("Failed to install: {}", e),
                    }
                    // Copy bundled specs
                    let bundled = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("specs");
                    let target = complete::default_specs_dir();
                    match installer::install_specs(&bundled, &target) {
                        Ok(n) => {
                            println!("Installed {} completion specs to {}", n, target.display())
                        }
                        Err(e) => eprintln!("Failed to install specs: {}", e),
                    }
                }
                None => eprintln!("Unsupported shell: {}", shell_name),
            }
        }
        Some(Commands::Uninstall) => {
            let shell_name = detect_shell();
            match installer::rc_path(shell_name) {
                Some(rc) => match installer::uninstall(&rc) {
                    Ok(true) => println!("Removed gig hook from {}", rc.display()),
                    Ok(false) => println!("gig is not installed in {}", rc.display()),
                    Err(e) => eprintln!("Failed to uninstall: {}", e),
                },
                None => eprintln!("Unsupported shell: {}", shell_name),
            }
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
