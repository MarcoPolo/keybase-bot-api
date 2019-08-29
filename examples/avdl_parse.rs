#![allow(dead_code)]
use avdl_parser::{AVDLParser, Rule};
use pest::{
  iterators::{Pair, Pairs},
  Parser,
};
use std::{
  error::Error,
  fs,
  io::{self, Write},
  str::FromStr,
};

// @namespace("keybase.1")
fn main() {
  let input = fs::read_to_string("keybase-protocol/chat1/common.avdl")
    .expect("Something went wrong reading the file");
  let parsed = AVDLParser::parse(Rule::avdl_protocol, &input);
  if let Err(e) = parsed {
    println!("{}", e);
  } else {
    let mut output = Vec::new();
    // let mut output = File::create("tmp.rs").expect("Failed to create protocol file");
    build_rust_code_from_avdl(parsed.unwrap(), &mut output).unwrap();
    println!("Output is:\n{}", String::from_utf8(output).unwrap());
  }
}

fn convert_path_str_to_rust_mod(path: &str, as_name: &str) -> String {
  let path = String::from(path);
  let parts: Vec<&str> = path.split("/").skip_while(|s| s != &"protocol").collect();

  let is_as_name_same = parts.last().unwrap() == &as_name;

  let mut module = parts
    .into_iter()
    .fold(String::from("crate"), |mut s, part| {
      s.push_str("::");
      s.push_str(part);
      s
    });

  if !is_as_name_same {
    module.push_str(" as ");
    module.push_str(as_name);
  }
  module.push_str(";");

  module
}

fn convert_to_import<W>(w: &mut W, p: Pair<Rule>) -> Result<(), Box<dyn Error>>
where
  W: Write,
{
  assert_eq!(p.as_rule(), Rule::import);
  let parts = p.into_inner().take(3).collect::<Vec<Pair<Rule>>>();
  let (kind, path, as_name) = (&parts[0], &parts[1], &parts[2]);
  match kind.as_str() {
    "idl" => {
      write!(
        w,
        "use {}",
        convert_path_str_to_rust_mod(path.as_str(), as_name.as_str())
      )?;
    }
    _ => panic!("Unhandled import kind (not idl)"),
  }

  Ok(())
}

fn convert_idl_type_to_rust_type(idl_type: &str) -> String {
  match idl_type {
    "bytes" => String::from("[u8]"),
    "uint64" => String::from("u64"),
    "uint32" => String::from("u32"),
    _ => idl_type.into(),
  }
}

fn convert_dot_to_sep(a: &str) -> String {
  a.split(".").collect::<Vec<&str>>().join("::")
}

// These are basically already rust enums, copy/paste instead
// struct AVDLEnum {
//   enum_name: String,
//   enum_fields: Vec<String>,
// }

// impl<'a> From<Pair<'a, Rule>> for AVDLEnum {
//   fn from(pair: Pair<'a, Rule>) -> Self {
//     assert_eq!(pair.as_rule(), Rule::enum_ty);
//     panic!();
//   }
// }

enum AVDLType {
  Simple(String),
  Maybe(String),
  Union(),
}

impl ToString for AVDLType {
  fn to_string(&self) -> String {
    match self {
      AVDLType::Simple(s) => s.clone(),
      AVDLType::Maybe(s) => format!("Option<{}>", s),
      AVDLType::Union() => panic!("Not implemented"),
    }
  }
}

impl<'a> From<Pair<'a, Rule>> for AVDLType {
  fn from(pair: Pair<'a, Rule>) -> Self {
    assert!(
      pair.as_rule() == Rule::ty || pair.as_rule() == Rule::maybe_ty,
      "Unexpected rule: {:?}",
      pair.as_rule()
    );

    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
      Rule::simple_ty => AVDLType::Simple(convert_dot_to_sep(&convert_idl_type_to_rust_type(
        inner.as_str(),
      ))),
      Rule::maybe_ty => {
        let inner_ty: AVDLType = inner.into();
        AVDLType::Maybe(inner_ty.to_string())
      }
      _ => panic!("Unhandled case: {:?}", inner.as_rule()),
    }
  }
}

