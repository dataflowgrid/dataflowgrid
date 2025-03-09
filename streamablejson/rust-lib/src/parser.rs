// This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich

#![allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum StreamableJSONReaderStateEnum {
    INIT{is_key: bool},
    OBJECT {is_key: bool},
    ARRAY,
    STRING{is_key: bool},
    CONSTANT{is_key: bool}, //true, false, number, null, object
    TYPE,
}

#[derive(Debug, PartialEq)]
pub enum StreamableJSONReaderEvent {
    StartObject,
    EndObject,
    StartArray,
    EndArray,
    String(String),
    Constant(String),
    MarkerKey,
    MarkerValue,
    StartType(String),
    EndType,
    Finished,
    Initialized
}

#[derive(Debug)]
pub enum StreamableJSONReaderError {
    InvalidJSON,
    InvalidState,
    CallbackError(Box<dyn Error>),
}

#[derive(Debug)]
pub enum StreamableJSONReaderCallbackReturn {
    Continue,
    SkipValue, //skip the value for this key
    Skip, //skip the rest of this structure, whether it is an object or an array or type
    StopOk, //stop parsing and return OK
    StopErr(Box<dyn Error>), //stop parsing and return error
}

enum Callback<'a> {
    None,
    Function(String, &'a dyn StreamableJSONReaderCallback)
}
use std::error::Error;
use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::str::Chars;

impl<'a> Debug for Callback<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Callback::None => write!(f, "None"),
            Callback::Function(name, _) => {
                let typename = std::any::type_name::<&'a dyn StreamableJSONReaderCallback>();
                write!(f, "Function {name}({typename})")
            }
        }
    }
}



pub trait StreamableJSONReaderCallback {
    fn on_streamablejson_event(&self, event: StreamableJSONReaderEvent) -> StreamableJSONReaderCallbackReturn;
}

#[derive(Debug)]
pub struct StreambleJSONReader<'a> {
    callback: Callback<'a>,
    chars: Vec<char>,
    stack: Vec<StreamableJSONReaderStateEnum>,

    stringescape: bool,
    stringescapeunicode: bool,
    stringescapeunicodecount: u8,
    stringescapeunicodevalue: u32,

    last_callback_return: StreamableJSONReaderCallbackReturn,
}

