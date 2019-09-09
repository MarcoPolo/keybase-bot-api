pub mod chat;
pub mod status;
use futures::channel::mpsc;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::{fmt, io};

pub mod keybase_cmd {
    use super::{ApiError, KBError};
    use futures::{channel::mpsc, stream::Stream};
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_json;
    use std::io::{self, BufRead, BufReader, Write};
    use std::process::{Command, Stdio};
    use std::thread;

    thread_local! {
        pub static KEYBASE: String = which_keybase();
    }

    #[derive(Deserialize, Serialize)]
    pub struct APIResult<T> {
        pub result: T,
        pub error: Option<KBError>,
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

    #[derive(Serialize, Deserialize, Debug)]
    pub struct StatusRes {
        #[serde(rename = "Username")]
        pub username: String,
    }

    pub fn call_status() -> Result<StatusRes, ApiError> {
        let child = KEYBASE.with(|kb| {
            Command::new(kb)
                .arg("status")
                .arg("-j")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Couldn't run `keybase`")
        });
        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Keybase did not return successful exit code",
            )
            .into());
        }

        let output = String::from_utf8(output.stdout)?;
        let res: StatusRes = serde_json::from_str(&output)?;
        Ok(res)
    }

    pub fn call_chat_api<T>(input: &[u8]) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
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
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Keybase did not return successful exit code",
                )
                .into());
            }

            let output = String::from_utf8(output.stdout)?;
            let res: APIResult<T> = serde_json::from_str(&output)?;
            if let Some(error) = res.error {
                Err(ApiError::KBErr(error))
            } else {
                Ok(res.result)
            }
        } else {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "Couldn't get stdin").into())
        }
    }

    pub fn listen_chat_api<T>() -> Result<
        (
            impl Stream<Item = T>,
            thread::JoinHandle<Result<(), ApiError>>,
        ),
        ApiError,
    >
    where
        T: DeserializeOwned + Send + 'static,
    {
        let mut child = KEYBASE.with(|kb| {
            Command::new(kb)
                .arg("chat")
                .arg("api-listen")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Couldn't run `keybase`")
        });
        if let Some(stdout) = child.stdout.take() {
            let (mut sender, receiver) = mpsc::channel::<T>(128);
            let handler: thread::JoinHandle<Result<(), ApiError>> = thread::spawn(move || {
                let mut reader = BufReader::new(stdout);
                loop {
                    let mut line = String::new();
                    let _bytes_written = reader.read_line(&mut line)?;
                    println!("got notif: {:?}", line);
                    let res: T = serde_json::from_str(&line)?;
                    sender.start_send(res)?;
                    // if let Some(error) = res.error {
                    //     let err = ApiError::KBErr(error);
                    //     println!("Error in listening: {:?}", &err);
                    //     return Err(err);
                    // } else {
                    //     println!("got notif: {:?}", line);
                    //     sender.start_send(res)?;
                    // }
                }
            });
            Ok((receiver, handler))
        } else {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "Couldn't get stdout").into())
        }
    }
}

#[derive(Debug)]
pub enum ApiError {
    Parsing(serde_json::error::Error),
    IOErr(io::Error),
    KBErr(KBError),
    UTF8Err(std::string::FromUtf8Error),
    ChannelErr(mpsc::SendError),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct KBError {
    pub code: i32,
    pub message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for ApiError {}

impl From<mpsc::SendError> for ApiError {
    fn from(error: mpsc::SendError) -> Self {
        ApiError::ChannelErr(error)
    }
}

impl From<std::string::FromUtf8Error> for ApiError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        ApiError::UTF8Err(error)
    }
}

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
