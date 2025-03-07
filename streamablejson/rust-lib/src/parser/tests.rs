use std::cell::RefCell;

use super::*;

struct TestCallback {
    events: RefCell<Vec<JSONSReaderEvent>>,
}

impl JSONSReaderCallback for TestCallback {
    fn on_jsons_event(&self, event: JSONSReaderEvent) -> JSONSReaderCallbackReturn {
        println!("{:?}", event);
        self.events.borrow_mut().push(event);
        JSONSReaderCallbackReturn::Continue
    }
}

#[test]
fn test_entities() {
    let b = "true ";
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });

    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata(b.chars()).unwrap();
    let mut events = c.events.borrow_mut();

    //as no finish was called, wo don't get the finished event
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("true")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}


#[test]
fn test_string1() {
    let b = " \"test\" ";
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });

    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata(b.chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();

    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::String(String::from("test")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_unicodeescape() {
    let b = " \"test\\\"\\u0008\\u000c\\n\\r\\t\\\\\"";
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });

    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata(b.chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();

    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::String(String::from("test\"\u{0008}\u{000c}\n\r\t\\")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_constant() {
    let b = " test";
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });

    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata(b.chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();

    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("test")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_invalid_constant() {
    let b = " test";
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });

    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata(b.chars()).unwrap();
    reader.pushdata(":2".chars()).expect_err("this should fail because not inside of object");
    reader.finish().expect_err("finish is also in invalid state");

    let mut events = c.events.borrow_mut();

    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("test")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}


#[test]
fn test_type() {
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata("test()".chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();

    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndType);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartType(String::from("test")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}


#[test]
fn test_type_with_string() {
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata("test(\"string\")".chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();

    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndType);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::String(String::from("string")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartType(String::from("test")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_empty_array() {
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata("[]".chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();

    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndArray);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartArray);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_array_with_numbers() {
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata("[1,2]".chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndArray);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("2")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("1")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartArray);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_array_with_strings() {
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata("[\"abc\",\"def\"]".chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndArray);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::String(String::from("def")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::String(String::from("abc")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartArray);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_array_with_numbers_and_strings() {
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata("[\"abc\", 1,\"def\" ,2]".chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndArray);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("2")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::String(String::from("def")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("1")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::String(String::from("abc")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartArray);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_empty_object() {
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata("{}".chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_object() {
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata("{\"abc\":\"def\"}".chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::String(String::from("def")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::String(String::from("abc")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_object2() {
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata("{\"abc\":\"def\", 1: 2}".chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("2")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("1")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::String(String::from("def")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::String(String::from("abc")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_nestedobject1() {
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata("{1 : { 2:3}}".chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("3")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("2")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("1")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}


#[test]
fn test_nestedobject1_with_array() {
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata("{1 : [{ 2:3},{4:5}]}".chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndArray);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("5")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("4")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("3")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("2")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartArray);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("1")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

#[test]
fn test_type_mapping() {
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata("{a(b):c(d)}".chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndType);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("d")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartType(String::from("c")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndType);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("b")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartType(String::from("a")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}


#[test]
fn test_nested_type() {
    let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
    let mut reader = JSONSReader::new(c.as_ref());

    reader.pushdata("{a(b(c)):d(e)}".chars()).unwrap();
    reader.finish().unwrap();

    let mut events = c.events.borrow_mut();
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndType);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("e")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartType(String::from("d")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndType);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::EndType);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Constant(String::from("c")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartType(String::from("b")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartType(String::from("a")));
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StartObject);
    assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
    assert!(events.pop().is_none());
}

