use crate::color::{ColoredString, Colors, Elem};
use crate::flags::Flags;
use std::fs::read_link;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct SymLink {
    target: Option<String>,
    valid: bool,
}

impl From<&Path> for SymLink {
    fn from(path: &Path) -> Self {
        if let Ok(target) = read_link(path) {
            eprintln!(
                "SymLink::From ~ target: {:#?} of path: {:#?}\n* components: {:#?}",
                target,
                path,
                path.components()
            );
            if target.is_absolute() {
                return Self {
                    valid: target.exists(),
                    target: Some(
                        target
                            .to_str()
                            .expect("failed to convert symlink to str")
                            .to_string(),
                    ),
                };
            }

            eprintln!("SymLink::From ~ path.parent(): {:#?}", path.parent());
            eprintln!(
                "SymLink::From ~ path.parent(): {:#?}",
                std::path::absolute(path).unwrap().parent()
            );
            let parent = match path.parent() {
                Some(p) => match path.as_os_str().to_str() {
                    Some(".") => std::path::Path::new(".."),
                    Some("..") => std::path::Path::new("../.."),
                    _ => p,
                },
                None => path,
            };
            eprintln!("SymLink::From ~ parent: {:#?}", parent);
            return Self {
                target: match path.as_os_str().to_str() {
                    Some(".") | Some("..") => Some(
                        parent
                            .join(&target)
                            .to_str()
                            .expect("failed to convert symlink to str")
                            .to_string(),
                    ),
                    _ => Some(
                        target
                            .to_str()
                            .expect("failed to convert symlink to str")
                            .to_string(),
                    ),
                },
                valid: parent.join(&target).exists(),
            };
        }

        Self {
            target: None,
            valid: false,
        }
    }
}

impl SymLink {
    pub fn symlink_string(&self) -> Option<String> {
        self.target.as_ref().map(|target| target.to_string())
    }

    pub fn render(&self, colors: &Colors, flag: &Flags) -> ColoredString {
        if let Some(target_string) = self.symlink_string() {
            let elem = if self.valid {
                &Elem::SymLink
            } else {
                &Elem::MissingSymLinkTarget
            };

            let strings: &[ColoredString] = &[
                ColoredString::new(Colors::default_style(), format!(" {} ", flag.symlink_arrow)), // ⇒ \u{21d2}
                colors.colorize(target_string, elem),
            ];

            let res = strings
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join("");
            ColoredString::new(Colors::default_style(), res)
        } else {
            ColoredString::new(Colors::default_style(), "".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::SymLink;

    use crate::app::Cli;
    use crate::color::{Colors, ThemeOption};
    use crate::config_file::Config;
    use crate::flags::Flags;

    #[test]
    fn test_symlink_render_default_valid_target_nocolor() {
        let link = SymLink {
            target: Some("/target".to_string()),
            valid: true,
        };
        let argv = ["lsd"];
        let cli = Cli::try_parse_from(argv).unwrap();
        assert_eq!(
            format!("{}", " ⇒ /target"),
            link.render(
                &Colors::new(ThemeOption::NoColor),
                &Flags::configure_from(&cli, &Config::with_none()).unwrap()
            )
            .to_string()
        );
    }

    #[test]
    fn test_symlink_render_default_invalid_target_nocolor() {
        let link = SymLink {
            target: Some("/target".to_string()),
            valid: false,
        };
        let argv = ["lsd"];
        let cli = Cli::try_parse_from(argv).unwrap();
        assert_eq!(
            format!("{}", " ⇒ /target"),
            link.render(
                &Colors::new(ThemeOption::NoColor),
                &Flags::configure_from(&cli, &Config::with_none()).unwrap()
            )
            .to_string()
        );
    }

    #[test]
    fn test_symlink_render_default_invalid_target_withcolor() {
        let link = SymLink {
            target: Some("/target".to_string()),
            valid: false,
        };
        let argv = ["lsd"];
        let cli = Cli::try_parse_from(argv).unwrap();
        assert_eq!(
            format!("{}", " ⇒ \u{1b}[38;5;124m/target\u{1b}[39m"),
            link.render(
                &Colors::new(ThemeOption::NoLscolors),
                &Flags::configure_from(&cli, &Config::with_none()).unwrap()
            )
            .to_string()
        );
    }
}
