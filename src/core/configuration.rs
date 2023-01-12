// app configuration options that can be set via the command line

use std::env;

#[derive(Debug)]
pub struct Configuration {
    authentication_mode: AuthenticationMode
}

impl Configuration {
    pub fn new(authentication_mode: AuthenticationMode) -> Self {
        Self {
            authentication_mode
        }
    }

    pub fn from_args() -> Self {
        let mut mode = AuthenticationMode::None;

        let args: Vec<String> = env::args().collect();
        let mode_arg = args.iter().find(|arg| arg.to_lowercase().starts_with("--auth-mode="));
        if let Some(pair) = mode_arg {
            let split: Vec<&str> = pair.split('=').collect();
            let value = split.get(1);
            if let Some(thing) = value {
                if thing.eq(&"basic") {
                    mode = AuthenticationMode::Basic;
                }
            }
        }

        Self::new(mode)
    }

    /// returns a copy of self's authentication mode
    pub fn authentication_mode(&self) -> AuthenticationMode {
        self.authentication_mode.clone()
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new(AuthenticationMode::None)
    }
}

/// designates which request authentication scheme to use
#[derive(Debug)]
pub enum AuthenticationMode {
    None,
    Basic
}

impl Clone for AuthenticationMode {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::Basic => Self::Basic,
        }
    }
}