// This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich

use std::cell::RefCell;

use dataflowgrid_commons::readers::reader::IteratorReadable;

use super::*;

struct TestCallback {
    events: RefCell<Vec<StreamableJSONReaderEvent>>,
}

impl StreamableJSONReaderCallback for TestCallback {
    fn on_streamablejson_event(&mut self, event: StreamableJSONReaderEvent) -> StreamableJSONReaderCallbackReturn {
        println!("{:?}", event);
        self.events.borrow_mut().push(event);
        StreamableJSONReaderCallbackReturn::Continue
    }
}

#[test]
fn test_entities() {
    let b = "true ";
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };

    let mut reader = StreamableJSONReader::new(&mut c);

    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    let mut events = c.events.borrow_mut();

    //as no finish was called, wo don't get the finished event
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("true")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}


#[test]
fn test_string1() {
    let b = " \"test\" ";
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };

    let mut reader = StreamableJSONReader::new(&mut c);

    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();

    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::String(String::from("test")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_unicodeescape() {
    let b = " \"test\\\"\\u0008\\u000c\\n\\r\\t\\\\\"";
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };

    let mut reader = StreamableJSONReader::new(&mut c);

    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();

    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::String(String::from("test\"\u{0008}\u{000c}\n\r\t\\")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_constant() {
    let b = " test";
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };

    let mut reader = StreamableJSONReader::new(&mut c);

    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();

    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("test")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_invalid_constant() {
    let b = " test";
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };

    let mut reader = StreamableJSONReader::new(&mut c);

    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.pushdata(&mut IteratorReadable::new(Box::new(":2".chars()))).expect_err("this should fail because not inside of object");

    let mut events = c.events.borrow_mut();

    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("test")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}


#[test]
fn test_type() {
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };
    let mut reader = StreamableJSONReader::new(&mut c);
    let b = "test()";
    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();

    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndType);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartType(String::from("test")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}


#[test]
fn test_type_with_string() {
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };
    let mut reader = StreamableJSONReader::new(&mut c);
    let b = "test(\"string\")";
    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();

    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndType);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::String(String::from("string")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartType(String::from("test")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_empty_array() {
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };
    let mut reader = StreamableJSONReader::new(&mut c);
    let b = "[]";
    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();

    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndArray);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartArray);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_array_with_numbers() {
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };
    let mut reader = StreamableJSONReader::new(&mut c);
    let b = "[1,2]";
    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndArray);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("2")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("1")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartArray);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_array_with_strings() {
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };
    let mut reader = StreamableJSONReader::new(&mut c);
    let b = "[\"abc\",\"def\"]";
    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndArray);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::String(String::from("def")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::String(String::from("abc")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartArray);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_array_with_numbers_and_strings() {
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };
    let mut reader = StreamableJSONReader::new(&mut c);
    let b = "[\"abc\", 1,\"def\" ,2]";
    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndArray);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("2")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::String(String::from("def")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("1")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::String(String::from("abc")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartArray);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_empty_object() {
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };
    let mut reader = StreamableJSONReader::new(&mut c);
    let b = "{}";
    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_object() {
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };
    let mut reader = StreamableJSONReader::new(&mut c);
    let b = "{\"abc\":\"def\"}";
    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::String(String::from("def")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::String(String::from("abc")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_object2() {
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };
    let mut reader = StreamableJSONReader::new(&mut c);
    let b = "{\"abc\":\"def\", 1: 2}";
    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("2")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("1")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::String(String::from("def")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::String(String::from("abc")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_nestedobject1() {
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };
    let mut reader = StreamableJSONReader::new(&mut c);
    let b = "{1 : { 2:3}}";
    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("3")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("2")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("1")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}


#[test]
fn test_nestedobject1_with_array() {
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };
    let mut reader = StreamableJSONReader::new(&mut c);
    let b = "{1 : [{ 2:3},{4:5}]}";
    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndArray);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("5")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("4")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("3")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("2")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartArray);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("1")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_type_mapping() {
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };
    let mut reader = StreamableJSONReader::new(&mut c);
    let b = "{a(b):c(d)}";
    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndType);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("d")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartType(String::from("c")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndType);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("b")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartType(String::from("a")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}


#[test]
fn test_nested_type() {
    let mut c = TestCallback { events: RefCell::new(Vec::new()) };
    let mut reader = StreamableJSONReader::new(&mut c);
    let b = "{a(b(c)):d(e)}";
    reader.pushdata(&mut IteratorReadable::new(Box::new(b.chars()))).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndType);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("e")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartType(String::from("d")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndType);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndType);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Constant(String::from("c")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartType(String::from("b")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartType(String::from("a")));
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

