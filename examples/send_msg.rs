use keybase_bot_api::{Chat, Bot, chat::ChannelParams};

fn main() {
  let bot = Bot::new(
    "pkt0",
    option_env!("PAPERKEY").expect("Missing PAPERKEY env")
  )
  .unwrap();
  let channel = ChannelParams {
    name: format!("{},{}", bot.username, "marcopolo"),
    ..Default::default()
  };
  let msg = "Hello World";

  if let Err(e) = bot.send_msg(&channel, &msg) {
    println!("Failed to send message: {:?}", e);
  }
}
