use hyper::Body;
use hyper::Request;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

pub enum EventMessage {
    HttpRequest(Request<Body>, oneshot::Sender<Body>),
}

#[derive(Debug, Clone)]
pub struct EventSender(UnboundedSender<EventMessage>);

impl EventSender {
    pub fn new(tx: UnboundedSender<EventMessage>) -> Self {
        Self(tx)
    }

    pub fn send(&self, msg: EventMessage) {
        if let Err(..) = self.0.send(msg) {
            tracing::error!("Failed to send message because event receiver was dropped");
        }
    }
}
