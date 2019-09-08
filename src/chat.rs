use super::{ApiError, KBError};
use crate::keybase_cmd::{call_chat_api, listen_chat_api, APIResult};
use futures::executor::LocalPool;
use futures::{
  executor::block_on,
  future,
  stream::{self, StreamExt},
};
use rusty_keybase_protocol::chat1::api;
use rusty_keybase_protocol::stellar1;
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

#[derive(Serialize, Debug)]
pub struct ReadConvParams<'a> {
  pub channel: &'a ChannelParams,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct ChannelParams {
  pub name: String,
  pub members_type: Option<String>,
  pub topic_name: Option<String>,
}

pub type ReadConv<'a> = APIRPC<OptionsOnly<ReadConvParams<'a>>>;

const LISTMETHOD: APIRPC<()> = APIRPC {
  method: "list",
  params: None,
};

#[derive(Deserialize, Serialize)]
pub struct ListResult {
  pub conversations: Vec<api::ConvSummary>,
}

//List inbox
pub fn list() -> Result<ListResult, ApiError> {
  let input = serde_json::to_vec(&LISTMETHOD)?;
  call_chat_api::<ListResult>(&input)
}

// Read a conversation:
//     {"method": "read", "params": {"options": {"channel": {"name": "you,them"}}}}

pub fn read_conv(channel: &ChannelParams) -> Result<api::Thread, ApiError> {
  let input: ReadConv = APIRPC {
    method: "read",
    params: Some(OptionsOnly { options: ReadConvParams { channel }}),
  };
  println!("opts: {}", &serde_json::to_string(&input)?);
  call_chat_api::<api::Thread>(&serde_json::to_vec(&input)?)
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
enum Notification {
  #[serde(rename = "chat")]
  Chat(api::MsgNotification),
  #[serde(rename = "wallet")]
  Wallet {
    notification: stellar1::PaymentDetailsLocal,
  },
}

pub fn listen() -> Result<(), ApiError> {
  let (notif_stream, handler) = listen_chat_api::<Notification>()?;
  let fut = notif_stream.for_each(|notif| {
    println!("Got notif: {:?}", notif);
    future::ready(())
  });
  block_on(fut);
  handler.join().unwrap()
}

#[derive(Serialize, Debug)]
struct MessageOptions<'a> {
  body: &'a str
}

#[derive(Serialize, Debug)]
struct SendMessageOptions<'a> {
  channel: &'a ChannelParams,
  message: MessageOptions<'a>,
}
pub type SendTextRPC<'a> = APIRPC<OptionsOnly<SendMessageOptions<'a>>>;

pub fn send_msg<'a>(channel: &'a ChannelParams, msg: &'a str) -> Result<api::SendRes, ApiError> {
  let options = SendMessageOptions {
    channel,
    message: MessageOptions {
      body: msg
    }
  };
  let input: SendTextRPC = APIRPC {
    method: "send",
    params: Some(OptionsOnly {
      options
    })
  };
  call_chat_api::<api::SendRes>(&serde_json::to_vec(&input)?)
}