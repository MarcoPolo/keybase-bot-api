use super::ApiError;
use crate::keybase_cmd;
use hex;
use rand::prelude::*;
use std::{
  env, fmt, fs,
  io::{self, Read},
  path::{Path, PathBuf},
  process::{self, Command, Stdio},
  thread::JoinHandle,
};

pub struct Bot {
  pub username: String,
  pub keybase_path: PathBuf,
  pub home_dir: PathBuf,
  service_process: process::Child,
  pub listen_threads: Vec<JoinHandle<Result<(), ApiError>>>,
}

impl fmt::Debug for Bot {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "Bot {{ username: {}, home_dir: {:?}, listen_threads: {} }}",
      self.username,
      self.home_dir,
      self.listen_threads.len()
    )
  }
}

// TODO
// 1. Copy binary to working dir
// 2. Login

#[derive(Debug)]
pub enum BotError {
  IO(io::Error),
  ApiErr(ApiError),
}

impl From<io::Error> for BotError {
  fn from(err: io::Error) -> Self {
    BotError::IO(err)
  }
}

impl From<ApiError> for BotError {
  fn from(err: ApiError) -> Self {
    BotError::ApiErr(err)
  }
}

fn start_service(keybase_path: &Path, home_dir: &Path) -> Result<process::Child, io::Error> {
  let mut child = Command::new(keybase_path)
    .arg("--home")
    .arg(home_dir.to_str().unwrap())
    .arg("service")
    .stderr(Stdio::piped())
    .spawn()?;

  let stderr = child.stderr.as_mut().expect("No stderr on service");
  let mut buf = vec![0u8];
  stderr.read_exact(&mut buf)?;
  Ok(child)
}

impl Bot {
  pub fn new<S>(username: S, paperkey: S) -> Result<Bot, BotError>
  where
    S: Into<String>,
  {
    let username: String = username.into();
    let paperkey: String = paperkey.into();
    let mut working_dir = env::temp_dir();
    let mut rng = rand::thread_rng();
    let mut hex_id = vec![0u8; 16];
    rng.fill_bytes(&mut hex_id);
    let hex_id = hex::encode(hex_id);

    // Tempory folder for bot
    working_dir.push(format!("keybase_bot_{}", hex_id));
    fs::create_dir(&working_dir)?;

    // Copy keybase path
    let mut keybase_path = working_dir.clone();
    keybase_path.push("keybase");
    fs::copy(keybase_cmd::which_keybase(), &keybase_path)?;

    // Start KB service
    let mut service_process = start_service(&keybase_path, &working_dir)?;

    // Login
    if let Err(e) = keybase_cmd::login_oneshot(&keybase_path, &working_dir, &username, &paperkey) {
      println!("Login failed");
      service_process.kill()?;
      return Err(e.into())
    }

    Ok(Bot {
      username,
      keybase_path,
      home_dir: working_dir,
      service_process,
      listen_threads: vec![],
    })
  }
}

impl Drop for Bot {
  fn drop(&mut self) {
    self.service_process.kill().expect("Failed to stop Keybase service");
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::chat::{self, Chat};

  #[test]
  fn start_bot() {
    let bot = Bot::new(
      "pkt0",
      option_env!("PAPERKEY").expect("Missing PAPERKEY env"),
    )
    .unwrap();
    println!("Bot is {:?}", bot);

    let channel = chat::ChannelParams {
      name: format!("{},{}", "pkt0", "marcopolo"),
      ..Default::default()
    };
    let res = bot
      .send_msg(&channel, "hello world")
      .expect("Couldn't send message");
    println!("Result is {:?}", res);
  }

}