fn convert_typedef<W>(w: &mut W, p: Pair<Rule>) -> Result<(), Box<dyn Error>>
where
  W: Write,
{
  assert_eq!(p.as_rule(), Rule::typedef);
  let mut parts = p.into_inner();
  let mut type_name: Option<String> = None;
  let mut type_target: Option<String> = None;
  while let Some(pair) = parts.next() {
    match pair.as_rule() {
      Rule::ty => {
        let ty: AVDLType = pair.into();
        type_target = Some(ty.to_string());
      }
      Rule::lint => {
        write!(w, "// LINT: {}", pair.as_str())?;
      }
      Rule::record => type_name = Some(pair.into_inner().next().unwrap().as_str().into()),
      _ => unreachable!(),
    }
  }

  write!(w, "type {} = {};", type_name.unwrap(), type_target.unwrap())?;

  Ok(())
}

fn convert_enum<W>(w: &mut W, p: Pair<Rule>) -> Result<(), Box<dyn Error>>
where
  W: Write,
{
  assert_eq!(p.as_rule(), Rule::enum_ty);
  // This is actually already a rust enum!
  write!(w, "{}", p.as_str())?;
  Ok(())
}

// Turns FOO -> Foo
fn quiet_voice(s: &str) -> String {
  s.chars()
    .enumerate()
    .map(|(i, c)| {
      if i == 0 {
        c.to_ascii_uppercase()
      } else {
        c.to_ascii_lowercase()
      }
    })
    .collect()
}

struct VariantCaseTy {
  enum_name: String,
  enum_inner_ty: String,
}

impl<'a> From<Pair<'a, Rule>> for VariantCaseTy {
  fn from(pair: Pair<'a, Rule>) -> Self {
    let mut parts = pair.into_inner();
    let mut enum_name: Option<String> = None;
    let mut enum_inner_ty: Option<AVDLType> = None;
    while let Some(pair) = parts.next() {
      match pair.as_rule() {
        Rule::ident => enum_name = Some(quiet_voice(pair.as_str())),
        Rule::ty => enum_inner_ty = Some(pair.into()),
        _ => unreachable!(),
      }
    }

    VariantCaseTy {
      enum_name: enum_name.unwrap(),
      enum_inner_ty: enum_inner_ty.unwrap().to_string(),
    }
  }
}

impl WriteTo for VariantCaseTy {
  fn write_to<W: Write>(&self, w: &mut W) -> Result<(), io::Error> {
    write!(w, "  {}({}),\n", self.enum_name, self.enum_inner_ty)?;
    Ok(())
  }
}

struct VariantTy {
  ident: String,
  cases: Vec<VariantCaseTy>,
}

impl<'a> From<Pair<'a, Rule>> for VariantTy {
  fn from(pair: Pair<'a, Rule>) -> Self {
    let mut parts = pair.into_inner();
    let mut ident: Option<String> = None;
    let mut cases: Vec<VariantCaseTy> = vec![];
    while let Some(pair) = parts.next() {
      match pair.as_rule() {
        Rule::ident => ident = Some(pair.as_str().into()),
        Rule::variant_case => cases.push(pair.into()),
        Rule::variant_param => {}
        _ => unreachable!(),
      }
    }

    VariantTy {
      ident: ident.expect("Couldn't find name for variant"),
      cases,
    }
  }
}

impl WriteTo for VariantTy {
  fn write_to<W: Write>(&self, w: &mut W) -> Result<(), io::Error> {
    write!(w, "enum {} {{\n", self.ident)?;
    for case in self.cases.iter() {
      case.write_to(w)?;
    }
    write!(w, "}}")?;
    Ok(())
  }
}

fn convert_variant<W>(w: &mut W, p: Pair<Rule>) -> Result<(), Box<dyn Error>>
where
  W: Write,
{
  assert_eq!(p.as_rule(), Rule::variant_ty);
  let ty: VariantTy = p.into();
  ty.write_to(w)?;
  Ok(())
}

