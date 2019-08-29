use pest::Parser;
use avdl_parser::{Rule, AVDLParser};
use std::fs;

fn main() {
  println!("Hello!");
  let input = fs::read_to_string("keybase-protocol/chat1/common.avdl")
    .expect("Something went wrong reading the file");
  let parsed = AVDLParser::parse(Rule::avdl_protocol, &input);
  if let Err(e) = parsed {
    println!("{}", e);
  } else {
    println!("{:#?}", parsed.unwrap().next().unwrap());
  }
}

fn convert_avdl_into_rust() {

}