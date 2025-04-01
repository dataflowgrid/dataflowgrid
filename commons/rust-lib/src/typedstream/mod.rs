#![allow(dead_code)]

use crate::orderedbag::OrderedBag;

#[derive(Debug)]
pub enum TypedStreamEvent {
    INIT,
    STARTOBJECT,
    ENDOBJECT,
    STARTARRAY,
    ENDARRAY,
    STARTTYPE(String),
    ENDTYPE,
    STRING(String),
    DECIMAL(usize),
    NULL,
    TRUE,
    FALSE,
    FINISH,

    BYTEARRAY(Vec<u8>),
    HINT(String),
    ANY(Box<dyn std::any::Any>),

    ERROR(Box<dyn std::error::Error>),
}

#[derive(Debug)]
pub enum TypedStreamEventReturn {
    CONTINUE,
    STOP,
    SKIP,
    ERROR,
}

#[derive(Debug)]
pub enum TypedStreamEventError {
    InvalidEvent,
    InvalidType,
    InvalidValue,
    InvalidState,
}

pub type TypedStreamEventHandler = fn(TypedStreamEvent) -> Result<TypedStreamEventReturn, TypedStreamEventError>;

#[derive(Debug)]
pub enum TypedStreamElement {
    Object(OrderedBag<TypedStreamElement, TypedStreamElement>),
    Array(Vec<TypedStreamElement>),
    Type(String, Vec<TypedStreamElement>),
    String(String),
    Decimal,
    Null,
    Boolean(bool),
    ByteArray(Vec<u8>),
    Any(Box<dyn std::any::Any>),
    Error(Box<dyn std::error::Error>),
}

impl PartialEq<TypedStreamElement> for TypedStreamElement {
    fn eq(&self, other: &TypedStreamElement) -> bool {
        match (self, other) {
            (TypedStreamElement::String(s1), TypedStreamElement::String(s2)) => s1 == s2,
            (TypedStreamElement::Boolean(b1), TypedStreamElement::Boolean(b2)) => b1 == b2,
            (TypedStreamElement::Null, TypedStreamElement::Null) => true,
            (TypedStreamElement::ByteArray(b1), TypedStreamElement::ByteArray(b2)) => b1 == b2,
            (TypedStreamElement::Object(o1), TypedStreamElement::Object(o2)) => o1 == o2,
            (TypedStreamElement::Array(a1), TypedStreamElement::Array(a2)) => a1 == a2,
            (TypedStreamElement::Type(t1, v1), TypedStreamElement::Type(t2, v2)) => t1 == t2 && v1 == v2,
            _ => false,
            // TODO: Implement other comparisons
        }
    }
}


pub struct TypeStream2OrderedMultiDictProcessor {
    stack: Vec<TypedStreamElement>,
    results: Vec<TypedStreamElement>,
}

