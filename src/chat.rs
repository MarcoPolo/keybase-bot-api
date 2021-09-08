use crate::bot::Bot;
use crate::keybase_cmd::{call_chat_api, listen_chat_api};
use crate::ApiError;
use async_std::channel::Receiver;
use async_std::task::JoinHandle;
use keybase_protocol::chat1::api;
use keybase_protocol::stellar1;
use serde::{Deserialize, Serialize};
use serde_json;

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

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
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

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "type")]
pub enum Notification {
    #[serde(rename = "chat")]
    Chat(api::MsgNotification),
    #[serde(rename = "wallet")]
    Wallet {
        notification: stellar1::PaymentDetailsLocal,
    },
}

#[derive(Serialize, Debug)]
struct MessageOptions<'a> {
    body: &'a str,
}

#[derive(Serialize, Debug)]
struct SendMessageOptions<'a> {
    channel: &'a ChannelParams,
    message: MessageOptions<'a>,
}

#[derive(Serialize, Debug)]
struct UploadAttachmentOptions<'a> {
    channel: &'a ChannelParams,
    filename: &'a str,
    title: &'a str,
}

type SendTextRPC<'a> = APIRPC<OptionsOnly<SendMessageOptions<'a>>>;
type UploadAttachmentRPC<'a> = APIRPC<OptionsOnly<UploadAttachmentOptions<'a>>>;

pub trait Chat {
    fn send_msg<'a>(
        &self,
        channel: &'a ChannelParams,
        msg: &'a str,
    ) -> Result<api::SendRes, ApiError>;
    fn upload_attachment<'a>(
        &self,
        channel: &'a ChannelParams,
        filename: &'a str,
        title: &'a str,
    ) -> Result<api::SendRes, ApiError>;
    fn listen(&mut self) -> Result<Receiver<Result<Notification, ApiError>>, ApiError>;
    fn list(&self) -> Result<ListResult, ApiError>;
    fn read_conv(&self, channel: &ChannelParams) -> Result<api::Thread, ApiError>;
}

impl Chat for Bot {
    // Send a message:
    // {"method": "send", "params": {"options": {"channel": {"name": "you,them"}, "message": {"body": "is it cold today?"}}}}
    fn send_msg<'a>(
        &self,
        channel: &'a ChannelParams,
        msg: &'a str,
    ) -> Result<api::SendRes, ApiError> {
        let options = SendMessageOptions {
            channel,
            message: MessageOptions { body: msg },
        };
        let input: SendTextRPC = APIRPC {
            method: "send",
            params: Some(OptionsOnly { options }),
        };
        call_chat_api::<api::SendRes>(
            self.keybase_path.as_path(),
            self.home_dir.as_path(),
            &serde_json::to_vec(&input)?,
        )
    }

    // Upload an attachment:
    // {"method": "attach", "params": {"options": {"channel": {"name": "you,them"}, "filename": "photo.jpg", "title": "Sunset last night"}}}
    fn upload_attachment<'a>(
        &self,
        channel: &'a ChannelParams,
        filename: &'a str,
        title: &'a str,
    ) -> Result<api::SendRes, ApiError> {
        let options = UploadAttachmentOptions {
            channel,
            filename,
            title,
        };

        let input: UploadAttachmentRPC = APIRPC {
            method: "attach",
            params: Some(OptionsOnly { options }),
        };

        call_chat_api::<api::SendRes>(
            self.keybase_path.as_path(),
            self.home_dir.as_path(),
            &serde_json::to_vec(&input)?,
        )
    }

    fn listen(&mut self) -> Result<Receiver<Result<Notification, ApiError>>, ApiError> {
        let (notif_stream, handle): (Receiver<Result<Notification, ApiError>>, JoinHandle<()>) =
            listen_chat_api::<Notification>(self.keybase_path.as_path(), self.home_dir.as_path())?;
        self.listen_threads.push(handle);
        Ok(notif_stream)
    }

    fn list(&self) -> Result<ListResult, ApiError> {
        let input = serde_json::to_vec(&LISTMETHOD)?;
        call_chat_api::<ListResult>(self.keybase_path.as_path(), self.home_dir.as_path(), &input)
    }

    fn read_conv(&self, channel: &ChannelParams) -> Result<api::Thread, ApiError> {
        let input: ReadConv = APIRPC {
            method: "read",
            params: Some(OptionsOnly {
                options: ReadConvParams { channel },
            }),
        };
        println!("opts: {}", &serde_json::to_string(&input)?);
        call_chat_api::<api::Thread>(
            self.keybase_path.as_path(),
            self.home_dir.as_path(),
            &serde_json::to_vec(&input)?,
        )
    }
}
