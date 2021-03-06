use futures::sync::mpsc::SendError;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use sub_lib::channel_wrappers::FuturesChannelFactory;
use sub_lib::channel_wrappers::ReceiverWrapper;
use sub_lib::channel_wrappers::SenderWrapper;
use tokio::prelude::Async;

pub struct FuturesChannelFactoryMock<T> {
    pub results: Vec<(Box<SenderWrapper<T>>, Box<ReceiverWrapper<T>>)>,
}

impl<T: 'static + Clone + Debug + Send> FuturesChannelFactory<T> for FuturesChannelFactoryMock<T> {
    fn make(&mut self, peer_addr: SocketAddr) -> (Box<SenderWrapper<T>>, Box<ReceiverWrapper<T>>) {
        if self.results.is_empty() {
            (
                Box::new(SenderWrapperMock::new(peer_addr)),
                Box::new(ReceiverWrapperMock::new()),
            )
        } else {
            self.results.remove(0)
        }
    }
}

pub struct ReceiverWrapperMock<T> {
    pub poll_results: Vec<Result<Async<Option<T>>, ()>>,
}

impl<T: Send> ReceiverWrapper<T> for ReceiverWrapperMock<T> {
    fn poll(&mut self) -> Result<Async<Option<T>>, ()> {
        self.poll_results.remove(0)
    }
}

impl<T> ReceiverWrapperMock<T> {
    pub fn new() -> ReceiverWrapperMock<T> {
        ReceiverWrapperMock {
            poll_results: vec![],
        }
    }
}

#[derive(Debug)]
pub struct SenderWrapperMock<T> {
    pub peer_addr: SocketAddr,
    pub unbounded_send_params: Arc<Mutex<Vec<T>>>,
    pub unbounded_send_results: Vec<Result<(), SendError<T>>>,
}

impl<T: 'static + Clone + Debug + Send> SenderWrapper<T> for SenderWrapperMock<T> {
    fn unbounded_send(&mut self, data: T) -> Result<(), SendError<T>> {
        self.unbounded_send_params.lock().unwrap().push(data);
        if self.unbounded_send_results.is_empty() {
            Ok(())
        } else {
            self.unbounded_send_results.remove(0)
        }
    }

    fn peer_addr(&self) -> SocketAddr {
        self.peer_addr
    }

    fn clone(&self) -> Box<SenderWrapper<T>> {
        Box::new(SenderWrapperMock {
            peer_addr: self.peer_addr,
            unbounded_send_params: self.unbounded_send_params.clone(),
            unbounded_send_results: self.unbounded_send_results.clone(),
        })
    }
}

impl<T> SenderWrapperMock<T> {
    pub fn new(peer_addr: SocketAddr) -> SenderWrapperMock<T> {
        SenderWrapperMock {
            peer_addr,
            unbounded_send_params: Arc::new(Mutex::new(vec![])),
            unbounded_send_results: vec![],
        }
    }
}
