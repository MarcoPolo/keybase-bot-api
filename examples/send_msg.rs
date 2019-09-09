use keybase_bot_api::chat;

fn main() {
  let my_username = "marcopolo";
  let channel = chat::ChannelParams {
    name: format!("{},{}", my_username, "kb_monbot"),
    ..Default::default()
  };
  let msg = "Hello World";

  if let Err(e) = chat::send_msg(&channel, &msg) {
    println!("Failed to send message: {:?}", e);
  }
}
