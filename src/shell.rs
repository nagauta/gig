/// Generate the shell hook script for the given shell.
pub fn init_script(shell: &str) -> &'static str {
    match shell {
        "zsh" => ZSH_HOOK,
        "bash" => BASH_HOOK,
        _ => "# Unsupported shell\n",
    }
}

const ZSH_HOOK: &str = r#"# gig shell integration (zsh)
_gig_completions() {
    local -a completions
    local cmd="${words[1]}"
    local args="${words[2,-1]}"

    # Call gig complete with current arguments
    local IFS=$'\n'
    completions=($(gig complete "$cmd" ${=args} 2>/dev/null))

    local completion
    for completion in "${completions[@]}"; do
        local value="${completion%%	*}"
        local desc="${completion#*	}"
        if [[ "$value" != "$desc" ]]; then
            compadd -X "$desc" -- "$value"
        else
            compadd -- "$value"
        fi
    done
}

# Register for known commands
_gig_register() {
    local specs_dir="${GIG_SPECS_DIR:-$HOME/.config/gig/specs}"
    if [[ -d "$specs_dir" ]]; then
        for spec_file in "$specs_dir"/*.toml; do
            local cmd=$(basename "$spec_file" .toml)
            compdef _gig_completions "$cmd"
        done
    fi
}

_gig_register
"#;

const BASH_HOOK: &str = r#"# gig shell integration (bash)
_gig_completions() {
    local cmd="${COMP_WORDS[0]}"
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local args="${COMP_WORDS[@]:1}"

    local IFS=$'\n'
    local completions=$(gig complete "$cmd" $args 2>/dev/null)

    local -a values
    while IFS=$'\t' read -r value desc; do
        values+=("$value")
    done <<< "$completions"

    COMPREPLY=($(compgen -W "${values[*]}" -- "$cur"))
}

# Register for known commands
_gig_register() {
    local specs_dir="${GIG_SPECS_DIR:-$HOME/.config/gig/specs}"
    if [[ -d "$specs_dir" ]]; then
        for spec_file in "$specs_dir"/*.toml; do
            local cmd=$(basename "$spec_file" .toml)
            complete -F _gig_completions "$cmd"
        done
    fi
}

_gig_register
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zsh_script_contains_compdef() {
        let script = init_script("zsh");
        assert!(script.contains("compdef"));
        assert!(script.contains("_gig_completions"));
        assert!(script.contains("gig complete"));
    }

    #[test]
    fn bash_script_contains_complete() {
        let script = init_script("bash");
        assert!(script.contains("complete -F"));
        assert!(script.contains("_gig_completions"));
        assert!(script.contains("COMPREPLY"));
    }

    #[test]
    fn unsupported_shell_returns_comment() {
        let script = init_script("fish");
        assert!(script.contains("Unsupported"));
    }
}
