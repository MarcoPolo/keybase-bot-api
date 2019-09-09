use keybase_bot_api::{Chat, Bot};
fn main() {
  let bot = Bot::new(
    "pkt0",
    option_env!("PAPERKEY").expect("Missing PAPERKEY env")
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
          let topic_name  = channel.topicName.unwrap();
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