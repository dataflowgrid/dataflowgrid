/* This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich */
#![allow(dead_code)]
use std::str::Chars;

use dataflowgrid_commons::ordered_multi_dict::OrderedMultiDict;
use dataflowgrid_commons::readers::reader::IteratorReadable;
use crate::parser::StreamableJSONReader;

use crate::parser::{StreamableJSONReaderCallback, StreamableJSONReaderCallbackReturn, StreamableJSONReaderError, StreamableJSONReaderEvent} ;
use crate::StreamableJSONEntry;

pub struct OrderMultiDictDeserializer {
    stack: Vec<StreamableJSONEntry>,
}

impl OrderMultiDictDeserializer {
    pub fn new() -> Self {
        OrderMultiDictDeserializer {
            stack: Vec::new(),
        }
    }
    pub fn result(mut self) -> StreamableJSONEntry {
        let r =  self.stack.remove(0);
        if let StreamableJSONEntry::Id(mut opt) = r {
            let r = opt.take().unwrap();
            return *r
        } else {
            unreachable!()
        }
    }
}

impl StreamableJSONReaderCallback for OrderMultiDictDeserializer {
    fn on_streamablejson_event(&mut self, event: StreamableJSONReaderEvent) -> StreamableJSONReaderCallbackReturn {
        match event {
            StreamableJSONReaderEvent::StartObject => {
                        self.stack.push(StreamableJSONEntry::Object(OrderedMultiDict::new()));
                    }
            StreamableJSONReaderEvent::EndObject => {
                        let obj = self.stack.pop().unwrap();
                        let mut last = self.stack.last_mut().unwrap();
                        match &mut last {
                            StreamableJSONEntry::Object(last) => {
                                last.push(StreamableJSONEntry::String("1".into()), obj);
                            }
                            StreamableJSONEntry::Id(opt) => {
                                opt.replace(Box::new(obj));
                            }
                            _ => panic!("Expected object")
                        }
                    }
            StreamableJSONReaderEvent::StartArray => {
                        self.stack.push(StreamableJSONEntry::Array(Vec::new()));
                    }
            StreamableJSONReaderEvent::EndArray => {
                        let arr = self.stack.pop().unwrap();
                        let last = self.stack.last_mut().unwrap();
                        match last {
                            StreamableJSONEntry::Array(last) => {
                                last.push(arr);
                            }
                            StreamableJSONEntry::Id(opt) => {
                                opt.replace(Box::new(arr));
                            }
                                    _ => panic!("Expected object")
                        }
                    }
            StreamableJSONReaderEvent::String(s) => {
                let last = self.stack.last_mut().unwrap();
                match last {
                    StreamableJSONEntry::Array(last) => {
                        last.push(StreamableJSONEntry::String(s));
                    }
                    StreamableJSONEntry::Id(opt) => {
                        opt.replace(Box::new(StreamableJSONEntry::String(s)));
                    }
                    StreamableJSONEntry::Object(opt) => {
                        match opt.last_entry() {
                            None => {
                                //dict is empty, insert as new key
                                opt.push(StreamableJSONEntry::String(s), StreamableJSONEntry::Id(None));
                            }
                            Some(StreamableJSONEntry::Id(None)) => {
                                //this is the value
                                opt.replace_last_entry(StreamableJSONEntry::String(s));
                            }
                            Some(StreamableJSONEntry::String(_)) => {
                                //this is a new key
                                opt.push(StreamableJSONEntry::String(s), StreamableJSONEntry::Id(None));
                            }
                            _ => panic!("Unexpected object state")
                        }
                    }
                    _ => panic!("Expected array, object or id")
                }
            }
            StreamableJSONReaderEvent::Constant(s) => {
                let last = self.stack.last_mut().unwrap();
                match last {
                    StreamableJSONEntry::Array(last) => {
                        last.push(StreamableJSONEntry::Constant(s));
                    }
                    StreamableJSONEntry::Id(opt) => {
                        opt.replace(Box::new(StreamableJSONEntry::Constant(s)));
                    }
                    StreamableJSONEntry::Type(_, last) => {
                        last.push(StreamableJSONEntry::Constant(s));
                    }
                    StreamableJSONEntry::Object(opt) => {
                        match opt.last_entry() {
                            None => {
                                //dict is empty, insert as new key
                                opt.push(StreamableJSONEntry::String(s), StreamableJSONEntry::Id(None));
                            }
                            Some(StreamableJSONEntry::Id(None)) => {
                                //this is the value
                                opt.replace_last_entry(StreamableJSONEntry::String(s));
                            }
                            Some(StreamableJSONEntry::String(_)) => {
                                //this is a new key
                                opt.push(StreamableJSONEntry::String(s), StreamableJSONEntry::Id(None));
                            }
                            _ => panic!("Unexpected object state")
                        }
                    }
                    _ => panic!("Expected array, type, object or id")
                }
            }
            StreamableJSONReaderEvent::StartType(s) => {
                        self.stack.push(StreamableJSONEntry::Type(s, Vec::new()));
                    }
            StreamableJSONReaderEvent::EndType => {
                        let t = self.stack.pop().unwrap();
                        let last = self.stack.last_mut().unwrap();
                        match last {
                            StreamableJSONEntry::Array(last) => {
                                last.push(t);
                            }
                            StreamableJSONEntry::Id(opt) => {
                                opt.replace(Box::new(t));
                            }
                            _ => panic!("Expected type")
                        }
                    }
            StreamableJSONReaderEvent::Finished => {
                assert!(self.stack.len() == 1);
            },
            StreamableJSONReaderEvent::Initialized => {
                assert!(self.stack.is_empty());
                self.stack.push(StreamableJSONEntry::Id(None))
            },
        }
        return StreamableJSONReaderCallbackReturn::Continue;
    }
}

