use std::io::{Result};
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};

use mio::{EventLoop, Sender, Token, EventSet, PollOpt, EventLoopConfig};
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
    
    pub fn build<D: Dispatcher>(self) -> Result<ELoop<D>> {
        ELoop::from_builder(self)
    }
}

//----------------------------------------------------------------------------//

pub struct ELoop<D: Dispatcher> {
    buffer_size: usize,
    socket_addr: SocketAddr,
    event_loop:  EventLoop<DispatchHandler<D>>
}

impl<D: Dispatcher> ELoop<D> {
    fn from_builder(builder: ELoopBuilder) -> Result<ELoop<D>> {
        let mut config = EventLoopConfig::new();
        
        let event_loop = try!(EventLoop::configured(config));
        
        Ok(ELoop{ buffer_size: builder.buffer_size, socket_addr: builder.bind_address,
            event_loop: event_loop })
    }
    
    pub fn channel(&self) -> Sender<D::Message> {
        self.event_loop.channel()
    }
    
    pub fn run(&mut self, dispatcher: D) -> Result<()> {
        let udp_socket = try!(UdpSocket::bound(&self.socket_addr));
        let mut dispatch_handler = DispatchHandler::new(udp_socket, self.buffer_size, dispatcher,
            &mut self.event_loop);
        
        self.event_loop.run(&mut dispatch_handler)
    }
}

