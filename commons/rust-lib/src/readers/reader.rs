use tokio::io::AsyncReadExt;

use crate::decoders::{self, decoders::TextDecoder};

pub enum ReaderError {
}
/// A trait to read chars from a stream
pub trait AsyncReader {
    async fn read_chunk(&self, chars: &[char]) -> Result<usize, ReaderError>;
    async fn next_char(&self) -> char;
    async fn peek_char(&self) -> char;
    async fn skip(&self, n: usize);
}

struct DefaultAsyncReader<I: AsyncReadExt, D: TextDecoder> {
    input: I,
    decoder: D,
    buffersize: u32
}

impl<I: AsyncReadExt,D: TextDecoder> DefaultAsyncReader<I,D> {
    pub fn new(input: I, decoder: D, buffersize:u32) -> DefaultAsyncReader<I,D> {
        DefaultAsyncReader {
            input,
            decoder,
            buffersize
        }
    }
}