impl TypeStream2OrderedMultiDictProcessor {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            results: Vec::new(),
        }
    }

    #[inline]
    fn insert_into_last_stack_element(&mut self, element: TypedStreamElement) -> Result<TypedStreamEventReturn, TypedStreamEventError>{
        if let Some(last) = self.stack.last_mut() {
            match last {
                TypedStreamElement::Object(obj) => {
                    if obj.keys_and_values_in_sync() {
                        obj.insert_key_only(element);
                    } else {
                        obj.insert_value_only(element);
                    }
            }
                TypedStreamElement::Array(arr) => {
                    arr.push(element);
                }
                TypedStreamElement::Type(_, t) => {
                    t.push(element);
                }
                _ => panic!("Unexpected stack element: {:?}", last),
            }
            Ok(TypedStreamEventReturn::CONTINUE)
        } else {
            //insert into result vec
            self.results.push(element);
            Ok(TypedStreamEventReturn::CONTINUE)
        }
    }

    pub fn process(&mut self, event: TypedStreamEvent) -> Result<TypedStreamEventReturn, TypedStreamEventError> {
        match event {
            TypedStreamEvent::INIT => {
                if self.stack.is_empty() {
                    Ok(TypedStreamEventReturn::CONTINUE)
                } else {
                    Err(TypedStreamEventError::InvalidState)
                }
            },
            TypedStreamEvent::STARTOBJECT => {
                self.stack.push(TypedStreamElement::Object(OrderedBag::new()));
                Ok(TypedStreamEventReturn::CONTINUE)
            },
            TypedStreamEvent::ENDOBJECT | TypedStreamEvent::ENDARRAY | TypedStreamEvent::ENDTYPE => {
                let o = self.stack.pop().unwrap();
                self.insert_into_last_stack_element(o)
            },
            TypedStreamEvent::STARTARRAY => {
                 self.stack.push(TypedStreamElement::Array(Vec::new()));
                 Ok(TypedStreamEventReturn::CONTINUE)
            }
            TypedStreamEvent::STARTTYPE(s) => {
                self.stack.push(TypedStreamElement::Type(s, Vec::new()));
                Ok(TypedStreamEventReturn::CONTINUE)
            }
            TypedStreamEvent::STRING(s) => self.insert_into_last_stack_element(TypedStreamElement::String(s)),
            TypedStreamEvent::DECIMAL(_) => Ok(TypedStreamEventReturn::CONTINUE),
            TypedStreamEvent::NULL => self.insert_into_last_stack_element(TypedStreamElement::Null),
            TypedStreamEvent::TRUE => self.insert_into_last_stack_element(TypedStreamElement::Boolean(true)),
            TypedStreamEvent::FALSE => self.insert_into_last_stack_element(TypedStreamElement::Boolean(false)),
            TypedStreamEvent::FINISH => {
                if !self.stack.is_empty() {
                    Err(TypedStreamEventError::InvalidState)
                } else {
                    Ok(TypedStreamEventReturn::STOP)
                }
            },
            TypedStreamEvent::BYTEARRAY(b) => self.insert_into_last_stack_element(TypedStreamElement::ByteArray(b)),
            TypedStreamEvent::HINT(_) => {
                Ok(TypedStreamEventReturn::CONTINUE)
            }
            TypedStreamEvent::ANY(v) => self.insert_into_last_stack_element(TypedStreamElement::Any(v)),
            TypedStreamEvent::ERROR(e) => self.insert_into_last_stack_element(TypedStreamElement::Error(e)),
        }
    }

    pub fn get_result(&mut self) -> Option<TypedStreamElement> {
        if self.results.is_empty() {
            None
        } else {
            Some(self.results.remove(0))
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_object() {
        let mut processor = TypeStream2OrderedMultiDictProcessor::new();
        processor.process(TypedStreamEvent::INIT).unwrap();
        processor.process(TypedStreamEvent::STARTOBJECT).unwrap();
        processor.process(TypedStreamEvent::STRING("key".to_string())).unwrap();
        processor.process(TypedStreamEvent::STRING("value".to_string())).unwrap();
        processor.process(TypedStreamEvent::ENDOBJECT).unwrap();
        processor.process(TypedStreamEvent::FINISH).unwrap();

        let result = processor.get_result();
        assert!(result.is_some());
    }

    #[test]
    fn test_2_strings() {
        let mut processor = TypeStream2OrderedMultiDictProcessor::new();
        processor.process(TypedStreamEvent::INIT).unwrap();
        processor.process(TypedStreamEvent::STRING("key".to_string())).unwrap();
        processor.process(TypedStreamEvent::STRING("value".to_string())).unwrap();
        processor.process(TypedStreamEvent::FINISH).unwrap();

        assert_eq!(processor.get_result().unwrap(), TypedStreamElement::String("key".to_owned()));
        assert_eq!(processor.get_result().unwrap(), TypedStreamElement::String("value".to_owned()));
        assert!(processor.get_result().is_none());
    }
}