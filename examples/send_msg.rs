use rusty_keybase_bot::chat;

fn main() {
  let channel = chat::ChannelParams {
    name: "marcopolo,pkt0".into(),
    ..Default::default()
  };
  let msg = "Hello there";

  let _ = chat::send_msg(&channel, &msg).unwrap();
}
