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
    buffer:     Vec<u8>,
    w_position: u64,
    r_position: u64
}

impl Buffer {
    fn new(len: usize) -> Buffer {
        Buffer{ buffer: vec![0u8; len], w_position: 0, r_position: 0 }
    }
    
    fn reset_position(&mut self) {
        self.w_position = 0;
        self.r_position = 0;
    }
}

impl Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut cursor = Cursor::new(&mut self.buffer[..]);
        cursor.set_position(self.w_position);
        
        let assign_w_position = &mut self.w_position;
        cursor.write(buf).map(|num_bytes| {
            *assign_w_position += num_bytes as u64;
            num_bytes
        })
    }
    
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Read for Buffer {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut cursor = Cursor::new(&mut self.buffer);
        cursor.set_position(self.r_position);
        
        let assign_r_position = &mut self.r_position;
        cursor.read(buf).map(|num_bytes| {
            *assign_r_position += num_bytes as u64;
            num_bytes
        })
    }
}

impl AsRef<[u8]> for Buffer {
    fn as_ref(&self) -> &[u8] {
        &self.buffer
    }
}