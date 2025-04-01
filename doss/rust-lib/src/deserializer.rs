/* This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich */

use std::{collections::BTreeMap, io::Result};
use tokio::io::AsyncReadExt;
use dataflowgrid_commons::orderedbag::OrderedBag;

#[derive(Debug)]
pub enum DossLowLevelStreamEvent {
    NoOp = 0,
    True = 1,
    False = 2,
    Varint = 3,
    Decimal = 4,
    Float = 5,
    DateTime = 6,
    String = 7,
    Binary = 8,
    Reference = 9,
    BlockStart = 10,
    BlockEnd = 11,
    ArrayStart = 12,
    ArrayEnd = 13,
    NULL = 14,
    UnsignedVarint = 15,
    TypeStart = 16,
    TypeEnd = 17,

    SetConfig = 20,
    StoreInDict = 21,
    StoreButDontUse = 22,
    SetDictPointer = 23,
    ClearDictEntries = 24,
    StackStart = 25,
    StackEnd = 26,
    SetHint = 27,

    SkipBytes16le = 30,
    SkipBytes32le = 31,

    ImportDict = 40,

    FileStart = 50,
}

pub trait DossLowLevelStream {
    async fn doss_event(&self, event: &DossLowLevelStreamEvent);
}

struct DossLowLevelStreamConsoleImpl {
}
impl DossLowLevelStream for DossLowLevelStreamConsoleImpl {
    async fn doss_event(&self, event: &DossLowLevelStreamEvent) {
        println!("Event: {:?}", event);
    }
}

struct Deserializer<'a> {
    dict: BTreeMap<usize, &'a str>
}

impl<'a> Deserializer<'a> {
    fn new() -> Deserializer<'a> {
        Deserializer {
            dict: BTreeMap::new()
        }
    }

    async fn deserialize<T: AsyncReadExt+Unpin, P:DossLowLevelStream>(&self, mut reader: T, processor: P) -> Result<()>{
        loop {
            let b = reader.read_u8().await;
            match b {
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Result::Err(e),
                Ok(opcode) => {
                    let event = match opcode {
                        0 => DossLowLevelStreamEvent::NoOp,
                        1 => DossLowLevelStreamEvent::True,
                        2 => DossLowLevelStreamEvent::False,
                        3 => DossLowLevelStreamEvent::Varint,

                        4 => DossLowLevelStreamEvent::Decimal,
                        5 => DossLowLevelStreamEvent::Float,
                        6 => DossLowLevelStreamEvent::DateTime,
                        7 => DossLowLevelStreamEvent::String,
                        8 => DossLowLevelStreamEvent::Binary,
                        9 => DossLowLevelStreamEvent::Reference,
                        10 => DossLowLevelStreamEvent::BlockStart,
                        11 => DossLowLevelStreamEvent::BlockEnd,
                        12 => DossLowLevelStreamEvent::ArrayStart,
                        13 => DossLowLevelStreamEvent::ArrayEnd,
                        14 => DossLowLevelStreamEvent::NULL,
                        15 => DossLowLevelStreamEvent::UnsignedVarint,
                        20 => DossLowLevelStreamEvent::SetConfig,
                        21 => DossLowLevelStreamEvent::StoreInDict,
                        22 => DossLowLevelStreamEvent::StoreButDontUse,
                        23 => DossLowLevelStreamEvent::SetDictPointer,
                        24 => DossLowLevelStreamEvent::ClearDictEntries,
                        25 => DossLowLevelStreamEvent::StackStart,
                        26 => DossLowLevelStreamEvent::StackEnd,
                        27 => DossLowLevelStreamEvent::SetHint,
                        30 => DossLowLevelStreamEvent::SkipBytes16le,
                        31 => DossLowLevelStreamEvent::SkipBytes32le,
                        40 => DossLowLevelStreamEvent::ImportDict,
                        50 => DossLowLevelStreamEvent::FileStart,
                        _ => return Result::Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid event"))
                    };
                    processor.doss_event(&event).await;
                }
            }
        }
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn deserialize_empty() {
        let deserializer = Deserializer::new();
        let serialized = [1_u8,2,3,4].as_slice();
        let logger = DossLowLevelStreamConsoleImpl {};
        let deserialized = deserializer.deserialize(serialized, logger).await;
        deserialized.unwrap()
    }
}
