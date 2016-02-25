use std::io::{Result};
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};

use mio::{EventLoop, Sender, Token, EventSet, PollOpt};
use mio::udp::{UdpSocket};

use buffer::{BufferPool};
use dispatcher::{Dispatcher, DispatchHandler};

const DEFAULT_MAX_BUFFERS: usize = 4;
const DEFAULT_BUFFER_SIZE: usize = 1500;

pub struct ELoopBuilder {
    buffer_size:  usize,
    bind_address: SocketAddr
}

impl ELoopBuilder {
    pub fn new() -> ELoopBuilder {
        let default_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0));
        
        ELoopBuilder{ buffer_size: DEFAULT_BUFFER_SIZE, bind_address: default_addr }
    }
    
    pub fn bind_address(mut self, address: SocketAddr) -> ELoopBuilder {
        self.bind_address = address;
        
        self
    }
    
    pub fn buffer_length(mut self, length: usize) -> ELoopBuilder {
        self.buffer_size = length;
        
        self
    }
    
    pub fn build<'a, D: Dispatcher>(self) -> Result<ELoop<'a, D>> {
        ELoop::from_builder(self)
    }
}

//----------------------------------------------------------------------------//

pub struct ELoop<'a, D: Dispatcher + 'a> {
    buffer_size: usize,
    socket_addr: SocketAddr,
    event_loop:  EventLoop<DispatchHandler<'a, D>>
}

impl<'a, D: Dispatcher> ELoop<'a, D> {
    fn from_builder(builder: ELoopBuilder) -> Result<ELoop<'a, D>> {
        let event_loop = try!(EventLoop::new());
        
        Ok(ELoop{ buffer_size: builder.buffer_size, socket_addr: builder.bind_address,
            event_loop: event_loop })
    }
    
    pub fn channel(&self) -> Sender<D::Message> {
        self.event_loop.channel()
    }
    
    pub fn run(&'a mut self, dispatcher: &'a mut D) -> Result<()> {
        let udp_socket = try!(UdpSocket::v4());
        udp_socket.bind(&self.socket_addr);
        
        // TODO: Refactor
        self.event_loop.register(&udp_socket, Token(2), EventSet::readable(), PollOpt::oneshot());
        
        let mut dispatch_handler = DispatchHandler::new(udp_socket,
            self.buffer_size, dispatcher);
        
        self.event_loop.run(&mut dispatch_handler)
    }
}

