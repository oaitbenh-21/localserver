// src/config/error.rs

use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse { message: String, pos: usize },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "Could not read config file: {}", e),
            ConfigError::Parse { message, pos } => {
                write!(f, "Config parse error at token {}: {}", pos, message)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

// Lets us use ? when reading the config file
impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> ConfigError {
        ConfigError::Io(e)
    }
}

impl ConfigError {
    pub fn parse(message: impl Into<String>, pos: usize) -> ConfigError {
        ConfigError::Parse {
            message: message.into(),
            pos,
        }
    }
}
