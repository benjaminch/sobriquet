use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum Shell {
    Zsh,
    Bash,
    Fish,
}

impl Shell {
    pub const fn alias_args(self) -> &'static [&'static str] {
        match self {
            Self::Fish => &["-c", "alias"],
            Self::Zsh | Self::Bash => &["-ic", "alias"],
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Zsh => "zsh",
            Self::Bash => "bash",
            Self::Fish => "fish",
        }
    }

    pub const fn detection_order() -> &'static [Self] {
        &[Self::Zsh, Self::Bash]
    }

    pub fn parse_shell(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "zsh" => Some(Self::Zsh),
            "bash" => Some(Self::Bash),
            "fish" => Some(Self::Fish),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum InitShell {
    Zsh,
    Bash,
    Fish,
}

pub fn generate_init_script(shell: InitShell) -> &'static str {
    match shell {
        InitShell::Zsh => {
            r#"# alx - fuzzy finder for shell aliases
# Add this to your ~/.zshrc

alx() {
  case "$1" in
    init|generate|config|stats|audit|--help|-h|--version|-V)
      command alx "$@"
      return
      ;;
  esac
  local selected
  selected=$(command alx "$@" 2>/dev/null)
  local exit_code=$?
  if [[ $exit_code -eq 0 && -n "$selected" ]]; then
    print -z "$selected"
  fi
  return $exit_code
}
"#
        }
        InitShell::Bash => {
            r#"# alx - fuzzy finder for shell aliases
# Add this to your ~/.bashrc

alx() {
  case "$1" in
    init|generate|config|stats|audit|--help|-h|--version|-V)
      command alx "$@"
      return
      ;;
  esac
  local selected
  selected=$(command alx "$@" 2>/dev/null)
  local exit_code=$?
  if [[ $exit_code -eq 0 && -n "$selected" ]]; then
    if [[ -n "$READLINE_LINE" ]] || [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
      READLINE_LINE="$selected"
      READLINE_POINT=${#READLINE_LINE}
    else
      history -s "$selected"
      echo "$selected"
    fi
  fi
  return $exit_code
}
"#
        }
        InitShell::Fish => {
            r#"# alx - fuzzy finder for shell aliases
# Add this to your ~/.config/fish/config.fish

function alx --description "Fuzzy finder for shell aliases"
  switch $argv[1]
    case init generate config stats audit --help -h --version -V
      command alx $argv
      return
  end
  set -l selected (command alx $argv 2>/dev/null)
  set -l exit_code $status
  if test $exit_code -eq 0 -a -n "$selected"
    commandline -r "$selected"
    commandline -f repaint
  end
  return $exit_code
end
"#
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_parse() {
        assert_eq!(Shell::parse_shell("zsh"), Some(Shell::Zsh));
        assert_eq!(Shell::parse_shell("ZSH"), Some(Shell::Zsh));
        assert_eq!(Shell::parse_shell("bash"), Some(Shell::Bash));
        assert_eq!(Shell::parse_shell("fish"), Some(Shell::Fish));
        assert_eq!(Shell::parse_shell("invalid"), None);
    }

    #[test]
    fn alias_args() {
        assert!(Shell::Zsh.alias_args().contains(&"-ic"));
        assert!(Shell::Bash.alias_args().contains(&"-ic"));
        assert!(Shell::Fish.alias_args().contains(&"-c"));
    }

    #[test]
    fn detection_order_prefers_zsh() {
        assert_eq!(Shell::detection_order()[0], Shell::Zsh);
    }

    #[test]
    fn init_scripts() {
        assert!(generate_init_script(InitShell::Zsh).contains("print -z"));
        assert!(
            generate_init_script(InitShell::Bash).contains("READLINE_LINE")
        );
        assert!(generate_init_script(InitShell::Fish).contains("commandline"));
    }
}