fn convert_fixed<W>(w: &mut W, p: Pair<Rule>) -> Result<(), Box<dyn Error>>
where
  W: Write,
{
  assert_eq!(p.as_rule(), Rule::fixed_ty);
  let mut parts = p.into_inner();
  let mut ty: Option<AVDLType> = None;
  let mut byte_size: usize = 0;
  while let Some(pair) = parts.next() {
    match pair.as_rule() {
      Rule::ty => ty = Some(pair.into()),
      Rule::byte_size => {
        byte_size = usize::from_str(pair.as_str()).expect("Couldn't parse byte_size")
      }
      _ => unreachable!(),
    }
  }

  write!(w, "type {} = [u8;{}];", ty.unwrap().to_string(), byte_size)?;
  Ok(())
}

struct AVDLRecordProp {
  ty: String,
  field: String,
  attributes: Vec<String>,
}

pub trait WriteTo {
  fn write_to<W: Write>(&self, w: &mut W) -> Result<(), io::Error>;
}

impl WriteTo for AVDLRecordProp {
  fn write_to<W: Write>(&self, w: &mut W) -> Result<(), io::Error> {
    for attr in self.attributes.iter() {
      write!(w, "// {}\n", attr)?;
    }
    write!(w, "{}: {},\n", self.field, self.ty)?;
    Ok(())
  }
}

impl<'a> From<Pair<'a, Rule>> for AVDLRecordProp {
  fn from(pair: Pair<'a, Rule>) -> Self {
    assert_eq!(pair.as_rule(), Rule::record_prop);
    let mut ty: Option<AVDLType> = None;
    let mut field: Option<String> = None;
    let mut attributes = vec![];
    let mut parts = pair.into_inner();
    while let Some(pair) = parts.next() {
      match pair.as_rule() {
        Rule::lint | Rule::generic_annotation => attributes.push(pair.as_str().into()),
        Rule::ty => ty = Some(pair.into()),
        Rule::ident => field = Some(pair.as_str().into()),
        _ => panic!("Unhandled case: {:?}", pair),
      }
    }

    AVDLRecordProp {
      ty: ty.expect("Couldn't find types").to_string(),
      field: field.expect("Couldn't find field"),
      attributes,
    }
  }
}

fn convert_record<W>(w: &mut W, p: Pair<Rule>) -> Result<(), Box<dyn Error>>
where
  W: Write,
{
  assert_eq!(p.as_rule(), Rule::record);
  let mut parts = p.into_inner();
  let mut type_name: Option<AVDLType> = None;
  let mut record_props: Vec<AVDLRecordProp> = vec![];

  while let Some(pair) = parts.next() {
    match pair.as_rule() {
      Rule::ty => type_name = Some(pair.into()),
      Rule::comment => write!(w, "// {}", pair.as_str())?,
      Rule::record_prop => {
        record_props.push(pair.into());
      }
      _ => panic!("Unhandled case: {:?}", pair),
    }
  }

  write!(
    w,
    "struct {} {{\n",
    type_name.expect("No Record name").to_string()
  )?;
  for prop in record_props.into_iter() {
    prop.write_to(w)?;
  }
  write!(w, "}}")?;

  Ok(())
}

