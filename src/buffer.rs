use std::io::{Write, Read, Cursor, Result};

pub struct BufferPool {
    // Use Stack For Temporal Locality
    buffers:     Vec<Buffer>,
    buffer_size: usize
}

impl BufferPool {
    pub fn new(buffer_size: usize) -> BufferPool {
        let buffers = Vec::new();
        
        BufferPool{ buffers: buffers, buffer_size: buffer_size }
    }
    
    pub fn pop(&mut self) -> Buffer {
        self.buffers.pop().unwrap_or(Buffer::new(self.buffer_size))
    }
    
    pub fn push(&mut self, mut buffer: Buffer) {
        buffer.reset_position();
        
        self.buffers.push(buffer);
    }
}

//----------------------------------------------------------------------------//

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