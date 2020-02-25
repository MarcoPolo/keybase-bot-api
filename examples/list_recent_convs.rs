use keybase_bot_api::{Bot, Chat};
fn main() {
    let bot = Bot::new(
        "pkt0",
        option_env!("KEYBASE_PAPERKEY").expect("Missing KEYBASE_PAPERKEY env"),
    )
    .unwrap();
    let mut convs = bot.list().expect("Couldn't make an API call");
    convs
        .conversations
        .sort_unstable_by(|a, b| b.activeAt.cmp(&a.activeAt));
    let recent_conv_names: Vec<String> = convs
        .conversations
        .into_iter()
        .take(20)
        .map(|conv| {
            let channel = conv.channel.unwrap();
            let topic_name = channel.topicName.unwrap();
            let channel_name = channel.name.unwrap();
            if !topic_name.is_empty() {
                format!("{}#{}", channel_name, topic_name)
            } else {
                format!("{}", channel_name)
            }
        })
        .collect();
    println!("Recent conversations: {:?}", recent_conv_names);
}