pub fn deserialize_ordermultidict_from_string(text: String) -> Result<StreamableJSONEntry, StreamableJSONReaderError> {
    let mut deserializer = OrderMultiDictDeserializer::new();
    let mut reader = StreamableJSONReader::new(&mut deserializer);
    let mut it = IteratorReadable::new(Box::new(StringCharIterator::new(text)));
    reader.pushdata(&mut it).unwrap();
    reader.finish().unwrap();

    return Ok(deserializer.result());
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
    fn test_ordermultidict_deserializer() {
        let mut deserializer = OrderMultiDictDeserializer::new();
        let mut reader = StreamableJSONReader::new(&mut deserializer);
        reader.pushdata(&mut IteratorReadable::new(Box::new("{}".chars()))).unwrap();
        reader.finish().unwrap();
        assert_eq!(deserializer.result(), StreamableJSONEntry::Object(OrderedMultiDict::new()));
    }

    #[test]
    fn test_string() {
        let mut deserializer = OrderMultiDictDeserializer::new();
        let mut reader = StreamableJSONReader::new(&mut deserializer);
        reader.pushdata(&mut IteratorReadable::new(Box::new("\"abc\"".chars()))).unwrap();
        reader.finish().unwrap();
        assert_eq!(deserializer.result(), StreamableJSONEntry::String(String::from("abc")));
    }

    #[test]
    fn test_array() {
        let mut deserializer = OrderMultiDictDeserializer::new();
        let mut reader = StreamableJSONReader::new(&mut deserializer);
        reader.pushdata(&mut IteratorReadable::new(Box::new("[1,2]".chars()))).unwrap();
        reader.finish().unwrap();
        assert_eq!(deserializer.result(), 
            StreamableJSONEntry::Array(
                vec![StreamableJSONEntry::Constant("1".into()), 
                StreamableJSONEntry::Constant("2".into())]));
    }

    #[test]
    fn test_nestedarray() {
        let mut deserializer = OrderMultiDictDeserializer::new();
        let mut reader = StreamableJSONReader::new(&mut deserializer);
        reader.pushdata(&mut IteratorReadable::new(Box::new("[1,[2,[\"3\"]]]".chars()))).unwrap();
        reader.finish().unwrap();
        assert_eq!(deserializer.result(), 
            StreamableJSONEntry::Array(
                vec![StreamableJSONEntry::Constant("1".into()),
                     StreamableJSONEntry::Array(
                        vec![StreamableJSONEntry::Constant("2".into()),
                             StreamableJSONEntry::Array(
                                vec![StreamableJSONEntry::String("3".into())])])])); 
    }

    #[test]
    fn test_type() {
        let mut deserializer = OrderMultiDictDeserializer::new();
        let mut reader = StreamableJSONReader::new(&mut deserializer);
        reader.pushdata(&mut IteratorReadable::new(Box::new("1(2)".chars()))).unwrap();
        reader.finish().unwrap();
        assert_eq!(deserializer.result(), 
            StreamableJSONEntry::Type(
                "1".into(),
                vec![StreamableJSONEntry::Constant("2".into())]
            ));
    }

}