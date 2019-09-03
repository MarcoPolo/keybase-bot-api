use super::{ApiError, KBError};
use crate::keybase_cmd::{call_chat_api, APIResult};
use rusty_keybase_protocol::chat1::api;
use serde::{Deserialize, Serialize};
use serde_json;
// use std::io::Error as IOError;

#[derive(Serialize, Deserialize, Debug)]
pub struct APIRPC<T> {
  method: &'static str,
  params: Option<T>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OptionsOnly<T> {
  pub options: T,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReadConvParams {
  pub channel: ChannelParams,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ChannelParams {
  pub name: String,
  pub members_type: Option<String>,
  pub topic_name: Option<String>,
}

pub type ReadConv = APIRPC<OptionsOnly<ReadConvParams>>;

const LISTMETHOD: APIRPC<()> = APIRPC {
  method: "list",
  params: None,
};

#[derive(Deserialize, Serialize)]
pub struct ListResult {
  #[serde(rename = "conversations")]
  pub conversations: Vec<api::ConvSummary>,
}

//List inbox
pub fn list() -> Result<ListResult, ApiError> {
  let input = serde_json::to_vec(&LISTMETHOD)?;
  call_chat_api::<ListResult>(&input)
}

// Read a conversation:
//     {"method": "read", "params": {"options": {"channel": {"name": "you,them"}}}}

pub fn read_conv(options: ReadConvParams) -> Result<api::Thread, ApiError> {
  let input: ReadConv = APIRPC {
    method: "read",
    params: Some(OptionsOnly { options }),
  };
  println!("opts: {}", &serde_json::to_string(&input)?);
  call_chat_api::<api::Thread>(&serde_json::to_vec(&input)?)
}

pub fn listen_for_new_msgs() -> Result<(), ApiError> {
  Ok(())

}
