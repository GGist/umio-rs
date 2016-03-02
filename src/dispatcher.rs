use std::collections::{VecDeque};
use std::net::{SocketAddr};
use std::sync::{Arc};

use mio::{Handler, EventLoop, Token, EventSet, PollOpt, Sender};
use mio::udp::{UdpSocket};
use threadpool::{ThreadPool};

use buffer::{BufferPool, Buffer};
use provider::{self, Provider};

/// Handles events occurring within the event loop.
pub trait Dispatcher: Sized {
    type Timeout;
    type Message: Send + 'static;
    
    /// Process an incoming message from the given address.
    fn incoming<'a>(&mut self, _: Provider<'a, Self>, _: &[u8], _: SocketAddr) { }
    
    /// Process a message sent via the event loop channel.
    fn notify<'a>(&mut self, _: Provider<'a, Self>, _: Self::Message) { }
    
    /// Process a timeout that has been triggered.
    fn timeout<'a>(&mut self, _: Provider<'a, Self>, _: Self::Timeout) { }
}

//----------------------------------------------------------------------------//

// Allows our dispatcher to receive both user provided messages and messages
// from our thread pool when buffers have been read in to and are ready to
// be handled as an incoming message.
pub enum DispatchMessage<M> {
    /// Reading thread sends us an incoming message.
    DispatchIncoming(Buffer, SocketAddr),
    /// Writing thread (from provider) sends us an outgoing message.
    DispatchOutgoing(Buffer, SocketAddr),
    /// User defined message to propogate up.
    DispatchNotify(M)
}

const UDP_SOCKET_TOKEN: Token = Token(2);

pub struct DispatchHandler<D: Dispatcher> {
    dispatch:    D,
    out_queue:   VecDeque<(Buffer, SocketAddr)>,
    thread_pool: ThreadPool,
    udp_socket:  Arc<UdpSocket>,
    buffer_pool: Arc<BufferPool>,
    current_set: EventSet
}

impl<D: Dispatcher> DispatchHandler<D> {
    pub fn new(udp_socket: UdpSocket, buffer_size: usize, max_buffers: usize, dispatch: D,
        event_loop: &mut EventLoop<DispatchHandler<D>>) -> DispatchHandler<D> {
        let shared_buffer_pool = Arc::new(BufferPool::new(buffer_size, max_buffers));
        let shared_udp_socket = Arc::new(udp_socket);
        let thread_pool = ThreadPool::new(1);
        
        event_loop.register(&*shared_udp_socket, UDP_SOCKET_TOKEN, EventSet::readable(), PollOpt::edge()).unwrap();
        
        DispatchHandler{ dispatch: dispatch, out_queue: VecDeque::new(), thread_pool: thread_pool,
            udp_socket: shared_udp_socket, buffer_pool: shared_buffer_pool, current_set: EventSet::readable() }
    } 
    
    pub fn handle_write(&mut self) {
        // Write in the current thread since we do not have to block.
        match self.out_queue.pop_front() {
            Some((buffer, addr)) => {
                self.udp_socket.send_to(buffer.as_ref(), &addr).unwrap();
                
                self.buffer_pool.push(buffer);
            },
            None => ()
        };
    }
    
    pub fn handle_read(&mut self, send: Sender<DispatchMessage<D::Message>>) {
        // Read in a separate thread in case we have to block for a free buffer.
        let share_socket = self.udp_socket.clone();
        let share_pool = self.buffer_pool.clone();
        
        self.thread_pool.execute(move || {
            let mut buffer = share_pool.pop();
            
            match share_socket.recv_from(buffer.as_mut()) {
                Ok(Some((bytes, addr))) => {
                    buffer.set_written(bytes);
                    
                    send.send(DispatchMessage::DispatchIncoming(buffer, addr)).unwrap();
                },
                _ => ()
            };
        });
    }
}

impl<D: Dispatcher> Handler for DispatchHandler<D> {
    type Timeout = D::Timeout;
    type Message = DispatchMessage<D::Message>;
    
    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: EventSet) {
        if token != UDP_SOCKET_TOKEN {
            return
        }
        
        if events.is_writable() {
            self.handle_write();
        }
        
        if events.is_readable() {
            self.handle_read(event_loop.channel());
        }
    }
    
    fn notify(&mut self, event_loop: &mut EventLoop<Self>, msg: Self::Message) {
        match msg {
            DispatchMessage::DispatchIncoming(buffer, addr) => {
                {
                    let provider = provider::new_provider(&self.thread_pool, &self.buffer_pool, event_loop);
                    
                    self.dispatch.incoming(provider, buffer.as_ref(), addr);
                }
                
                self.buffer_pool.push(buffer);
            },
            DispatchMessage::DispatchOutgoing(buffer, addr) => {
                self.out_queue.push_back((buffer, addr));
            },
            DispatchMessage::DispatchNotify(message) => {
                let provider = provider::new_provider(&self.thread_pool, &self.buffer_pool, event_loop);
            
                self.dispatch.notify(provider, message);
            }
        }
    }
    
    fn timeout(&mut self, event_loop: &mut EventLoop<Self>, timeout: Self::Timeout) {
        let provider = provider::new_provider(&self.thread_pool, &self.buffer_pool, event_loop);
        
        self.dispatch.timeout(provider, timeout);
    }
    
    fn tick(&mut self, event_loop: &mut EventLoop<Self>) {
        self.current_set = if !self.out_queue.is_empty() {
            EventSet::readable() | EventSet::writable()
        } else {
            EventSet::readable()
        };
        
        event_loop.reregister(&*self.udp_socket, UDP_SOCKET_TOKEN, self.current_set, PollOpt::edge()).unwrap();
    }
}