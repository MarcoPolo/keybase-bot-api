use keybase_bot_api::chat;
use keybase_bot_api::status::status;

fn main() {
  let status = status().unwrap();
  let channel = chat::ChannelParams {
    name: format!("{},{}", status.username, "kb_monbot"),
    ..Default::default()
  };
  let msg = "Hello World";

  if let Err(e) = chat::send_msg(&channel, &msg) {
    println!("Failed to send message: {:?}", e);
  }
}
