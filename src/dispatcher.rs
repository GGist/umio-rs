use std::collections::{VecDeque};
use std::net::{SocketAddr};

use mio::{Handler, EventLoop, Token, EventSet, PollOpt};
use mio::udp::{UdpSocket};

use buffer::{BufferPool, Buffer};
use provider::{self, Provider};

/// Handles events occurring within the event loop.
pub trait Dispatcher: Sized {
    type Timeout;
    type Message: Send;
    
    /// Process an incoming message from the given address.
    fn incoming<'a>(&mut self, _: Provider<'a, Self>, _: &[u8], _: SocketAddr) { }
    
    /// Process a message sent via the event loop channel.
    fn notify<'a>(&mut self, _: Provider<'a, Self>, _: Self::Message) { }
    
    /// Process a timeout that has been triggered.
    fn timeout<'a>(&mut self, _: Provider<'a, Self>, _: Self::Timeout) { }
}

//----------------------------------------------------------------------------//

const UDP_SOCKET_TOKEN: Token = Token(2);

pub struct DispatchHandler<D: Dispatcher> {
    dispatch:    D,
    out_queue:   VecDeque<(Buffer, SocketAddr)>,
    udp_socket:  UdpSocket,
    buffer_pool: BufferPool,
    current_set: EventSet
}

impl<D: Dispatcher> DispatchHandler<D> {
    pub fn new(udp_socket: UdpSocket, buffer_size: usize, dispatch: D, event_loop: &mut EventLoop<DispatchHandler<D>>)
        -> DispatchHandler<D> {
        let buffer_pool = BufferPool::new(buffer_size);
        let out_queue = VecDeque::new();
        
        event_loop.register(&udp_socket, UDP_SOCKET_TOKEN, EventSet::readable(), PollOpt::level()).unwrap();
        
        DispatchHandler{ dispatch: dispatch, out_queue: out_queue, udp_socket: udp_socket,
            buffer_pool: buffer_pool, current_set: EventSet::readable() }
    } 
    
    pub fn handle_write(&mut self) {
        match self.out_queue.pop_front() {
            Some((buffer, addr)) => {
                self.udp_socket.send_to(buffer.as_ref(), &addr).unwrap();
                
                self.buffer_pool.push(buffer);
            },
            None => ()
        };
    }
    
    pub fn handle_read(&mut self) -> Option<(Buffer, SocketAddr)> {
        let mut buffer = self.buffer_pool.pop();
        
        if let Ok(Some((bytes, addr))) = self.udp_socket.recv_from(buffer.as_mut()) {
            buffer.set_written(bytes);
            
            Some((buffer, addr))
        } else {
            None
        }
    }
}

impl<D: Dispatcher> Handler for DispatchHandler<D> {
    type Timeout = D::Timeout;
    type Message = D::Message;
    
    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: EventSet) {
        if token != UDP_SOCKET_TOKEN {
            return
        }
        
        if events.is_writable() {
            self.handle_write();
        }
        
        if events.is_readable() {
            let (buffer, addr) = if let Some((buffer, addr)) = self.handle_read() {
                (buffer, addr)
            } else { return };
            
            {
                let provider = provider::new(&mut self.buffer_pool, &mut self.out_queue, event_loop);
                
                self.dispatch.incoming(provider, buffer.as_ref(), addr);
            }
            
            self.buffer_pool.push(buffer);
        }
    }
    
    fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Self::Message) {
        let provider = provider::new(&mut self.buffer_pool, &mut self.out_queue, event_loop);
    
        self.dispatch.notify(provider, msg);
    }
    
    fn timeout(&mut self, event_loop: &mut EventLoop<Self>, timeout: Self::Timeout) {
        let provider = provider::new(&mut self.buffer_pool, &mut self.out_queue, event_loop);
        
        self.dispatch.timeout(provider, timeout);
    }
    
    fn tick(&mut self, event_loop: &mut EventLoop<Self>) {
        self.current_set = if !self.out_queue.is_empty() {
            EventSet::readable() | EventSet::writable()
        } else {
            EventSet::readable()
        };
        
        event_loop.reregister(&self.udp_socket, UDP_SOCKET_TOKEN, self.current_set, PollOpt::level()).unwrap();
    }
}