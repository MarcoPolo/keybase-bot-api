use rusty_keybase_bot::chat;
fn main() {
  let msgs = chat::listen().unwrap();
  println!("Done: {:?}", msgs);
}
