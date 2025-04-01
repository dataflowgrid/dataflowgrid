pub mod parser;
pub mod deserializer;
pub mod serializer;

use dataflowgrid_commons::orderedbag::OrderedBag;

#[derive(Debug, PartialEq)]
pub enum StreamableJSONEntry {
    Object(OrderedBag<StreamableJSONEntry, StreamableJSONEntry>), // key, value
    Array(Vec<StreamableJSONEntry>),
    String(String),
    Constant(String),
    Type(String, Vec<StreamableJSONEntry>),
}
