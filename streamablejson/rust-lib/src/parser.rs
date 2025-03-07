#![allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum JSONSReaderStateEnum {
    INIT{is_key: bool},
    OBJECT {is_key: bool},
    ARRAY,
    STRING{is_key: bool},
    CONSTANT{is_key: bool}, //true, false, number, null, object
    TYPE,
}

#[derive(Debug, PartialEq)]
pub enum JSONSReaderEvent {
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
        stack.push(JSONSReaderStateEnum::INIT{is_key: false});

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
            if self.stack.is_empty() {
                return Err(JSONSReaderError::InvalidState);
            }
            match self.stack.last().unwrap().clone() {
                JSONSReaderStateEnum::INIT{is_key} => {
                    if c.is_whitespace() {
                        continue;
                    }
                    match c {
                        '{' => {
                            self.stack.push(JSONSReaderStateEnum::OBJECT {is_key: true});
                            self.callback(JSONSReaderEvent::StartObject);
                        }
                        '[' => {
                            self.stack.push(JSONSReaderStateEnum::ARRAY);
                            self.stack.push(JSONSReaderStateEnum::INIT{is_key});
                            self.callback(JSONSReaderEvent::StartArray);
                        }
                        '"' => {
                            self.stack.push(JSONSReaderStateEnum::STRING{is_key});
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
                                self.stack.push(JSONSReaderStateEnum::CONSTANT{is_key});
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
                            self.callback(JSONSReaderEvent::EndObject);
                        }
                        ':'  => {
                            if is_key {
                                self.stack.pop().unwrap();
                                self.stack.push(JSONSReaderStateEnum::OBJECT{is_key: false});
                                self.stack.push(JSONSReaderStateEnum::INIT{is_key: false});
                            } else {
                                self.stack.push(JSONSReaderStateEnum::INIT{is_key: false});
                            }
                            self.chars.clear();
                        }
                        ',' => {
                            self.stack.push(JSONSReaderStateEnum::INIT{is_key:true});
                        }
                        _ => {
                            if c.is_whitespace() {
                                continue;
                            }
                            if c == '"' {
                                self.stack.push(JSONSReaderStateEnum::STRING{is_key});
                                self.chars.clear();
                            } else {
                                self.chars.push(c);
                                self.stack.push(JSONSReaderStateEnum::CONSTANT {is_key});
                            }
                        }
                    }
                }
                JSONSReaderStateEnum::ARRAY => {
                    match c {
                        ']' => {
                            self.stack.pop().unwrap();
                            self.callback(JSONSReaderEvent::EndArray);
                        }
                        ',' => {
                            self.stack.push(JSONSReaderStateEnum::INIT{is_key: false});
                        }
                        _ => {
                            if c.is_whitespace() {
                                continue;
                            }
                            if c == '"' {
                                self.stack.pop().unwrap();
                                self.callback(JSONSReaderEvent::String(String::from_iter(&self.chars)));
                            } else {
                                return Err(JSONSReaderError::InvalidJSON);
                            }
                        }
                    }
                }
                JSONSReaderStateEnum::STRING{..} => {
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
                                self.callback(JSONSReaderEvent::String(String::from_iter(&self.chars)));
                                self.stack.pop().unwrap();
                            } else {
                                self.chars.push(c);
                            }
                        }
                    }
                }
                JSONSReaderStateEnum::CONSTANT{is_key} => {
                    if c.is_alphanumeric() {
                        self.chars.push(c);
                    } else if c == '(' {
                        self.stack.pop().unwrap();
                        self.stack.push(JSONSReaderStateEnum::TYPE);
                        self.stack.push(JSONSReaderStateEnum::INIT{is_key});
                        self.callback(JSONSReaderEvent::StartType(String::from_iter(&self.chars)));
                        self.chars.clear();
                    } else {
                        self.callback(JSONSReaderEvent::Constant(String::from_iter(&self.chars)));
                        self.stack.pop().unwrap();
                        reprocess_char = true;
                        self.chars.clear();
                    }
                }
                JSONSReaderStateEnum::TYPE => {
                    if c == ')' {
                        self.stack.pop().unwrap();
                        self.callback(JSONSReaderEvent::EndType);
                    } else {
                        self.callback(JSONSReaderEvent::Constant(String::from_iter(&self.chars)));
                        self.chars.clear();
                        self.stack.pop().unwrap();
                        reprocess_char = true;
                    }
                }
            }
        }
    
    }

    pub fn finish(&mut self) -> Result<(), JSONSReaderError> {
        if self.stack.is_empty() {
            return Err(JSONSReaderError::InvalidState);
        }
        match self.stack.pop().unwrap() {
            JSONSReaderStateEnum::INIT{..} => {
            }
            JSONSReaderStateEnum::OBJECT { .. } => {
                return Err(JSONSReaderError::InvalidJSON);
            }
            JSONSReaderStateEnum::ARRAY => {
                return Err(JSONSReaderError::InvalidJSON);
            }
            JSONSReaderStateEnum::STRING{..} => {
                return Err(JSONSReaderError::InvalidJSON);
            }
            JSONSReaderStateEnum::CONSTANT{is_key} => {
                if !is_key {
                    self.callback(JSONSReaderEvent::Constant(String::from_iter(&self.chars)));
                } else {
                    return Err(JSONSReaderError::InvalidJSON);
                }
            }
            JSONSReaderStateEnum::TYPE => {
                return Err(JSONSReaderError::InvalidJSON);
            }
        }
        if self.stack.len() >1 {
            return Err(JSONSReaderError::InvalidState);
        }
        self.callback(JSONSReaderEvent::Finished);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
