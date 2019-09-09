pub mod bot;
pub mod chat;
pub mod status;
pub use bot::Bot;
pub use chat::Chat;
use futures::channel::mpsc;
use serde::{Deserialize, Serialize};
pub use status::Status;
use std::error::Error;
use std::{fmt, io};

pub(crate) mod keybase_cmd {
    use super::{ApiError, KBError};
    use futures::{channel::mpsc, stream::Stream};
    use serde::{de::DeserializeOwned, Deserialize, Serialize};
    use serde_json;
    use std::io::{self, BufRead, BufReader, Write};
    use std::path::{Path, PathBuf};
    use std::process::{Child, Command, Stdio};
    use std::thread;

    thread_local! {
        pub static KEYBASE: PathBuf = which_keybase();
    }

    #[derive(Deserialize, Serialize)]
    pub struct APIResult<T> {
        pub result: Option<T>,
        pub error: Option<KBError>,
    }

    pub fn which_keybase() -> PathBuf {
        let path = String::from_utf8(
            Command::new("which")
                .arg("keybase")
                .output()
                .expect("which is not installed")
                .stdout,
        )
        .expect("Output not in UTF-8");
        Path::new(path.trim()).to_path_buf()
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct StatusRes {
        #[serde(rename = "Username")]
        pub username: String,
    }

    pub fn call_status(keybase_path: &Path, home_dir: &Path) -> Result<StatusRes, ApiError> {
        let child = keybase_exec(keybase_path, home_dir, &["status", "-j"])?;
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

    fn keybase_exec<I, S>(keybase_path: &Path, home_dir: &Path, args: I) -> Result<Child, io::Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        Command::new(keybase_path)
            .arg("--home")
            .arg(home_dir.to_str().unwrap())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
    }

    pub fn call_chat_api<T>(
        keybase_path: &Path,
        home_dir: &Path,
        input: &[u8],
    ) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let mut child = keybase_exec(keybase_path, home_dir, &["chat", "api"])?;
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
                Ok(res.result.expect("Missing result from api call"))
            }
        } else {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "Couldn't get stdin").into())
        }
    }

    pub fn login_oneshot(
        keybase_path: &Path,
        home_dir: &Path,
        username: &str,
        paperkey: &str,
    ) -> Result<(), ApiError> {
        let mut child = keybase_exec(keybase_path, home_dir, &["oneshot", "--username", username])?;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(paperkey.as_bytes())?;
            let output = child.wait_with_output()?;
            if !output.status.success() {
                println!(
                    "err in login: {:?} home_dir: {:?} exitCode: {:?} stdout:{:?} stderr:{:?}",
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Keybase did not return successful exit code",
                    ),
                    home_dir,
                    output.status.code(),
                    String::from_utf8(output.stdout),
                    String::from_utf8(output.stderr)
                );
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Keybase did not return successful exit code",
                )
                .into())
            } else {
                Ok(())
            }
        } else {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "Couldn't get stdin").into())
        }
    }

    pub fn listen_chat_api<T>(
        keybase_path: &Path,
        home_dir: &Path,
    ) -> Result<
        (
            impl Stream<Item = T>,
            thread::JoinHandle<Result<(), ApiError>>,
        ),
        ApiError,
    >
    where
        T: DeserializeOwned + Send + 'static,
    {
        let mut child = keybase_exec(keybase_path, home_dir, &["chat", "api-listen"])?;

        if let Some(stdout) = child.stdout.take() {
            let (mut sender, receiver) = mpsc::channel::<T>(128);
            let handler: thread::JoinHandle<Result<(), ApiError>> = thread::spawn(move || {
                let mut reader = BufReader::new(stdout);
                loop {
                    let mut line = String::new();
                    let _bytes_written = reader.read_line(&mut line)?;
                    let res: T = serde_json::from_str(&line)?;
                    sender.start_send(res)?;
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
        println!("Keybase is at: {:?}", which_keybase());
        assert!(!which_keybase().to_str().unwrap().is_empty());
    }

    #[test]
    fn ls_inbox() {
        // list().unwrap();
    }
}
