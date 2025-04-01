#![allow(dead_code)]
#![allow(unused_imports)]
use std::{clone, fs::read, future, ops::{Deref, DerefMut}, rc::Rc, sync::{Arc, Mutex, Weak}};
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use derive_more::{Display, Error};
use tokio::sync::Notify;


//internal state structs

enum Callback {
    None,
    Function(Box<dyn Fn() + Send>)
}

impl Debug for Callback {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Callback::None => write!(f, "None"),
            Callback::Function(_) => write!(f, "Function")
        }
    }
}
#[derive(Debug)]
struct CursedBufferInternalState<T> {
    first_read_position: usize, //the first element's of the first slice of buffers position
    buffers: Vec<Arc<Vec<T>>>,
    is_closed: bool,
    all_readers: Vec<Option<CursedBufferReaderInternalState>>,
    callback_function: Callback,
    notify_written_async: Arc<Notify>
}

#[derive(Debug)]
struct CursedBufferReaderInternalState {
    position: usize,
}

// public structs

#[derive(Debug)]
pub struct CursedBuffer<T> {
    bufferstate: Arc<Mutex<CursedBufferInternalState<T>>>,
}

#[derive(Debug)]
pub struct CursedBufferReader<T> {
    bufferstate: Arc<Mutex<CursedBufferInternalState<T>>>,
    reader_in_buffer: usize,
}

#[derive(Debug)]
pub struct CursedChunk<T> {
    pub slice: Arc<Vec<T>>,
    pub pos: usize,
    pub len: usize,
}


#[derive(Debug, PartialEq, Display, Error)]
pub enum CursedBufferError {
    NotEnoughData,
    InvalidData,
    IoError,
    BufferClosed
}

// implementations

impl<T> Clone for CursedBuffer<T> {
    fn clone(&self) -> Self {
        CursedBuffer {
            bufferstate: self.bufferstate.clone()
        }
    }
}

impl<T> CursedBuffer<T> {
    pub fn new() -> CursedBuffer<T> {
        let state = CursedBufferInternalState::<T> {
            first_read_position: 0,
            buffers: Vec::new(),
            all_readers: Vec::new(),
            is_closed: false,
            callback_function: Callback::None,
            notify_written_async: Arc::new(Notify::new())
        };
        let bufferstate = Arc::new(Mutex::new(state));
        
        CursedBuffer {
            bufferstate
        }
    }

    /// Sets a callback function that will be called whenever some data is finally removed from the buffer
    pub fn set_callback(&mut self, callback: Box<dyn Fn() + Send>) {
        let mut bufferstate = self.bufferstate.lock().unwrap();
        bufferstate.callback_function = Callback::Function(callback);
    }

    pub fn close(&self) {
        let mut bufferstate = self.bufferstate.lock().unwrap();
        bufferstate.is_closed = true;
    }

    pub fn reader(&self, position: usize) -> CursedBufferReader<T> {
        let mut bufferstate = self.bufferstate.lock().unwrap();

        let readerstate = CursedBufferReaderInternalState {
            position: position,
        };

        bufferstate.all_readers.push(Some(readerstate));

        CursedBufferReader {
            bufferstate: self.bufferstate.clone(),
            reader_in_buffer: bufferstate.all_readers.len()-1
        }
    }

    pub fn write(&self, data: Vec<T>) -> Result<(), CursedBufferError> {
        let mut s = self.bufferstate.lock().unwrap();
        if s.is_closed {
            return Err(CursedBufferError::BufferClosed);
        }
        s.buffers.push(Arc::new(data));
        s.notify_written_async.notify_waiters();
        Ok(())
    }

    pub async fn awrite(&self, data: Vec<T>) -> Result<(), CursedBufferError> {
        self.write(data)
    }

}

impl<T> CursedBufferInternalState<T> {
    fn sync_reader_states(&mut self) {
        //we basically look for the lowest position of all readers and whether we can therefor remove some buffers
        let mut min_pos = usize::MAX;
        for r in self.all_readers.iter() {
            if let Some(r) = r {
                if r.position < min_pos {
                    min_pos = r.position;
                }
            }
        }
        let mut can_delete = min_pos - self.first_read_position;
        let mut call = false;
        while !self.buffers.is_empty() && self.buffers[0].len() <= can_delete {
            can_delete -= self.buffers[0].len();
            self.first_read_position += self.buffers[0].len();
            self.buffers.remove(0);
            call = true;
        }
        if let Callback::Function(callback) = &self.callback_function {
            if call {
                callback();
            }
        }
}
}