impl<'a> StreambleJSONReader<'a> {
    pub fn new(callback: &'a dyn StreamableJSONReaderCallback) -> StreambleJSONReader<'a> {   
        let mut stack = Vec::new();
        stack.push(StreamableJSONReaderStateEnum::INIT{is_key: false});

        let r = callback.on_streamablejson_event(StreamableJSONReaderEvent::Initialized);
        StreambleJSONReader {
            callback: Callback::Function(String::from("callback"), callback),
            chars: Vec::new(),
            stack,
            stringescape: false,
            stringescapeunicode: false,
            stringescapeunicodecount: 0,
            stringescapeunicodevalue: 0,
            last_callback_return: r,
        }
    }

    fn callback(&mut self, event: StreamableJSONReaderEvent) {
        self.last_callback_return = match &self.callback {
            Callback::Function(_, f) => f.on_streamablejson_event(event),
            Callback::None => StreamableJSONReaderCallbackReturn::Continue,
        }
    }

    pub fn pushdata(&mut self, data: Chars) -> Result<(), StreamableJSONReaderError> {
        let mut iter = data.into_iter();
        let mut reprocess_char = false;
        let mut c = ' ';
        loop {
            let e = std::mem::replace(&mut self.last_callback_return, StreamableJSONReaderCallbackReturn::Continue);
            if let StreamableJSONReaderCallbackReturn::StopErr(e) = e {
                return Err(StreamableJSONReaderError::CallbackError(e));
            }
            if let StreamableJSONReaderCallbackReturn::StopOk = e {
                return Ok(());
            }
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
            if self.stack.is_empty() {
                return Err(StreamableJSONReaderError::InvalidState);
            }
            match self.stack.last().unwrap().clone() {
                StreamableJSONReaderStateEnum::INIT{is_key} => {
                    if c.is_whitespace() {
                        continue;
                    }
                    match c {
                        '{' => {
                            self.stack.push(StreamableJSONReaderStateEnum::OBJECT {is_key: true});
                            self.callback(StreamableJSONReaderEvent::StartObject);
                        }
                        '[' => {
                            self.stack.push(StreamableJSONReaderStateEnum::ARRAY);
                            self.stack.push(StreamableJSONReaderStateEnum::INIT{is_key});
                            self.callback(StreamableJSONReaderEvent::StartArray);
                        }
                        '"' => {
                            self.stack.push(StreamableJSONReaderStateEnum::STRING{is_key});
                            self.chars.clear();
                        }
                        ')'|']'|'}'|','|':' => {
                            //loks like the end of a type
                            reprocess_char = true;
                            self.stack.pop().unwrap();
                            self.chars.clear();
                        }
                        _ => {
                            if c.is_alphanumeric() {
                                self.stack.push(StreamableJSONReaderStateEnum::CONSTANT{is_key});
                                self.chars.push(c);
                            } else {
                                return Err(StreamableJSONReaderError::InvalidJSON);
                            }
                        }
                    }
                }
                StreamableJSONReaderStateEnum::OBJECT{is_key} => {
                    match c {
                        '}' => {
                            self.stack.pop().unwrap();
                            self.callback(StreamableJSONReaderEvent::EndObject);
                        }
                        ':'  => {
                            if is_key {
                                self.stack.pop().unwrap();
                                self.stack.push(StreamableJSONReaderStateEnum::OBJECT{is_key: false});
                                self.stack.push(StreamableJSONReaderStateEnum::INIT{is_key: false});
                            } else {
                                self.stack.push(StreamableJSONReaderStateEnum::INIT{is_key: false});
                            }
                            self.chars.clear();
                        }
                        ',' => {
                            self.stack.push(StreamableJSONReaderStateEnum::INIT{is_key:true});
                        }
                        _ => {
                            if c.is_whitespace() {
                                continue;
                            }
                            if c == '"' {
                                self.stack.push(StreamableJSONReaderStateEnum::STRING{is_key});
                                self.chars.clear();
                            } else {
                                self.chars.push(c);
                                self.stack.push(StreamableJSONReaderStateEnum::CONSTANT {is_key});
                            }
                        }
                    }
                }
                StreamableJSONReaderStateEnum::ARRAY => {
                    match c {
                        ']' => {
                            self.stack.pop().unwrap();
                            self.callback(StreamableJSONReaderEvent::EndArray);
                        }
                        ',' => {
                            self.stack.push(StreamableJSONReaderStateEnum::INIT{is_key: false});
                        }
                        _ => {
                            if c.is_whitespace() {
                                continue;
                            }
                            if c == '"' {
                                self.stack.pop().unwrap();
                                self.callback(StreamableJSONReaderEvent::String(String::from_iter(&self.chars)));
                            } else {
                                return Err(StreamableJSONReaderError::InvalidJSON);
                            }
                        }
                    }
                }
                StreamableJSONReaderStateEnum::STRING{..} => {
                    if self.stringescape {
                        if self.stringescapeunicode {
                            if self.stringescapeunicodecount < 4 {
                                if c.is_digit(16) {
                                    self.stringescapeunicodevalue = self.stringescapeunicodevalue << 4 | c.to_digit(16).unwrap();
                                    self.stringescapeunicodecount += 1;
                                    continue;
                                } else {
                                    return Err(StreamableJSONReaderError::InvalidJSON);
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
                                    return Err(StreamableJSONReaderError::InvalidJSON);
                                }
                            }
                        }
                    } else {
                        if c == '\\' {
                            self.stringescape = true;
                        } else {
                            if c == '"' {
                                self.callback(StreamableJSONReaderEvent::String(String::from_iter(&self.chars)));
                                self.stack.pop().unwrap();
                            } else {
                                self.chars.push(c);
                            }
                        }
                    }
                }
                StreamableJSONReaderStateEnum::CONSTANT{is_key} => {
                    if c.is_alphanumeric() {
                        self.chars.push(c);
                    } else if c == '(' {
                        self.stack.pop().unwrap();
                        self.stack.push(StreamableJSONReaderStateEnum::TYPE);
                        self.stack.push(StreamableJSONReaderStateEnum::INIT{is_key});
                        self.callback(StreamableJSONReaderEvent::StartType(String::from_iter(&self.chars)));
                        self.chars.clear();
                    } else {
                        self.callback(StreamableJSONReaderEvent::Constant(String::from_iter(&self.chars)));
                        self.stack.pop().unwrap();
                        reprocess_char = true;
                        self.chars.clear();
                    }
                }
                StreamableJSONReaderStateEnum::TYPE => {
                    if c == ')' {
                        self.stack.pop().unwrap();
                        self.callback(StreamableJSONReaderEvent::EndType);
                    } else {
                        self.callback(StreamableJSONReaderEvent::Constant(String::from_iter(&self.chars)));
                        self.chars.clear();
                        self.stack.pop().unwrap();
                        reprocess_char = true;
                    }
                }
            }
        }
    
    }

    pub fn finish(&mut self) -> Result<(), StreamableJSONReaderError> {
        if self.stack.is_empty() {
            return Err(StreamableJSONReaderError::InvalidState);
        }
        match self.stack.pop().unwrap() {
            StreamableJSONReaderStateEnum::INIT{..} => {
            }
            StreamableJSONReaderStateEnum::OBJECT { .. } => {
                return Err(StreamableJSONReaderError::InvalidJSON);
            }
            StreamableJSONReaderStateEnum::ARRAY => {
                return Err(StreamableJSONReaderError::InvalidJSON);
            }
            StreamableJSONReaderStateEnum::STRING{..} => {
                return Err(StreamableJSONReaderError::InvalidJSON);
            }
            StreamableJSONReaderStateEnum::CONSTANT{is_key} => {
                if !is_key {
                    self.callback(StreamableJSONReaderEvent::Constant(String::from_iter(&self.chars)));
                } else {
                    return Err(StreamableJSONReaderError::InvalidJSON);
                }
            }
            StreamableJSONReaderStateEnum::TYPE => {
                return Err(StreamableJSONReaderError::InvalidJSON);
            }
        }
        if self.stack.len() >1 {
            return Err(StreamableJSONReaderError::InvalidState);
        }
        self.callback(StreamableJSONReaderEvent::Finished);
        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod tests2 {
    use super::*;
    use std::cell::RefCell;

    struct TestCallback {
        result: RefCell<Vec<StreamableJSONReaderCallbackReturn>>,
        events: RefCell<Vec<StreamableJSONReaderEvent>>,
    }
    
    impl StreamableJSONReaderCallback for TestCallback {
        fn on_streamablejson_event(&self, event: StreamableJSONReaderEvent) -> StreamableJSONReaderCallbackReturn {
            println!("{:?}", event);
            self.events.borrow_mut().push(event);
            self.result.borrow_mut().pop().unwrap()
        }
    }
    
    #[test]
    fn test_continue() {
        let b = "{}";
        let c = Box::new(TestCallback { 
            result: RefCell::new(
                vec![
                    StreamableJSONReaderCallbackReturn::Continue,
                    StreamableJSONReaderCallbackReturn::Continue,
                    StreamableJSONReaderCallbackReturn::Continue,
                    StreamableJSONReaderCallbackReturn::Continue,
                ]
            ),
            events: RefCell::new(Vec::new()) 
        });
    
        let mut reader = StreambleJSONReader::new(c.as_ref());
    
        reader.pushdata(b.chars()).unwrap();
        reader.finish().unwrap();
        let mut events = c.events.borrow_mut();
    
        //as no finish was called, wo don't get the finished event
        assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
        assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::EndObject);
        assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartObject);
        assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
        assert!(events.pop().is_none());
    }

    #[test]
    fn test_stopok() {
        let b = "{}";
        let c = Box::new(TestCallback { 
            result: RefCell::new(
                vec![
                    StreamableJSONReaderCallbackReturn::Continue,
                    StreamableJSONReaderCallbackReturn::StopOk,
                ]
            ),
            events: RefCell::new(Vec::new()) 
        });
    
        let mut reader = StreambleJSONReader::new(c.as_ref());
    
        reader.pushdata(b.chars()).unwrap();
        reader.finish().unwrap(); //finish works because we never nested into any object
        let mut events = c.events.borrow_mut();
    
        //as no finish was called, wo don't get the finished event
        assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Finished);
        assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
        assert!(events.pop().is_none());
    }

