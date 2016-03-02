use mio::{self, NotifyError};

use dispatcher::{DispatchMessage};

/// Creates a new Sender for the client to utilize.
pub fn new_sender<M: Send>(send: mio::Sender<DispatchMessage<M>>) -> Sender<M> {
    Sender{ send: send }
}

/// Sender for notifying the Dispatcher of events.
#[derive(Clone, Debug)]
pub struct Sender<M: Send> {
    send: mio::Sender<DispatchMessage<M>>
}

impl<M: Send> Sender<M> {
    pub fn send(&self, msg: M) -> Result<(), NotifyError<M>> {
        self.send.send(DispatchMessage::DispatchNotify(msg)).map_err(|error| {
            match error {
                NotifyError::Io(err)         => NotifyError::Io(err),
                NotifyError::Full(msg)       => NotifyError::Full(convert_message(msg)),
                NotifyError::Closed(opt_msg) => NotifyError::Closed(opt_msg.map(convert_message))
            }
        })
    }
}

fn convert_message<M>(msg: DispatchMessage<M>) -> M {
    match msg {
        DispatchMessage::DispatchNotify(m) => m,
        _ => panic!("Bug In umio-rs: Channel error returned a different dispatch message than what was sent")
    }
}