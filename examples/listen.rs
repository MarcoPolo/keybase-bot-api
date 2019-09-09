use keybase_bot_api::{Chat, Bot};
use futures::stream::StreamExt;
use futures::executor::block_on;
use futures::prelude::*;

fn main() {
  let mut bot = Bot::new(
    "pkt0",
    option_env!("PAPERKEY").expect("Missing PAPERKEY env")
  )
  .unwrap();
  let notifs = bot.listen().unwrap();
    let fut = notifs.for_each(|notif| {
      println!("Got notif: {:?}", notif);
      future::ready(())
    });

  block_on(fut);
}
