#![allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum JSONSReaderStateEnum {
    INIT,
    OBJECT {is_key: bool},
    ARRAY,
    STRING(bool),
    CONSTANT(bool), //true, false, number, null, object
    TYPE,
}

#[derive(Debug, PartialEq)]
pub enum JSONSReaderEvent {
    ValueStartObject,
    ValueEndObject,
    ValueStartArray,
    ValueEndArray,
    StringKey(String),
    ConstantKey(String),
    StringValue(String),
    ConstantValue(String),
    ValueStartType(String),
    ValueEndType,
    Finished,
    Initialized
}

#[derive(Debug)]
pub enum JSONSReaderError {
    InvalidJSON,
    InvalidState,
}

pub enum JSONSReaderCallbackReturn {
    Continue,
    Stop,
}

enum Callback<'a> {
    None,
    Function(&'a dyn JSONSReaderCallback)
}
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::str::Chars;

impl<'a> Debug for Callback<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Callback::None => write!(f, "None"),
            Callback::Function(_) => write!(f, "Function")
        }
    }
}



pub trait JSONSReaderCallback {
    fn on_jsons_event(&self, event: JSONSReaderEvent) -> JSONSReaderCallbackReturn;
}

#[derive(Debug)]
pub struct JSONSReader<'a> {
    callback: Callback<'a>,
    chars: Vec<char>,
    stack: Vec<JSONSReaderStateEnum>,

    stringescape: bool,
    stringescapeunicode: bool,
    stringescapeunicodecount: u8,
    stringescapeunicodevalue: u32,
}

