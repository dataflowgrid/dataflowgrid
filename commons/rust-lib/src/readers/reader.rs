/* This file is part of dataFlowGrid. See file LICENSE for full license details. (c) 2025 Alexander Zich */
use std::sync::Arc;

use crate::cursedbuffer::CursedBufferReader;


#[derive(Debug)]
pub enum ReaderError {
    EOF,
    IO(std::io::Error),
}

pub trait Readable<T> {
    fn read_next(&mut self) -> Result<T, ReaderError> where T: Copy;
    fn skip(&mut self, skipped: usize) -> Result<usize, ReaderError>;
    fn read_chunk(&mut self) -> Result<ReadableChunk<T>, ReaderError>;
    fn pos(&self) -> Option<usize>;
    fn len(&self) -> Option<usize>;
}

pub struct CursedBufferReadable<T> {
    reader: CursedBufferReader<T>,
    current_chunk: Option<Arc<Vec<T>>>,
    current_chunk_pos: usize,
    current_chunk_len: usize,
}

impl<T> CursedBufferReadable<T> {
    pub fn new(reader: CursedBufferReader<T>) -> Self {
        CursedBufferReadable { 
            reader,
            current_chunk: None,
            current_chunk_pos: 0,
            current_chunk_len: 0,
        }
    }
}

pub struct ReadableChunk<T> {
    chunk: Arc<Vec<T>>,
    pos: usize,
    len: usize
}

impl<T> Readable<T> for CursedBufferReadable<T> {
    fn read_next(&mut self) -> Result<T, ReaderError> where T: Copy {
        if self.current_chunk.is_none() {
            match self.read_chunk() {
                Ok(chunk) => {
                    self.current_chunk = Some(chunk.chunk);
                    self.current_chunk_pos = chunk.pos;
                    self.current_chunk_len = chunk.len;
                }
                Err(e) => return Err(e)
            }
        }

        match self.current_chunk {
            Some(ref chunk) => {
                let r = chunk[self.current_chunk_pos];
                self.current_chunk_pos += 1;
                if self.current_chunk_pos >= self.current_chunk_len {
                    self.current_chunk = None;
                }
                Ok(r)
            }
            None => Err(ReaderError::EOF)
        }
    }

    fn skip(&mut self, skipped: usize) -> Result<usize, ReaderError> {
        //skip might be realizable in current_chunk, so we try that first
        if self.current_chunk.is_some() {
            let remaining = self.current_chunk_len - self.current_chunk_pos;
            if skipped <= remaining {
                self.current_chunk_pos += skipped;
                return Ok(skipped);
            } else {
                self.current_chunk = None;
                self.reader.skip(skipped);
                return Ok(skipped);
            }
        } else {
            self.reader.skip(skipped);
            return Ok(skipped);
        }
    }

    fn read_chunk(&mut self) -> Result<ReadableChunk<T>, ReaderError> {
        if self.current_chunk.is_none() {
            let chunk = self.reader.next_chunk();
            match chunk {
                Ok(chunk) => {
                    self.current_chunk = Some(chunk.slice);
                    self.current_chunk_pos = chunk.pos;
                    self.current_chunk_len = chunk.len;
                }
                Err(_) => return Err(ReaderError::EOF),
            }
        }
        let r = ReadableChunk {
            chunk: std::mem::replace(&mut self.current_chunk, None).unwrap(), //we know this is save because of the previous check
            pos: self.current_chunk_pos,
            len: self.current_chunk_len
        };
        Ok(r)
    }

    fn pos(&self) -> Option<usize> {
        Some(self.reader.pos())
    }

    fn len(&self) -> Option<usize> {
        None
    }
}

pub struct IteratorReadable<T> {
    iter: Box<dyn Iterator<Item=T>>,
    pos: usize,
}

impl<T> IteratorReadable<T> {
    pub fn new(iter: Box<dyn Iterator<Item=T>>)  -> Self {
        IteratorReadable {
            iter,
            pos: 0,
        }
    }
}

impl<T> Readable<T> for IteratorReadable<T> where T: Copy {
    fn read_next(&mut self) -> Result<T, ReaderError> {
        match self.iter.next() {
            Some(v) => {
                self.pos += 1;
                Ok(v)
            }
            None => Err(ReaderError::EOF)
        }
    }

    fn skip(&mut self, skipped: usize) -> Result<usize, ReaderError> {
        for _ in 0..skipped {
            match self.iter.next() {
                Some(_) => self.pos += 1,
                None => return Ok(self.pos)
            }
        }
        Ok(self.pos)
    }

    fn read_chunk(&mut self) -> Result<ReadableChunk<T>, ReaderError> {
        let n = self.iter.next();
        match n {
            None => return Err(ReaderError::EOF),
            Some(v) => {
                Ok(ReadableChunk {
                    chunk: Arc::new(vec![v]),
                    pos: 0,
                    len: 1
                })
            }
        }
    }

    fn pos(&self) -> Option<usize> {
        Some(self.pos)
    }

    fn len(&self) -> Option<usize> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cursedbuffer::CursedBuffer;

    #[test]
    fn test_cursedbuffer_readable1() {

        let buffer = CursedBuffer::<u8>::new();
        buffer.write(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).unwrap();
        let reader = buffer.reader(0);
        let mut readable = CursedBufferReadable::new(reader);

        readable.skip(2).unwrap();
        assert_eq!(readable.read_next().unwrap(), 3);
        assert_eq!(readable.read_next().unwrap(), 4);
        readable.skip(2).unwrap();
        assert_eq!(readable.read_next().unwrap(), 7);
        assert_eq!(readable.read_next().unwrap(), 8);

    }

    #[test]
    fn test_cursedbuffer_readable2() {

        let buffer = CursedBuffer::<u8>::new();
        buffer.write(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).unwrap();
        let reader = buffer.reader(0);
        let mut readable = CursedBufferReadable::new(reader);

        readable.skip(2).unwrap();
        assert_eq!(readable.read_next().unwrap(), 3);
        let b = readable.read_chunk().unwrap();
        assert_eq!(b.len, 10);
        assert_eq!(b.pos, 3);
        assert_eq!(b.chunk[b.pos], 4);
    }

    #[test]
    fn test_cursedbuffer_readable3() {

        let buffer = CursedBuffer::<u8>::new();
        buffer.write(vec![1, 2]).unwrap();
        let reader = buffer.reader(0);
        let mut readable = CursedBufferReadable::new(reader);

        readable.skip(2).unwrap();
        buffer.write(vec![3, 4, 5, 6, 7, 8, 9, 10]).unwrap();

        assert_eq!(readable.read_next().unwrap(), 3);
        let b = readable.read_chunk().unwrap();
        assert_eq!(b.len, 8);
        assert_eq!(b.pos, 1);
        assert_eq!(b.chunk[b.pos], 4);
    }

}