use hyper::Body;
use hyper::Request;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum EventMessage {
    HttpRequest(Request<Body>, oneshot::Sender<Body>),
}
