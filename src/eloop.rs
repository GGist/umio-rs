const DEFAULT_MAX_BUFFERS: usize = 4;
const DEFAULT_BUFFER_SIZE: usize = 1500;

struct ELoopBuilder {
    max_buffers:  usize,
    buffer_size:  usize,
    bind_address: SocketAddr
}

impl ELoopBuilder {
    pub fn new() -> ELoopBuilder {
        let default_addr = 
        
        ELoopBuilfer{ max_buffers: DEFAULT_MAX_BUFFERS, buffer_size: DEFAULT_BUFFER_SIZE }
    }
    
    pub fn bind_address(self, )
    
    pub fn max_buffers(self, max: usize) -> ELoopBuilder {
        self.max_buffers = max;
        
        self
    }
    
    pub fn buffer_length(self, length: usize) -> ELoopBuilder {
        self.buffer_size = length;
        
        self
    }
    
    pub fn build() -> ELoop {
        
    }
}

//----------------------------------------------------------------------------//

struct ELoop {
    buffer_pool: BufferPool
}

impl ELoop {
    fn from_builder(builder: ELoopBuilder) -> ELoop {
        let buffer_pool = BufferPool::new(builder.buffer_size, builder.max_buffers);
        
        ELoop{ buffer_pool: buffer_pool }
    }
    
    pub fn channel(&self) -> Sender<
}

