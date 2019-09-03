use rusty_keybase_bot::chat;
fn main() {
  let mut convs = chat::list().expect("Couldn't make an API call");
  convs
      .conversations
      .sort_unstable_by(|a, b| b.activeAt.cmp(&a.activeAt));
  let recent_conv_names: Vec<String> = convs
      .conversations
      .into_iter()
      .take(20)
      .map(|conv| {
          if !conv.channel.topicName.is_empty() {
              format!("{}#{}", conv.channel.name, conv.channel.topicName)
          } else {
              format!("{}", conv.channel.name)
          }
      })
      .collect();
  println!("Recent conversations: {:?}", recent_conv_names);
}