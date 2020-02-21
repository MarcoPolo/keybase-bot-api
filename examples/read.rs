use keybase_bot_api::{chat, Bot, Chat};
fn main() {
    let bot = Bot::new(
        "pkt0",
        option_env!("KEYBASE_PAPERKEY").expect("Missing KEYBASE_PAPERKEY env"),
    )
    .unwrap();
    let channel = chat::ChannelParams {
        name: "marcopolo,pkt0".into(),
        ..Default::default()
    };

    let msgs = bot.read_conv(&channel).unwrap();
    println!("Msgs: {:?}", msgs);
}
