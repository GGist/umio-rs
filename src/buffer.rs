use std::sync::atomic::{AtomicUsize, Ordering};

use crossbeam::sync::{MsQueue};

pub struct BufferPool {
    buffers:      MsQueue<Buffer>,
    buffer_size:  usize,
    max_buffers:  usize,
    curr_buffers: AtomicUsize
}

impl BufferPool {
    pub fn new(buffer_size: usize, max_buffers: usize) -> BufferPool {
        BufferPool{ buffers: MsQueue::new(), buffer_size: buffer_size,
            max_buffers: max_buffers, curr_buffers: AtomicUsize::new(0) }
    }
    
    pub fn pop(&self) -> Buffer {
        // Try to get a buffer, otherwise see if we can create more.
        let opt_buffer = self.buffers.try_pop().or_else(|| {
            let new_num_buffers = self.curr_buffers.fetch_add(1, Ordering::AcqRel);
        
            // If we see that our new increment is less than or equal to
            // the max, we are free to create a new buffer and return it.
            // Otherwise, we should set the curr_buffers back to the max.
            if new_num_buffers <= self.max_buffers {
                Some(Buffer::new(self.buffer_size))
            } else {
                self.curr_buffers.store(self.max_buffers, Ordering::AcqRel);
                
                None
            }
        });
    
        // Return a buffer now or block until one is available.
        match opt_buffer {
            Some(buffer) => buffer,
            None         => self.buffers.pop()
        }
    }
    
    pub fn push(&self, mut buffer: Buffer) {
        buffer.reset_position();
        
        self.buffers.push(buffer);
    }
}

//----------------------------------------------------------------------------//

/// Reusable region of memory for incoming and outgoing messages.
pub struct Buffer {
    buffer:        Vec<u8>,
    bytes_written: usize
}

impl Buffer {
    fn new(len: usize) -> Buffer {
        Buffer{ buffer: vec![0u8; len], bytes_written: 0 }
    }
    
    fn reset_position(&mut self) {
        self.set_written(0);
    }
    
    /// Update the number of bytes written to the buffer.
    pub fn set_written(&mut self, bytes: usize) {
        self.bytes_written = bytes;
    }
}

impl AsRef<[u8]> for Buffer {
    fn as_ref(&self) -> &[u8] {
        &self.buffer[..self.bytes_written]
    }
}

impl AsMut<[u8]> for Buffer {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[self.bytes_written..]
    }
}