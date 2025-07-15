use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub struct AdeptError {
    pub name: String,
    pub args: Vec<String>,
}

impl Display for AdeptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("AdeptError({}, {:?})", self.name, self.args,))
    }
}

impl Error for AdeptError {}