    #[test]
    fn test_stopok2() {
        let b = "{}";
        let c = Box::new(TestCallback { 
            result: RefCell::new(
                vec![
                    StreamableJSONReaderCallbackReturn::Continue,
                    StreamableJSONReaderCallbackReturn::StopOk,
                    StreamableJSONReaderCallbackReturn::Continue,
                ]
            ),
            events: RefCell::new(Vec::new()) 
        });
    
        let mut reader = StreambleJSONReader::new(c.as_ref());
    
        reader.pushdata(b.chars()).unwrap();
        reader.finish().expect_err("we were in the middle of an object");
        let mut events = c.events.borrow_mut();
    
        //as no finish was called, wo don't get the finished event
        assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartObject);
        assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
        assert!(events.pop().is_none());
    }

    #[test]
    fn test_stoperr() {
        let b = "{}";
        let c = Box::new(TestCallback { 
            result: RefCell::new(
                vec![
                    StreamableJSONReaderCallbackReturn::Continue,
                    StreamableJSONReaderCallbackReturn::StopErr(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test"))),
                    StreamableJSONReaderCallbackReturn::Continue,
                ]
            ),
            events: RefCell::new(Vec::new()) 
        });
    
        let mut reader = StreambleJSONReader::new(c.as_ref());
    
        let err = reader.pushdata(b.chars()).expect_err("we generated this error");
        if let StreamableJSONReaderError::CallbackError(err) = err {
            let err = err.as_ref();
            assert!(err.is::<std::io::Error>());
        } else {
            panic!("wrong error type");
        }
        reader.finish().expect_err("we were in the middle of an object");
        let mut events = c.events.borrow_mut();
    
        //as no finish was called, wo don't get the finished event
        assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::StartObject);
        assert_eq!(events.pop().unwrap(), StreamableJSONReaderEvent::Initialized);
        assert!(events.pop().is_none());
    }


}
