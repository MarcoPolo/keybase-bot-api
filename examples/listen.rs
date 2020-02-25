use async_std::prelude::*;
use keybase_bot_api::{Bot, Chat};

fn main() {
    async_std::task::block_on( async{
    let mut bot = Bot::new(
        "mmou",
        option_env!("KEYBASE_PAPERKEY").expect("Missing KEYBASE_PAPERKEY env"),
    )
    .unwrap();
    let mut notifs = bot.listen().unwrap();
    loop {
        let n = notifs.next().await;
        if let Some(notif) = n {
            println!("Got notif: {:?}", notif);
        }
        else {
            println!("Channel sender dropped");
            return
        }
    } })
}
