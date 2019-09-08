use rusty_keybase_bot::chat;
fn main() {
  let channel = chat::ChannelParams {
    name: "marcopolo,pkt0".into(),
    ..Default::default()
  };

  let msgs = chat::read_conv(&channel).unwrap();
  println!("Msgs: {:?}", msgs);
}