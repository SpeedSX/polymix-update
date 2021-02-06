use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Command {
    Upload,
    Download,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(input: &str) -> Result<Command, Self::Err> {
        match input.to_lowercase().as_str() {
            "upload" => Ok(Command::Upload),
            "download" => Ok(Command::Download),
            _ => Err(()),
        }
    }
}
