use std::error::Error;
use std::{fmt, io};
pub mod protocol;
pub mod keybase_cmd {
    use std::io::{self, Write};
    use std::process::{Command, Stdio};
    thread_local! {
        pub static KEYBASE: String = which_keybase();
    }

    pub fn which_keybase() -> String {
        let path = String::from_utf8(
            Command::new("which")
                .arg("keybase")
                .output()
                .expect("which is not installed")
                .stdout,
        )
        .expect("Output not in UTF-8");
        String::from(path.trim())
    }

    pub fn call_chat_api(input: &[u8]) -> Result<Vec<u8>, io::Error> {
        let mut child = KEYBASE.with(|kb| {
            Command::new(kb)
                .arg("chat")
                .arg("api")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Couldn't run `keybase`")
        });
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(input)?;
            let output = child.wait_with_output()?;
            if !output.status.success() {
                println!("Keybase did not return successful exit code");
                return Err(io::ErrorKind::InvalidInput.into());
            }

            Ok(output.stdout)
        } else {
            Err(io::ErrorKind::BrokenPipe.into())
        }
    }
}

#[derive(Debug)]
pub enum ApiError {
    Parsing(serde_json::error::Error),
    IOErr(io::Error),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ApiError {}

impl From<serde_json::error::Error> for ApiError {
    fn from(error: serde_json::error::Error) -> Self {
        ApiError::Parsing(error)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(error: std::io::Error) -> Self {
        ApiError::IOErr(error)
    }
}

pub mod chat {
    use super::ApiError;
    use crate::keybase_cmd::call_chat_api;
    // use crate::protocol::chat1::api;
    use serde::{Deserialize, Serialize};
    use serde_json;
    // use std::io::Error as IOError;

    #[derive(Serialize, Deserialize, Debug)]
    struct List {
        method: &'static str,
    }

    const LISTMETHOD: List = List { method: "list" };



    pub fn list() -> Result<String, ApiError> {
        // let input: Vec<u8>
        // call_chat_api()
        let input = serde_json::to_vec(&LISTMETHOD)?;
        let output = call_chat_api(&input)?;
        let output = String::from_utf8(output).unwrap();
        println!("Output is {:?}", output);
        serde_json::from_str(&output)?;
        panic!()
    }
}

#[cfg(test)]
mod tests {
    use super::chat::*;
    use super::keybase_cmd::*;
    use super::*;

    #[test]
    fn can_find_keybase() {
        println!("Keybase is at: {}", which_keybase());
        assert!(!which_keybase().is_empty());
    }

    #[test]
    fn ls_inbox() {
        list().unwrap();
    }
}
