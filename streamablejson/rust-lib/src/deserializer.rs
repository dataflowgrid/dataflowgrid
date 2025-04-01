/* This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich */
#![allow(dead_code)]
use std::str::Chars;

use dataflowgrid_commons::orderedbag::OrderedBag;
use dataflowgrid_commons::readers::reader::IteratorReadable;
use crate::parser::StreamableJSONReader;

use crate::parser::{StreamableJSONReaderCallback, StreamableJSONReaderCallbackReturn, StreamableJSONReaderError, StreamableJSONReaderEvent} ;
use crate::StreamableJSONEntry;

pub struct OrderedBagDeserializer {
    stack: Vec<StreamableJSONEntry>,
    results: Vec<StreamableJSONEntry>,
}

impl OrderedBagDeserializer {
    pub fn new() -> Self {
        OrderedBagDeserializer {
            stack: Vec::new(),
            results: Vec::new(),
        }
    }
    pub fn result(&mut self) -> Option<StreamableJSONEntry> {
        if self.results.is_empty() {
            None
        } else {
            Some(self.results.remove(0))
        }
    }

    #[inline]
    fn add_to_last_stack_element(&mut self, entry: StreamableJSONEntry) {
        if self.stack.is_empty() {
            self.results.push(entry);
        } else {
            let last = self.stack.last_mut().unwrap();
            match last {
                StreamableJSONEntry::Array(last) => {
                    last.push(entry);
                }
                StreamableJSONEntry::Object(opt) => {
                    opt.push(StreamableJSONEntry::String("1".into()), entry);
                }
                StreamableJSONEntry::Type(_, attr) => {
                    attr.push(entry);
                }
                _ => panic!("Expected array, object or id, but was {:?}", last),
            }
        }
    }

}

impl StreamableJSONReaderCallback for OrderedBagDeserializer {
    fn on_streamablejson_event(&mut self, event: StreamableJSONReaderEvent) -> StreamableJSONReaderCallbackReturn {
        match event {
            StreamableJSONReaderEvent::StartObject => {
                        self.stack.push(StreamableJSONEntry::Object(OrderedBag::new()));
                    }
            StreamableJSONReaderEvent::EndObject | StreamableJSONReaderEvent::EndArray | StreamableJSONReaderEvent::EndType => {
                        let obj = self.stack.pop().unwrap();
                        self.add_to_last_stack_element(obj);
                    }
            StreamableJSONReaderEvent::StartArray => {
                        self.stack.push(StreamableJSONEntry::Array(Vec::new()));
                    }
            StreamableJSONReaderEvent::String(s) => self.add_to_last_stack_element(StreamableJSONEntry::String(s)),
            StreamableJSONReaderEvent::Constant(s) => self.add_to_last_stack_element(StreamableJSONEntry::Constant(s)),
            StreamableJSONReaderEvent::StartType(s) => self.stack.push(StreamableJSONEntry::Type(s, Vec::new())),
            StreamableJSONReaderEvent::Finished => {
                assert!(self.stack.len() == 0);
            },
            StreamableJSONReaderEvent::Initialized => {
                assert!(self.stack.is_empty());
            },
        }
        return StreamableJSONReaderCallbackReturn::Continue;
    }
}

pub fn deserialize_orderedbag_from_string(text: String) -> Result<StreamableJSONEntry, StreamableJSONReaderError> {
    let mut deserializer = OrderedBagDeserializer::new();
    let mut reader = StreamableJSONReader::new(&mut deserializer);
    let mut it = IteratorReadable::new(Box::new(StringCharIterator::new(text)));
    reader.pushdata(&mut it).unwrap();
    reader.finish().unwrap();

    return Ok(deserializer.result().unwrap());
}

pub struct StringCharIterator {
    text: String,
}

impl Iterator for StringCharIterator {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.text.is_empty() {
            return None;
        }
        let c = self.text.remove(0);
        return Some(c);
    }
}

impl StringCharIterator {
    pub fn new(text: String) -> Self {
        StringCharIterator {
            text,
        }
    }
    pub fn chars(&self) -> Chars {
        return self.text.chars()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orderedbag_deserializer() {
        let mut deserializer = OrderedBagDeserializer::new();
        let mut reader = StreamableJSONReader::new(&mut deserializer);
        reader.pushdata(&mut IteratorReadable::new(Box::new("{}".chars()))).unwrap();
        reader.finish().unwrap();
        assert_eq!(deserializer.result().unwrap(), StreamableJSONEntry::Object(OrderedBag::new()));
    }

    #[test]
    fn test_string() {
        let mut deserializer = OrderedBagDeserializer::new();
        let mut reader = StreamableJSONReader::new(&mut deserializer);
        reader.pushdata(&mut IteratorReadable::new(Box::new("\"abc\"".chars()))).unwrap();
        reader.finish().unwrap();
        assert_eq!(deserializer.result().unwrap(), StreamableJSONEntry::String(String::from("abc")));
    }

    #[test]
    fn test_array() {
        let mut deserializer = OrderedBagDeserializer::new();
        let mut reader = StreamableJSONReader::new(&mut deserializer);
        reader.pushdata(&mut IteratorReadable::new(Box::new("[1,2]".chars()))).unwrap();
        reader.finish().unwrap();
        assert_eq!(deserializer.result().unwrap(), 
            StreamableJSONEntry::Array(
                vec![StreamableJSONEntry::Constant("1".into()), 
                StreamableJSONEntry::Constant("2".into())]));
    }

    #[test]
    fn test_nestedarray() {
        let mut deserializer = OrderedBagDeserializer::new();
        let mut reader = StreamableJSONReader::new(&mut deserializer);
        reader.pushdata(&mut IteratorReadable::new(Box::new("[1,[2,[\"3\"]]]".chars()))).unwrap();
        reader.finish().unwrap();
        assert_eq!(deserializer.result().unwrap(), 
            StreamableJSONEntry::Array(
                vec![StreamableJSONEntry::Constant("1".into()),
                     StreamableJSONEntry::Array(
                        vec![StreamableJSONEntry::Constant("2".into()),
                             StreamableJSONEntry::Array(
                                vec![StreamableJSONEntry::String("3".into())])])])); 
    }

    #[test]
    fn test_type() {
        let mut deserializer = OrderedBagDeserializer::new();
        let mut reader = StreamableJSONReader::new(&mut deserializer);
        reader.pushdata(&mut IteratorReadable::new(Box::new("1(2)".chars()))).unwrap();
        reader.finish().unwrap();
        assert_eq!(deserializer.result().unwrap(), 
            StreamableJSONEntry::Type(
                "1".into(),
                vec![StreamableJSONEntry::Constant("2".into())]
            ));
    }

}