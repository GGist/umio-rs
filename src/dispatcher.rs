use std::collections::{VecDeque};
use std::net::{SocketAddr};

use mio::{Handler, EventLoop, Token, EventSet, PollOpt};
use mio::udp::{UdpSocket};

use buffer::{BufferPool, Buffer};
use provider::{Provider};
use route::{RouteInfo};

pub trait Dispatcher: Sized {
    type Timeout;
    type Message: Send;
    
    fn incoming<'a, 'b>(&mut self, provider: Provider<'a, 'b, Self>, message: &[u8]) { }
    
    fn notify<'a, 'b>(&mut self, provider: Provider<'a, 'b, Self>, msg: Self::Message) { }
    
    fn timeout<'a, 'b>(&mut self, provider: Provider<'a, 'b, Self>, timeout: Self::Timeout) { }
}

//----------------------------------------------------------------------------//

const UDP_SOCKET_TOKEN: Token = Token(2);

pub struct DispatchHandler<'a, D: Dispatcher + 'a> {
    dispatch:    &'a mut D,
    out_queue:   VecDeque<(Buffer, SocketAddr)>,
    udp_socket:  UdpSocket,
    buffer_pool: BufferPool,
    current_set: EventSet
}

impl<'a, D: Dispatcher> DispatchHandler<'a, D> {
    pub fn new(udp_socket: UdpSocket, buffer_size: usize, dispatch: &'a mut D)
        -> DispatchHandler<'a, D> {
        let buffer_pool = BufferPool::new(buffer_size);
        let out_queue = VecDeque::new();
        
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
        
        // Update current set if more messages are pending
        self.current_set = if self.out_queue.is_empty() {
            EventSet::readable()
        } else {
            EventSet::readable() | EventSet::writable()
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
        
        // No need to update current set (always readable)
    }
}

impl<'a, D: Dispatcher + 'a> Handler for DispatchHandler<'a, D> {
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
                let provider = Provider::new(&mut self.buffer_pool, &mut self.out_queue, event_loop);
                
                self.dispatch.incoming(provider, buffer.as_ref());
            }
            
            self.buffer_pool.push(buffer);
        }
        
        event_loop.reregister(&self.udp_socket, UDP_SOCKET_TOKEN, self.current_set, PollOpt::oneshot()).unwrap();
    }
    
    fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Self::Message) {
        //let provider = Provider::new(&mut self.buffer_pool, &mut self.out_queue, event_loop);
    
        //self.dispatch.notify(provider, msg);
    }
    
    fn timeout(&mut self, event_loop: &mut EventLoop<Self>, timeout: Self::Timeout) {
        //let provider = Provider::new(&mut self.buffer_pool, &mut self.out_queue, event_loop);
        
        //self.dispatch.timeout(provider, timeout);
    }
    
    fn tick(&mut self, event_loop: &mut EventLoop<Self>) {
        // Triggers when the first message is put in the queue (from a notify probably)
        if !self.out_queue.is_empty() && !self.current_set.is_writable() {
            let new_set = self.current_set | EventSet::writable();
            
            event_loop.reregister(&self.udp_socket, Token(2), new_set, PollOpt::oneshot()).unwrap();
        }
    }
}