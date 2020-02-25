use keybase_bot_api::{chat::ChannelParams, Bot, Chat};

fn main() {
    let bot = Bot::new(
        "rustybot",
        option_env!("KEYBASE_PAPERKEY").expect("Missing KEYBASE_PAPERKEY env"),
    )
    .unwrap();
    let channel = ChannelParams {
        name: format!("{},{}", bot.username, "marcopolo,pkt0"),
        ..Default::default()
    };
    let msg = "Hello World";

    if let Err(e) = bot.send_msg(&channel, &msg) {
        println!("Failed to send message: {:?}", e);
    }
}
