/// Generate the shell hook script for the given shell.
/// Embeds the full path to the current gig binary.
pub fn init_script(shell: &str) -> String {
    let gig_bin = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "gig".to_string());

    match shell {
        "zsh" => zsh_hook(&gig_bin),
        "bash" => bash_hook(&gig_bin),
        _ => "# Unsupported shell\n".to_string(),
    }
}

fn zsh_hook(gig_bin: &str) -> String {
    format!(
        r#"# gig shell integration (zsh)

# Save the original tab handler
if [[ -z "$_gig_original_widget" ]]; then
    _gig_original_widget="expand-or-complete"
fi

_gig_fzf_complete() {{
    local cmd
    local -a tokens
    tokens=(${{(z)LBUFFER}})
    cmd="${{tokens[1]}}"

    local specs_dir="${{GIG_SPECS_DIR:-$HOME/.config/gig/specs}}"

    # If no gig spec for this command, fall back to default
    if [[ ! -f "$specs_dir/$cmd.toml" ]]; then
        zle "${{_gig_original_widget}}"
        return
    fi

    local -a args
    args=(${{tokens[2,-1]}})

    # If the line ends with a space, the user finished the current token
    # and expects completions for the next position. Append an empty arg
    # so that spec.completions() sees e.g. ["add", ""] instead of ["add"].
    if [[ "$LBUFFER" == *" " ]]; then
        args+=("")
    fi

    # Estimate cursor column for dropdown positioning
    # Use LBUFFER length + small offset for prompt (safe fallback)
    local indent=$((${{#LBUFFER}} + 2))

    # Use a temp file for the result. crossterm reads /dev/tty directly (use-dev-tty feature).
    local tmpfile=$(mktemp /tmp/gig-pick.XXXXXX)
    "{gig}" pick --output "$tmpfile" --indent "$indent" "$cmd" "${{args[@]}}" >/dev/tty 2>/dev/tty
    local selected=$(<"$tmpfile")
    rm -f "$tmpfile"

    if [[ -n "$selected" ]]; then
        # Remove the partial word being typed
        local partial="${{tokens[-1]}}"
        if (( ${{#tokens}} > 1 )) && [[ "$LBUFFER" != *" " ]]; then
            LBUFFER="${{LBUFFER%$partial}}$selected "
        else
            LBUFFER="${{LBUFFER% }} $selected "
        fi
        zle reset-prompt
    else
        zle reset-prompt
    fi
}}

zle -N _gig_fzf_complete
bindkey '^I' _gig_fzf_complete
"#,
        gig = gig_bin
    )
}

fn bash_hook(gig_bin: &str) -> String {
    format!(
        r#"# gig shell integration (bash)
_gig_completions() {{
    local cmd="${{COMP_WORDS[0]}}"
    local cur="${{COMP_WORDS[COMP_CWORD]}}"
    local args="${{COMP_WORDS[@]:1}}"

    local IFS=$'\n'
    local completions=$("{gig}" complete "$cmd" $args 2>/dev/null)

    local -a values
    while IFS=$'\t' read -r value desc; do
        values+=("$value")
    done <<< "$completions"

    COMPREPLY=($(compgen -W "${{values[*]}}" -- "$cur"))
}}

# Register for known commands
_gig_register() {{
    local specs_dir="${{GIG_SPECS_DIR:-$HOME/.config/gig/specs}}"
    if [[ -d "$specs_dir" ]]; then
        for spec_file in "$specs_dir"/*.toml; do
            local cmd=$(basename "$spec_file" .toml)
            complete -F _gig_completions "$cmd"
        done
    fi
}}

_gig_register
"#,
        gig = gig_bin
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zsh_script_contains_pick_widget() {
        let script = init_script("zsh");
        assert!(script.contains("_gig_fzf_complete"));
        assert!(script.contains("pick --output"));
        assert!(script.contains("bindkey"));
        assert!(script.contains("/dev/tty"));
        assert!(script.contains("mktemp"));
    }

    #[test]
    fn zsh_script_embeds_binary_path() {
        let script = init_script("zsh");
        // Should contain an absolute path or "gig"
        assert!(script.contains("/gig") || script.contains("\"gig\""));
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
