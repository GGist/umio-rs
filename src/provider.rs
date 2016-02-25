use std::collections::{VecDeque};
use std::net::{SocketAddr};

use mio::{EventLoop, Sender, TimerResult, Timeout};

use buffer::{BufferPool, Buffer};
use dispatcher::{Dispatcher, DispatchHandler};

pub struct Provider<'a, D: Dispatcher + 'a> {
    buffer_pool: &'a mut BufferPool,
    out_queue:   &'a mut VecDeque<(Buffer, SocketAddr)>,
    event_loop:  &'a mut EventLoop<DispatchHandler<D>>
}

pub fn new<'a, D: Dispatcher>(buffer_pool: &'a mut BufferPool, out_queue: &'a mut VecDeque<(Buffer, SocketAddr)>,
    event_loop: &'a mut EventLoop<DispatchHandler<D>>) -> Provider<'a, D> {
    Provider{ buffer_pool: buffer_pool, out_queue: out_queue, event_loop: event_loop }
}

impl<'a, D: Dispatcher> Provider<'a, D> {
    pub fn channel(&self) -> Sender<D::Message> {
        self.event_loop.channel()
    }
    
    pub fn outgoing<F>(&mut self, out: F)
        where F: FnOnce(&mut Buffer) -> Option<SocketAddr> {
        let mut buffer = self.buffer_pool.pop();
        let opt_send_to = out(&mut buffer);
        
        match opt_send_to {
            Some(addr) => self.out_queue.push_back((buffer, addr)),
            None       => self.buffer_pool.push(buffer)
        }
    }
    
    pub fn set_timeout(&mut self, token: D::Timeout, delay: u64) -> TimerResult<Timeout> {
        self.event_loop.timeout_ms(token, delay)
    }
    
    pub fn clear_timeout(&mut self, timeout: Timeout) -> bool {
        self.event_loop.clear_timeout(timeout)
    }
    
    pub fn shutdown(&mut self) {
        self.event_loop.shutdown()
    }
}