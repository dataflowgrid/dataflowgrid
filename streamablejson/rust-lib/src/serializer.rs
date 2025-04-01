/* This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich */

use std::error::Error;

use crate::StreamableJSONEntry;
use dataflowgrid_commons::cursedbuffer::{CursedBuffer, CursedBufferError};

#[derive(Debug)]
pub enum SerializerError {
    InvalidEntry,
    InvalidOutput,
    OutputError(Box<dyn Error>)
}

impl From<CursedBufferError> for SerializerError {
    fn from(value: CursedBufferError) -> Self {
        Self::OutputError(Box::new(value))
    }
}

pub struct StreamableJSONSerializer {

}

impl StreamableJSONSerializer {
    pub fn new() -> StreamableJSONSerializer {
        StreamableJSONSerializer {
        }
    }

    pub fn serialize(&self, entry: &StreamableJSONEntry, output: &CursedBuffer<char>) -> Result<(), SerializerError> {
        match entry {
            StreamableJSONEntry::Object(obj) => {
                output.write(vec!['{'])?;
                let mut first = true;
                for (key, value) in obj.iter() {
                    if !first {
                        output.write(vec![','])?;
                    }
                    self.serialize(key, output)?;
                    output.write(vec![':'])?;
                    self.serialize(value, output)?;
                    first = false;
                }
                output.write(vec!['}'])?;
            }
            StreamableJSONEntry::Array(arr) => {
                output.write(vec!['['])?;
                let mut first = true;
                for value in arr.iter() {
                    if !first {
                        output.write(vec![','])?;
                    }
                    self.serialize(value, output)?;
                    first = false;
                }
                output.write(vec![']'])?;
            }
            StreamableJSONEntry::String(s) => {
                output.write(vec!['\"'])?;
                output.write(s.chars().collect())?;
                output.write(vec!['\"'])?;
            }
            StreamableJSONEntry::Constant(s) => {
                output.write(s.chars().collect())?;
            }
            StreamableJSONEntry::Type(t, arr) => {
                output.write(t.chars().collect())?;
                output.write(vec!['('])?;
                let mut first = true;
                for value in arr.iter() {
                    if !first {
                        output.write(vec![','])?;
                    }
                    self.serialize(value, output)?;
                    first = false;
                }
                output.write(vec![')'])?;
            }
        }
        Ok(())
    }

    pub fn serialize_to_string(entry: &StreamableJSONEntry) -> Result<String, SerializerError> {
        let ser = StreamableJSONSerializer::new();
        let output = CursedBuffer::new();
        ser.serialize(entry, &output).unwrap();
        let reader = output.reader(0);
        let mut r = Vec::<char>::new();
        loop {
            let chunk = reader.next_chunk();
            match chunk {
                Ok(chunk) => {
                    r.extend(chunk.as_slice());
                }
                Err(_) => {
                    break;
                }
            }
        }
        Ok(String::from_iter(r))
    }

}

#[cfg(test)]
mod tests {
    use dataflowgrid_commons::orderedbag::OrderedBag;

    use super::*;
 
    #[test]
    fn test_array() {
        let r = StreamableJSONSerializer::serialize_to_string(
        &StreamableJSONEntry::Array(
            vec![
                StreamableJSONEntry::String("value1".to_string()),
                StreamableJSONEntry::String("value2".to_string()),
            ]
        ));
        assert_eq!(r.unwrap(), "[\"value1\",\"value2\"]");
    }

    #[test]
    fn test_type() {
        let r = StreamableJSONSerializer::serialize_to_string(
             &StreamableJSONEntry::Type("type".to_string(),
            vec![
                StreamableJSONEntry::Constant("value1".to_string()),
                StreamableJSONEntry::String("value2".to_string()),
            ]
        ));
        assert_eq!(r.unwrap(), "type(value1,\"value2\")");
    }

    #[test]
    fn test_string() {
        let r = StreamableJSONSerializer::serialize_to_string(
        &StreamableJSONEntry::String("value1".to_string()));
        assert_eq!(r.unwrap(), "\"value1\"");
    }

    #[test]
    fn test_object() {
        let mut k = OrderedBag::new();
        k.push(StreamableJSONEntry::String("value2".to_string()), StreamableJSONEntry::Constant("value1".to_string()));
        k.push(StreamableJSONEntry::String("value2".to_string()), StreamableJSONEntry::Constant("value3".to_string()));
        let r = StreamableJSONSerializer::serialize_to_string(
             &StreamableJSONEntry::Object(k)
        );
        assert_eq!(r.unwrap(), "{\"value2\":value1,\"value2\":value3}");
    }


}
