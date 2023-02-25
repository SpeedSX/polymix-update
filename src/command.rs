use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::{Display, EnumIter};

#[derive(Debug, PartialEq, Display, EnumIter)]
pub enum Command {
    Upload,
    Download,
    List,
}

impl Command {
    fn from_str_case_insensitive(input: &str) -> Result<Command, ()> {
        for variant in Command::iter() {
            if input.eq_ignore_ascii_case(&variant.to_string()) {
                return Ok(variant);
            }
        }
        Err(())
    }
}

impl FromStr for Command {
    type Err = ();

    fn from_str(input: &str) -> Result<Command, Self::Err> {
        Command::from_str_case_insensitive(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_case_insensitive() {
        assert_eq!(Command::from_str_case_insensitive("Upload"), Ok(Command::Upload));
        assert_eq!(Command::from_str_case_insensitive("upload"), Ok(Command::Upload));
        assert_eq!(Command::from_str_case_insensitive("Download"), Ok(Command::Download));
        assert_eq!(Command::from_str_case_insensitive("download"), Ok(Command::Download));
        assert_eq!(Command::from_str_case_insensitive("List"), Ok(Command::List));
        assert_eq!(Command::from_str_case_insensitive("list"), Ok(Command::List));
        assert_eq!(Command::from_str_case_insensitive("invalid"), Err(()));
    }
}
