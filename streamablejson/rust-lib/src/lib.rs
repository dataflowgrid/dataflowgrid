pub mod parser;
pub mod deserializer;
pub mod serializer;

use dataflowgrid_commons::ordered_multi_dict::OrderedMultiDict;

#[derive(Debug, PartialEq)]
pub enum StreamableJSONEntry {
    Object(OrderedMultiDict<StreamableJSONEntry, StreamableJSONEntry>), // key, value
    Array(Vec<StreamableJSONEntry>),
    String(String),
    Constant(String),
    Type(String, Vec<StreamableJSONEntry>),
    Id(Option<Box<StreamableJSONEntry>>)
}
