use pest::Parser;

#[derive(Parser)]
#[grammar = "./avdl/avdl.pest"]
pub struct AVDLParser;
