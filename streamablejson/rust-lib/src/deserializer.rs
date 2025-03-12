/* This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich */

use dataflowgrid_commons::ordered_multi_dict::OrderedMultiDict;

use crate::parser::{StreamableJSONReaderCallback, StreamableJSONReaderEvent,StreamableJSONReaderCallbackReturn} ;
use crate::StreamableJSONEntry;

struct OrderMultiDictDeserializer {
    stack: Vec<StreamableJSONEntry>,
}

impl OrderMultiDictDeserializer {
    pub fn new() -> Self {
        OrderMultiDictDeserializer {
            stack: Vec::new(),
        }
    }
    pub fn result(&self) -> &Option<Box<StreamableJSONEntry>> {
        match self.stack.first() {
            Some(StreamableJSONEntry::Id(opt)) => 
                opt,
            _ => &None
        }
    }
}

impl StreamableJSONReaderCallback for OrderMultiDictDeserializer {
    fn on_streamablejson_event(&mut self, event: StreamableJSONReaderEvent) -> StreamableJSONReaderCallbackReturn {
        println!("{:?}", event);
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
                    _ => panic!("Expected array or id")
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
                    _ => panic!("Expected array or id")
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
            StreamableJSONReaderEvent::MarkerKey => todo!(),
            StreamableJSONReaderEvent::MarkerValue => todo!(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::StreamableJSONReader;
    use dataflowgrid_commons::readers::reader::IteratorReadable;

    #[test]
    fn test_ordermultidict_deserializer() {
        let mut deserializer = OrderMultiDictDeserializer::new();
        let mut reader = StreamableJSONReader::new(&mut deserializer);
        reader.pushdata(&mut IteratorReadable::new(Box::new("{}".chars()))).unwrap();
        reader.finish().unwrap();
        println!("{:?}", deserializer.result());
        assert_eq!(*deserializer.result().as_ref().unwrap().as_ref(), StreamableJSONEntry::Object(OrderedMultiDict::new()));
    }

    #[test]
    fn test_string() {
        let mut deserializer = OrderMultiDictDeserializer::new();
        let mut reader = StreamableJSONReader::new(&mut deserializer);
        reader.pushdata(&mut IteratorReadable::new(Box::new("\"abc\"".chars()))).unwrap();
        reader.finish().unwrap();
        println!("{:?}", deserializer.result());
        assert_eq!(*deserializer.result().as_ref().unwrap().as_ref(), StreamableJSONEntry::String(String::from("abc")));
    }

    #[test]
    fn test_array() {
        let mut deserializer = OrderMultiDictDeserializer::new();
        let mut reader = StreamableJSONReader::new(&mut deserializer);
        reader.pushdata(&mut IteratorReadable::new(Box::new("[1,2]".chars()))).unwrap();
        reader.finish().unwrap();
        println!("{:?}", deserializer.result());
        assert_eq!(*deserializer.result().as_ref().unwrap().as_ref(), 
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
        println!("{:?}", deserializer.result());
        assert_eq!(*deserializer.result().as_ref().unwrap().as_ref(), 
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
        println!("{:?}", deserializer.result());
        assert_eq!(*deserializer.result().as_ref().unwrap().as_ref(), 
            StreamableJSONEntry::Type(
                "1".into(),
                vec![StreamableJSONEntry::Constant("2".into())]
            ));
    }

}