use std::collections::{VecDeque};
use std::net::{SocketAddr};

use mio::{Handler, EventLoop, Token, EventSet};
use mio::udp::{UdpSocket};

use buffer::{BufferPool, Buffer};
use provider::{Provider};
use route::{RouteInfo};

pub trait Dispatcher: Sized {
    type Timeout;
    type Message: Send;
    
    fn incoming<'a>(&mut self, provider: Provider<'a, Self>, message: &[u8], route: RouteInfo) { }
    
    fn notify<'a>(&mut self, provider: Provider<'a, Self>, msg: Self::Message) { }
    
    fn timeout<'a>(&mut self, provider: Provider<'a, Self>, timeout: Self::Timeout) { }
}

//----------------------------------------------------------------------------//

const UDP_SOCKET_TOKEN: Token = Token(2);

pub struct DispatchHandler<'a, D: Dispatcher + 'a> {
    dispatch:    &'a mut D,
    out_queue:   VecDeque<(Buffer, SocketAddr)>,
    udp_socket:  UdpSocket,
    buffer_pool: BufferPool
}

impl<'a, D: Dispatcher> DispatchHandler<'a, D> {
    pub fn new(udp_socket: UdpSocket, buffer_size: usize, dispatch: &'a mut D)
        -> DispatchHandler<'a, D> {
        let buffer_pool = BufferPool::new(buffer_size);
        let out_queue = VecDeque::new();
        
        DispatchHandler{ dispatch: dispatch, out_queue: out_queue, udp_socket: udp_socket,
            buffer_pool: buffer_pool }
    }
    
    
}

impl<'a, D: Dispatcher> Handler for DispatchHandler<'a, D> {
    type Timeout = D::Timeout;
    type Message = D::Message;
    
    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: EventSet) {
        if token != UDP_SOCKET_TOKEN {
            return
        }
        
        if events.is_writable() {
            
        }
        
        if events.is_readable() {
            
        }
    }
    
    fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Self::Message) {
    
    }
    
    fn timeout(&mut self, event_loop: &mut EventLoop<Self>, timeout: Self::Timeout) {
    
    }
    
    fn tick(&mut self, event_loop: &mut EventLoop<Self>) {
        
    }
}