impl<'a> JSONSReader<'a> {
    pub fn new(callback: &'a dyn JSONSReaderCallback) -> JSONSReader<'a> {   
        let mut stack = Vec::new();
        stack.push(JSONSReaderStateEnum::INIT);

        callback.on_jsons_event(JSONSReaderEvent::Initialized);
        JSONSReader {
            callback: Callback::Function(callback),
            chars: Vec::new(),
            stack,
            stringescape: false,
            stringescapeunicode: false,
            stringescapeunicodecount: 0,
            stringescapeunicodevalue: 0,
        }
    }

    fn callback(&mut self, event: JSONSReaderEvent) -> JSONSReaderCallbackReturn {
        match &self.callback {
            Callback::Function(f) => f.on_jsons_event(event),
            Callback::None => JSONSReaderCallbackReturn::Continue,
        }
    }

    pub fn pushdata(&mut self, data: Chars) -> Result<(), JSONSReaderError> {
        let mut iter = data.into_iter();
        let mut reprocess_char = false;
        let mut c = ' ';
        loop {
            if !reprocess_char {
                match iter.next() {
                    Some(ch) => {
                        c = ch;
                    }
                    None => {
                        return Ok(());
                    }
                }
            }
            reprocess_char = false;
            println!("{:?}",c);
            match self.stack.last().unwrap() {
                JSONSReaderStateEnum::INIT => {
                    if c.is_whitespace() {
                        continue;
                    }
                    match c {
                        '{' => {
                            self.stack.push(JSONSReaderStateEnum::OBJECT {is_key: true});
                            self.callback(JSONSReaderEvent::ValueStartObject);
                        }
                        '[' => {
                            self.stack.push(JSONSReaderStateEnum::ARRAY);
                            self.stack.push(JSONSReaderStateEnum::INIT);
                            self.callback(JSONSReaderEvent::ValueStartArray);
                        }
                        '"' => {
                            self.stack.push(JSONSReaderStateEnum::STRING(false));
                            self.chars.clear();
                        }
                        ')'|']'|'}'|',' => {
                            //loks like the end of a type
                            reprocess_char = true;
                            self.stack.pop().unwrap();
                        }
                        _ => {
                            if c.is_alphanumeric() {
                                self.stack.push(JSONSReaderStateEnum::CONSTANT(false));
                                self.chars.push(c);
                            } else {
                                return Err(JSONSReaderError::InvalidJSON);
                            }
                        }
                    }
                }
                JSONSReaderStateEnum::OBJECT{is_key} => {
                    match c {
                        '}' => {
                            self.stack.pop().unwrap();
                            self.callback(JSONSReaderEvent::ValueEndObject);
                        }
                        ':'  => {
                            if *is_key {
                                self.stack.push(JSONSReaderStateEnum::INIT);
                            } else {
                                return Err(JSONSReaderError::InvalidJSON);
                            }
                        }
                        ',' => {
                            self.stack.push(JSONSReaderStateEnum::INIT);
                        }
                        _ => {
                            if c.is_whitespace() {
                                continue;
                            }
                            if c == '"' {
                                self.stack.push(JSONSReaderStateEnum::STRING(true));
                                self.chars.clear();
                            } else {
                                return Err(JSONSReaderError::InvalidJSON);
                            }
                        }
                    }
                }
                JSONSReaderStateEnum::ARRAY => {
                    match c {
                        ']' => {
                            self.stack.pop().unwrap();
                            self.callback(JSONSReaderEvent::ValueEndArray);
                        }
                        ',' => {
                            self.stack.push(JSONSReaderStateEnum::INIT);
                        }
                        _ => {
                            if c.is_whitespace() {
                                continue;
                            }
                            if c == '"' {
                                self.stack.pop().unwrap();
                                self.callback(JSONSReaderEvent::StringValue(String::from_iter(&self.chars)));
                            } else {
                                return Err(JSONSReaderError::InvalidJSON);
                            }
                        }
                    }
                }
                JSONSReaderStateEnum::STRING(is_key) => {
                    if self.stringescape {
                        if self.stringescapeunicode {
                            if self.stringescapeunicodecount < 4 {
                                if c.is_digit(16) {
                                    self.stringescapeunicodevalue = self.stringescapeunicodevalue << 4 | c.to_digit(16).unwrap();
                                    self.stringescapeunicodecount += 1;
                                    continue;
                                } else {
                                    return Err(JSONSReaderError::InvalidJSON);
                                }
                            } else {
                                self.chars.push(std::char::from_u32(self.stringescapeunicodevalue).unwrap());
                                self.stringescapeunicode = false;
                                self.stringescapeunicodecount = 0;
                                self.stringescapeunicodevalue = 0;
                            }
                        } else {
                            match c {
                                '"' => {
                                    self.chars.push('"');
                                    self.stringescape = false;
                                }
                                '\\' => {
                                    self.chars.push('\\');
                                    self.stringescape = false;
                                }
                                '/' => {
                                    self.chars.push('/');
                                    self.stringescape = false;
                                }
                                'b' => {
                                    self.chars.push('\u{0008}');
                                    self.stringescape = false;
                                }
                                'f' => {
                                    self.chars.push('\u{000c}');
                                    self.stringescape = false;
                                }
                                'n' => {
                                    self.chars.push('\n');
                                    self.stringescape = false;
                                }
                                'r' => {
                                    self.chars.push('\r');
                                    self.stringescape = false;
                                }
                                't' => {
                                    self.chars.push('\t');
                                    self.stringescape = false;
                                }
                                'u' => {
                                    self.stringescapeunicode = true;
                                }
                                _ => {
                                    return Err(JSONSReaderError::InvalidJSON);
                                }
                            }
                        }
                    } else {
                        if c == '\\' {
                            self.stringescape = true;
                        } else {
                            if c == '"' {
                                if *is_key {
                                       self.callback(JSONSReaderEvent::StringKey(String::from_iter(&self.chars)));
                                } else {
                                    self.callback(JSONSReaderEvent::StringValue(String::from_iter(&self.chars)));
                                }                                    
                                self.stack.pop().unwrap();
                            } else {
                                self.chars.push(c);
                            }
                        }
                    }
                }
                JSONSReaderStateEnum::CONSTANT(is_key) => {
                    if c.is_alphanumeric() {
                        self.chars.push(c);
                    } else if c == '(' {
                        self.stack.pop().unwrap();
                        self.stack.push(JSONSReaderStateEnum::TYPE);
                        self.stack.push(JSONSReaderStateEnum::INIT);
                        self.callback(JSONSReaderEvent::ValueStartType(String::from_iter(&self.chars)));
                    } else {
                        if *is_key {
                            self.callback(JSONSReaderEvent::StringKey(String::from_iter(&self.chars)));
                        } else {
                            self.callback(JSONSReaderEvent::ConstantKey(String::from_iter(&self.chars)));
                        }
                        self.stack.pop().unwrap();
                        reprocess_char = true;
                    }
                }
                JSONSReaderStateEnum::TYPE => {
                    if c == ')' {
                        self.stack.pop().unwrap();
                        self.callback(JSONSReaderEvent::ValueEndType);
                    } else {
                        self.callback(JSONSReaderEvent::ConstantValue(String::from_iter(&self.chars)));
                        self.chars.clear();
                        self.stack.pop().unwrap();
                        reprocess_char = true;
                    }
                }
            }
        }
    
    }

    pub fn finish(&mut self) -> Result<(), JSONSReaderError> {
        match self.stack.last().unwrap() {
            JSONSReaderStateEnum::INIT => {
            }
            JSONSReaderStateEnum::OBJECT { .. } => {
                return Err(JSONSReaderError::InvalidJSON);
            }
            JSONSReaderStateEnum::ARRAY => {
                return Err(JSONSReaderError::InvalidJSON);
            }
            JSONSReaderStateEnum::STRING(_) => {
                return Err(JSONSReaderError::InvalidJSON);
            }
            JSONSReaderStateEnum::CONSTANT(_) => {
                self.callback(JSONSReaderEvent::ConstantValue(String::from_iter(&self.chars)));
            }
            JSONSReaderStateEnum::TYPE => {
                return Err(JSONSReaderError::InvalidJSON);
            }
        }
        self.callback(JSONSReaderEvent::Finished);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(c.events.borrow().len(), 1);
    }


    #[test]
    fn test_string1() {
        let b = " \"test\" ";
        let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });

