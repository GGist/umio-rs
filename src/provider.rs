use std::collections::{VecDeque};
use std::net::{SocketAddr};

use mio::{EventLoop, Sender, TimerResult, Timeout};

use buffer::{BufferPool, Buffer};
use dispatcher::{Dispatcher, DispatchHandler};

pub struct Provider<'a, D: Dispatcher + 'a> {
    buffer_pool: &'a mut BufferPool,
    out_queue:   &'a mut VecDeque<(Buffer, SocketAddr)>,
    event_loop:  &'a mut EventLoop<DispatchHandler<'a, D>>
}

impl<'a, D: Dispatcher + 'a> Provider<'a, D> {
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