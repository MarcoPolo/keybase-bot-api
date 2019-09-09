use keybase_bot_api::chat;
fn main() {
  let msgs = chat::listen().unwrap();
  println!("Done: {:?}", msgs);
}
