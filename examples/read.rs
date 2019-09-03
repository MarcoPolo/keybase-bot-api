use rusty_keybase_bot::chat;
fn main() {
  let param = chat::ReadConvParams {
    channel: chat::ChannelParams {
      name: "marcopolo,pkt5,pkt9".into(),
      ..Default::default()
    }
  };

  let msgs = chat::read_conv(param).unwrap();
  println!("Msgs: {:?}", msgs);
}