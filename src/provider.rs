use std::collections::{VecDeque};
use std::net::{SocketAddr};

use mio::{EventLoop, Sender, TimerResult, Timeout};

use buffer::{BufferPool, Buffer};
use dispatcher::{Dispatcher, DispatchHandler};

pub struct Provider<'a, 'b: 'a, D: Dispatcher + 'b> {
    buffer_pool: &'a mut BufferPool,
    out_queue:   &'a mut VecDeque<(Buffer, SocketAddr)>,
    event_loop:  &'a mut EventLoop<DispatchHandler<'b, D>>
}

impl<'a, 'b: 'a, D: Dispatcher + 'b> Provider<'a, 'b, D> {
    pub fn new(buffer_pool: &'a mut BufferPool, out_queue: &'a mut VecDeque<(Buffer, SocketAddr)>,
        event_loop: &'a mut EventLoop<DispatchHandler<'b, D>>) -> Provider<'a, 'b, D> {
        Provider{ buffer_pool: buffer_pool, out_queue: out_queue, event_loop: event_loop }   
    }

    pub fn channel(&self) -> Sender<D::Message> {
        self.event_loop.channel()
    }
    
    pub fn outgoing<F>(&mut self, out: F)
        where F: FnOnce(Buffer) -> Option<(Buffer, SocketAddr)> {
        let buffer = self.buffer_pool.pop();
        
        if let Some(message) = out(buffer) {
            self.out_queue.push_back(message);
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