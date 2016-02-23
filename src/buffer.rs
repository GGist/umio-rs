pub struct BufferPool {
    // Use Stack For Temporal Locality
    buffers: TreiberStack<Vec<u8>>
}

impl BufferPool {
    pub fn new(buffer_len: usize, num_buffers: usize) -> BufferPool {
        let mut buffers = TreiberStack::new();
        
        for _ in 0..num_buffers {
            buffers.push(Buffer::new(buffer_len));
        }
        
        BufferPool{ buffers: buffers }
    }
    
    pub fn pop(&self) -> Buffer {
        self.buffers.pop()
    }
    
    pub fn push(&self, mut buffer: Buffer) {
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
        let mut cursor = Cursor::new(&mut self.buffer);
        cursor.set_position(self.w_position);
        
        cursor.write(buf).map(|num_bytes| {
            self.w_position += (num_bytes as u64);
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
        
        cursor.read(buf).map(|num_bytes| {
            self.r_position += (num_bytes as u64);
            num_bytes
        })
    }
}

impl AsRef<[u8]> for Buffer {
    fn as_ref(&self) -> &[u8] {
        &self.buffer
    }
}