        let mut reader = JSONSReader::new(c.as_ref());

        reader.pushdata(b.chars()).unwrap();
        reader.finish().unwrap();

        let events = c.events.borrow();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], JSONSReaderEvent::StringValue(String::from("test")));
        assert_eq!(events[1], JSONSReaderEvent::Finished);
    }

    #[test]
    fn test_unicodeescape() {
        let b = " \"test\\\"\\u0008\\u000c\\n\\r\\t\\\\\"";
        let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });

        let mut reader = JSONSReader::new(c.as_ref());

        reader.pushdata(b.chars()).unwrap();
        reader.finish().unwrap();

        let events = c.events.borrow();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], JSONSReaderEvent::StringValue(String::from("test\"\u{0008}\u{000c}\n\r\t\\")));
        assert_eq!(events[1], JSONSReaderEvent::Finished);
    }

    #[test]
    fn test_constant() {
        let b = " test";
        let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });

        let mut reader = JSONSReader::new(c.as_ref());

        reader.pushdata(b.chars()).unwrap();
        reader.finish().unwrap();

        let events = c.events.borrow();
        assert_eq!(events.len(), 2);                                                             
        assert_eq!(events[0], JSONSReaderEvent::ConstantValue(String::from("test")));
        assert_eq!(events[1], JSONSReaderEvent::Finished);
    }

    #[test]
    fn test_invalid_constant() {
        let b = " test";
        let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });

        let mut reader = JSONSReader::new(c.as_ref());

        reader.pushdata(b.chars()).unwrap();
        reader.pushdata(":2".chars()).expect_err("this should fail because not inside of object");
        reader.finish().unwrap();

        let events = c.events.borrow();
        assert_eq!(events.len(), 2);                                                             
        assert_eq!(events[0], JSONSReaderEvent::ConstantValue(String::from("test")));
        assert_eq!(events[1], JSONSReaderEvent::Finished);
    }


    #[test]
    fn test_type() {
        let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
        let mut reader = JSONSReader::new(c.as_ref());

        reader.pushdata("test()".chars()).unwrap();
        reader.finish().unwrap();

        let events = c.events.borrow();
        assert_eq!(events.len(), 3);                                                             
        assert_eq!(events[0], JSONSReaderEvent::ValueStartType(String::from("test")));
        assert_eq!(events[1], JSONSReaderEvent::ValueEndType);
        assert_eq!(events[2], JSONSReaderEvent::Finished);
    }


    #[test]
    fn test_type_with_string() {
        let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
        let mut reader = JSONSReader::new(c.as_ref());

        reader.pushdata("test(\"string\")".chars()).unwrap();
        reader.finish().unwrap();

        let events = c.events.borrow();
        assert_eq!(events.len(), 4);                                                             
        assert_eq!(events[0], JSONSReaderEvent::ValueStartType(String::from("test")));
        assert_eq!(events[1], JSONSReaderEvent::StringValue(String::from("string")));
        assert_eq!(events[2], JSONSReaderEvent::ValueEndType);
        assert_eq!(events[3], JSONSReaderEvent::Finished);
    }

    #[test]
    fn test_empty_array() {
        let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
        let mut reader = JSONSReader::new(c.as_ref());

        reader.pushdata("[]".chars()).unwrap();
        reader.finish().unwrap();

        let events = c.events.borrow();
        assert_eq!(events.len(), 3);                                                             
        assert_eq!(events[0], JSONSReaderEvent::ValueStartArray);
        assert_eq!(events[1], JSONSReaderEvent::ValueEndArray);
        assert_eq!(events[2], JSONSReaderEvent::Finished);
    }

    #[test]
    fn test_array_with_numbers() {
        let c = Box::new(TestCallback { events: RefCell::new(Vec::new()) });
        let mut reader = JSONSReader::new(c.as_ref());

        reader.pushdata("[1,2]".chars()).unwrap();
        reader.finish().unwrap();

        let mut events = c.events.borrow_mut();
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Finished);
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ValueEndArray);
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ConstantValue(String::from("2")));
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ConstantValue(String::from("1")));
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ValueStartArray);
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
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ValueEndArray);
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StringValue(String::from("def")));
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StringValue(String::from("abc")));
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ValueStartArray);
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
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ValueEndArray);
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ConstantValue(String::from("2")));
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StringValue(String::from("def")));
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ConstantValue(String::from("1")));
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StringValue(String::from("abc")));
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ValueStartArray);
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
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ValueEndObject);
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ValueStartObject);
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
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ValueEndObject);
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StringValue(String::from("def")));
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StringKey(String::from("abc")));
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ValueStartObject);
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
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ValueEndObject);
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ConstantValue(String::from("2")));
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ConstantKey(String::from("1")));
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StringValue(String::from("def")));
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::StringKey(String::from("abc")));
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::ValueStartObject);
        assert_eq!(events.pop().unwrap(), JSONSReaderEvent::Initialized);
        assert!(events.pop().is_none());
    }

}
