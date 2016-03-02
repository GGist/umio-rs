use std::net::{SocketAddr};
use std::sync::{Arc};

use mio::{EventLoop, TimerResult, Timeout};
use threadpool::{ThreadPool};

use buffer::{BufferPool};
use client::{self};
use dispatcher::{Dispatcher, DispatchHandler, DispatchMessage};

/// Provides services to dispatcher clients.
pub struct Provider<'a, D: Dispatcher + 'a> {
    thread_pool: &'a ThreadPool,
    buffer_pool: &'a Arc<BufferPool>,
    event_loop:  &'a mut EventLoop<DispatchHandler<D>>
}

pub fn new_provider<'a, D: Dispatcher>(thread_pool: &'a ThreadPool, buffer_pool: &'a Arc<BufferPool>,
    event_loop: &'a mut EventLoop<DispatchHandler<D>>) -> Provider<'a, D> {
    Provider{ thread_pool: thread_pool, buffer_pool: buffer_pool, event_loop: event_loop }   
}

impl<'a, D: Dispatcher> Provider<'a, D> {
    /// Grab a channel to send messages to the event loop.
    pub fn channel(&self) -> client::Sender<D::Message> {
        client::new_sender(self.event_loop.channel())
    }
    
    /// Execute the closure in another thread, possibly blocking until a buffer is available.
    ///
    /// Sends the buffer with the number of bytes written to the destination address or nothing.
    pub fn outgoing<F>(&mut self, out: F)
        where F: FnOnce(&mut [u8]) -> Option<(usize, SocketAddr)> + Send + 'static {
        let out_channel = self.event_loop.channel();
        let share_pool = self.buffer_pool.clone();
        
        self.thread_pool.execute(move || {
            let mut buffer = share_pool.pop();
            
            match out(buffer.as_mut()) {
                Some((bytes, addr)) => {
                    buffer.set_written(bytes);
                    
                    out_channel.send(DispatchMessage::DispatchOutgoing(buffer, addr)).unwrap();
                },
                None => {
                    share_pool.push(buffer);
                }
            }
        });
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