impl<T> Drop for CursedBufferReader<T> {
    fn drop(&mut self) {
        let mut bufferstate = self.bufferstate.lock().unwrap();
        bufferstate.all_readers[self.reader_in_buffer] = None;
        bufferstate.sync_reader_states();
    }
}


impl<T> CursedBufferReader<T> {
    pub fn pos(&self) -> usize {
        let bufferstate = self.bufferstate.lock().unwrap();
        bufferstate.all_readers[self.reader_in_buffer].as_ref().unwrap().position
    }

    pub fn skip(&self, len:usize) {
        let mut bufferstate = self.bufferstate.lock().unwrap();
        bufferstate.all_readers[self.reader_in_buffer].as_mut().unwrap().position += len;
        bufferstate.sync_reader_states();
    }

    pub fn next_chunk(&self) -> Result<CursedChunk<T>, CursedBufferError> {
        let mut bufferstate = self.bufferstate.lock().unwrap();

        //our own position is in internalstate.pos
        //now we need to find the next buffer which holds that data
        let internalstate = bufferstate.all_readers[self.reader_in_buffer].as_ref().unwrap();
        let mut still_to_go = internalstate.position - bufferstate.first_read_position;

        if bufferstate.buffers.len() == 0 {
            return Err(CursedBufferError::NotEnoughData);
        }

        let mut i = 0;
        while still_to_go >= bufferstate.buffers[i].len() {
            still_to_go -= bufferstate.buffers[i].len();
            i += 1;
            if i >= bufferstate.buffers.len() {
                return Err(CursedBufferError::NotEnoughData);
            }
        }

        bufferstate.all_readers[self.reader_in_buffer].as_mut().unwrap().position += bufferstate.buffers[i].len();

        let r = CursedChunk{
            slice: bufferstate.buffers[i].clone(),
            pos: still_to_go,
            len: bufferstate.buffers[i].len()
        };
        bufferstate.sync_reader_states();
        Ok(r)
    }

    pub async fn anext_chunk(&self) -> Result<CursedChunk<T>, CursedBufferError> {
        let n = self.bufferstate.lock().unwrap().notify_written_async.clone();
        loop {
            let n1 = n.notified();
            let chunk = self.next_chunk();
            match chunk {
                Ok(chunk) => return Ok(chunk),
                Err(CursedBufferError::NotEnoughData) => {
                    n1.await
                }
                Err(e) => return Err(e)
            }
        }
    }

}


impl<T> Deref for CursedChunk<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<'a,T> CursedChunk<T> {
    pub fn as_slice(&self) -> &[T] {
        &self.slice.as_ref()[self.pos..]
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::{OnceCell, RefCell}, sync::RwLock};

    use super::*;

    #[test]
    fn it_works() {
        let b = CursedBuffer::<u8>::new();
        b.write(vec![1, 2, 3, 4, 5]).unwrap();
        let r = b.reader(0);
        let x = r.next_chunk();
        println!("{:?}",x);
        let x = r.next_chunk();
        println!("{:?}",x);
        b.write(vec![1, 2, 3]).unwrap();
        let x = r.next_chunk();
        println!("{:?}",x);
        b.write(vec![1, 2, 3, 4]).unwrap();
        r.skip(5);
        b.write(vec![1, 2, 3, 5]).unwrap();
        let x = r.next_chunk();
        println!("{:?}",x.unwrap().as_slice());
        let x = r.next_chunk();
        println!("{:?}",x);
    }

    #[test]
    fn test_close() {
        let b = CursedBuffer::<u8>::new();
        b.write(vec![1, 2, 3, 4, 5]).expect("");
        b.close();
        b.write(vec![1, 2, 3, 4, 5]).expect_err("");
    }

    #[test]
    fn test_callback() {
        let mut b = CursedBuffer::<u8>::new();
        let c = Arc::new(RwLock::<usize>::new(0));
        let c2 = c.clone();
        b.set_callback(Box::new(move || {
            println!("Callback called");
            *c2.write().unwrap().deref_mut() += 1;
        }));
        let _ = b.write(vec![1, 2, 3, 4, 5]);
        let r = b.reader(0);
        let x = r.next_chunk();
        println!("{:?}",x);
        assert_eq!(*c.read().unwrap(), 1);
    }
}