fn build_rust_code_from_avdl<W>(mut input: Pairs<Rule>, w: &mut W) -> Result<(), Box<dyn Error>>
where
  W: Write,
{
  for (i, node) in input
    .next()
    .expect("Nothing to parse")
    .into_inner()
    .enumerate()
  {
    if i > 3 {
      return Ok(());
    }
    match node.as_rule() {
      Rule::namespace_annotation => {
        if let Some(n) = node.into_inner().next() {
          match n.as_rule() {
            Rule::namespace_name => write!(w, "Namespace: {:?}", n.as_str())?,
            _ => unreachable!(),
          }
        }
      }
      Rule::protocol => {
        let mut inner = node.into_inner();
        while let Some(n) = inner.next() {
          match n.as_rule() {
            Rule::protocol_name => write!(w, "Protocol: {:?}\n", n.as_str())?,
            Rule::protocol_body => {
              let mut inner = n.into_inner();
              while let Some(protocol_body_node) = inner.next() {
                match protocol_body_node.as_rule() {
                  Rule::comment => write!(w, "{}", protocol_body_node.as_str())?,
                  Rule::import => convert_to_import(w, protocol_body_node)?,
                  Rule::typedef => convert_typedef(w, protocol_body_node)?,
                  _ => {}
                }
              }
            }
            _ => unreachable!(),
          }
          write!(w, "\n")?;
        }
      }
      _ => unreachable!(),
    }
    println!("-----------------");
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn test_conversion<F>(
    r: Rule,
    conversion_fn: F,
    input: &str,
    expected: &str,
  ) -> Result<(), Box<dyn Error>>
  where
    F: Fn(&mut Vec<u8>, Pair<Rule>) -> Result<(), Box<dyn Error>>,
  {
    let mut output = vec![];
    conversion_fn(&mut output, AVDLParser::parse(r, input)?.next().unwrap())?;
    assert_eq!(String::from_utf8(output).unwrap(), expected);
    Ok(())
  }

  #[test]
  fn test_convert_import() -> Result<(), Box<dyn Error>> {
    test_conversion(
      Rule::import,
      convert_to_import,
      r#"import idl "github.com/keybase/client/go/protocol/gregor1" as gregor1;"#,
      "use crate::protocol::gregor1;",
    )
    .unwrap();
    test_conversion(
      Rule::import,
      convert_to_import,
      r#"import idl "github.com/keybase/client/go/protocol/gregor1" as otherGregor;"#,
      "use crate::protocol::gregor1 as otherGregor;",
    )
    .unwrap();

    Ok(())
  }

  #[test]
  fn test_typedef() {
    test_conversion(
      Rule::typedef,
      convert_typedef,
      r#"@typedef("bytes")  record ThreadID {}"#,
      "type ThreadID = [u8];",
    )
    .unwrap();
  }

  #[test]
  fn test_enum() {
    test_conversion(
      Rule::enum_ty,
      convert_enum,
      r#"enum RetentionPolicyType {
  NONE_0,
  RETAIN_1, // Keep messages forever
  EXPIRE_2, // Delete after a while
  INHERIT_3, // Use the team's policy
  EPHEMERAL_4 // Force all messages to be exploding.
}"#,
      "enum RetentionPolicyType {
  NONE_0,
  RETAIN_1, // Keep messages forever
  EXPIRE_2, // Delete after a while
  INHERIT_3, // Use the team's policy
  EPHEMERAL_4 // Force all messages to be exploding.
}",
    )
    .unwrap();
  }

  #[test]
  fn test_record() {
    test_conversion(
      Rule::record,
      convert_record,
      r#"record InboxVersInfo {
  gregor1.UID uid;
  InboxVers vers;
}"#,
      r#"struct InboxVersInfo {
uid: gregor1::UID,
vers: InboxVers,
}"#,
    )
    .unwrap();

    test_conversion(
      Rule::record,
      convert_record,
      r#"record InboxVersInfo {
    @mpackkey("b") @jsonkey("b")
    union { null, gregor1.UID } botUID;
    @mpackkey("c") @jsonkey("c")
    InboxVers vers;
}"#,
      r#"struct InboxVersInfo {
// @mpackkey("b")
// @jsonkey("b")
botUID: Option<gregor1::UID>,
// @mpackkey("c")
// @jsonkey("c")
vers: InboxVers,
}"#,
    )
    .unwrap();
  }

  #[test]
  fn test_fixed() {
    test_conversion(
      Rule::fixed_ty,
      convert_fixed,
      r#"fixed Bytes32(32);"#,
      "type Bytes32 = [u8;32];",
    )
    .unwrap();
  }

  #[test]
  fn test_variant() {
    test_conversion(
      Rule::variant_ty,
      convert_variant,
      r#"variant AssetMetadata switch (AssetMetadataType assetType) {
  case IMAGE: AssetMetadataImage;
  case VIDEO: AssetMetadataVideo;
  case AUDIO: AssetMetadataAudio;
}"#,
      "enum AssetMetadata {
  Image(AssetMetadataImage),
  Video(AssetMetadataVideo),
  Audio(AssetMetadataAudio),
}",
    )
    .unwrap();
  }

}
