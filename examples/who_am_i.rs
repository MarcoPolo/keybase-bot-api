use keybase_bot_api::{Bot, Status};
fn main() {
  let bot = Bot::new(
    "pkt0",
    option_env!("PAPERKEY").expect("Missing PAPERKEY env"),
  ).unwrap();
  println!("Username is: {:?}", bot.status().unwrap().username);
}
