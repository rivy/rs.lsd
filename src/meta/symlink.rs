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
        match Self::try_from(path) {
            Ok(symlink) => symlink,
            Err(_) => Self {
                target: None,
                valid: false,
            },
        }
    }
}

impl SymLink {
    pub fn symlink_string(&self, file_name: &String) -> Option<String> {
        let prefix = match file_name.as_str() {
            "." => format!("..{}", std::path::MAIN_SEPARATOR),
            ".." => format!(
                "..{}..{}",
                std::path::MAIN_SEPARATOR,
                std::path::MAIN_SEPARATOR
            ),
            _ => format!(""),
        };
        self.target
            .as_ref()
            .map(|target| format!("{}{}", prefix, target.to_string()))
    }

    pub fn render(&self, colors: &Colors, flag: &Flags, file_name: &String) -> ColoredString {
        // eprintln!("SymLink::render\n ~ self: {:#?}", self,);
        if let Some(target_string) = self.symlink_string(file_name) {
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

    fn try_from(path: &Path) -> std::io::Result<Self> {
        if let Ok(target) = read_link(path) {
            // eprintln!(
            //     "SymLink::From\n~ target: {:#?} of path: {:#?}",
            //     // "SymLink::From ~ target: {:#?} of path: {:#?}\n* components: {:#?}",
            //     target,
            //     path,
            //     // path.components()
            // );
            if target.is_absolute() {
                return Ok(Self {
                    target: Some(
                        target
                            .to_str()
                            .expect("failed to convert symlink to str")
                            .to_string(),
                    ),
                    valid: target.exists(),
                });
            }

            let path_verbatim = crate::meta::utils::to_verbatim_path(&path)?;
            let parent = path_verbatim
                .parent()
                .expect("failed to get parent of path for symlink");
            let final_target = parent.join(&target);
            let valid = final_target.exists();
            let t = crate::meta::utils::relative_path(&final_target, &parent);
            let target = Some(
                t.to_str()
                    .expect("failed to convert symlink to str")
                    .to_string(),
            );
            // eprintln!("SymLink::From\n~ path: {:#?}\n~ path_verbatim: {:#?}\n~ parent: {:#?}\n~ target: {:#?}\n~ final_target: {:#?}\n~ valid: {:#?}\n ~ t: {:#?}", path, path_verbatim, parent, target, final_target, valid, t);

            return Ok(Self { target, valid });
        }

        Ok(Self {
            target: None,
            valid: false,
        })
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
                &Flags::configure_from(&cli, &Config::with_none()).unwrap(),
                &String::from("")
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
                &Flags::configure_from(&cli, &Config::with_none()).unwrap(),
                &String::from("")
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
                &Flags::configure_from(&cli, &Config::with_none()).unwrap(),
                &String::from("")
            )
            .to_string()
        );
    }
}
