use std::collections::{VecDeque};
use std::net::{SocketAddr};

use mio::{EventLoop, Sender, TimerResult, Timeout};

use buffer::{BufferPool, Buffer};
use dispatcher::{Dispatcher, DispatchHandler};

/// Provides services to dispatcher clients.
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
    /// Grab a channel to send messages to the event loop.
    pub fn channel(&self) -> Sender<D::Message> {
        self.event_loop.channel()
    }
    
    /// Execute a closure with a buffer and send the buffer contents to the
    /// destination address or reclaim the buffer and do not send anything.
    pub fn outgoing<F>(&mut self, out: F)
        where F: FnOnce(&mut Buffer) -> Option<SocketAddr> {
        let mut buffer = self.buffer_pool.pop();
        let opt_send_to = out(&mut buffer);
        
        match opt_send_to {
            Some(addr) => self.out_queue.push_back((buffer, addr)),
            None       => self.buffer_pool.push(buffer)
        }
    }
    
    /// Set a timeout with the given delay and token.
    pub fn set_timeout(&mut self, token: D::Timeout, delay: u64) -> TimerResult<Timeout> {
        self.event_loop.timeout_ms(token, delay)
    }
    
    /// Clear a timeout using the provided timeout identifier.
    pub fn clear_timeout(&mut self, timeout: Timeout) -> bool {
        self.event_loop.clear_timeout(timeout)
    }
    
    /// Shutdown the event loop.
    pub fn shutdown(&mut self) {
        self.event_loop.shutdown()
    }
}