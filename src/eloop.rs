use std::io::{Result};
use std::net::{SocketAddr, SocketAddrV4, Ipv4Addr};

use mio::{EventLoop, Sender, EventLoopConfig};
use mio::udp::{UdpSocket};

use dispatcher::{Dispatcher, DispatchHandler};

const DEFAULT_BUFFER_SIZE:      usize = 1500;
const DEFAULT_CHANNEL_CAPACITY: usize = 4096;
const DEFAULT_TIMER_CAPACITY:   usize = 65536;

/// Builder for specifying attributes of an event loop.
pub struct ELoopBuilder {
    channel_capacity: usize,
    timer_capacity:   usize,
    buffer_size:      usize,
    bind_address:     SocketAddr
}

impl ELoopBuilder {
    /// Create a new event loop builder.
    pub fn new() -> ELoopBuilder {
        let default_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0));
        
        ELoopBuilder{ channel_capacity: DEFAULT_CHANNEL_CAPACITY, timer_capacity: DEFAULT_TIMER_CAPACITY,
            buffer_size: DEFAULT_BUFFER_SIZE, bind_address: default_addr }
    }
    
    /// Manually set the maximum channel message capacity.
    pub fn channel_capacity(mut self, capacity: usize) -> ELoopBuilder {
        self.channel_capacity = capacity;
        
        self
    }
    
    /// Manually set the maximum timer capacity.
    pub fn timer_capacity(mut self, capacity: usize) -> ELoopBuilder {
        self.timer_capacity = capacity;
        
        self
    }
    
    /// Manually set the bind address for the udp socket in the event loop.
    pub fn bind_address(mut self, address: SocketAddr) -> ELoopBuilder {
        self.bind_address = address;
        
        self
    }
    
    /// Manually set the length of buffers provided by the event loop.
    pub fn buffer_length(mut self, length: usize) -> ELoopBuilder {
        self.buffer_size = length;
        
        self
    }
    
    /// Build the event loop with the current builder.
    pub fn build<D: Dispatcher>(self) -> Result<ELoop<D>> {
        ELoop::from_builder(self)
    }
}

//----------------------------------------------------------------------------//

/// Wrapper around the main application event loop.
pub struct ELoop<D: Dispatcher> {
    buffer_size: usize,
    socket_addr: SocketAddr,
    event_loop:  EventLoop<DispatchHandler<D>>
}

impl<D: Dispatcher> ELoop<D> {
    fn from_builder(builder: ELoopBuilder) -> Result<ELoop<D>> {
        let mut event_loop_config = EventLoopConfig::new();
        event_loop_config.notify_capacity(builder.channel_capacity)
            .timer_capacity(builder.timer_capacity);
        
        let event_loop = try!(EventLoop::configured(event_loop_config));
        
        Ok(ELoop{ buffer_size: builder.buffer_size, socket_addr: builder.bind_address,
            event_loop: event_loop })
    }
    
    /// Grab a channel to send messages to the event loop.
    pub fn channel(&self) -> Sender<D::Message> {
        self.event_loop.channel()
    }
    
    /// Run the event loop with the given dispatcher until a shutdown occurs.
    pub fn run(&mut self, dispatcher: D) -> Result<()> {
        let udp_socket = try!(UdpSocket::bound(&self.socket_addr));
        let mut dispatch_handler = DispatchHandler::new(udp_socket, self.buffer_size, dispatcher,
            &mut self.event_loop);
        
        self.event_loop.run(&mut dispatch_handler)
    }
}

