use hyper::Body;
use hyper::Request;
use tokio::sync::oneshot;

use crate::runtime::Runtime;

pub enum EventMessage {
    HttpRequest(Request<Body>, oneshot::Sender<Body>),
    Schedule(Box<dyn Fn(&mut Runtime) + Send + Sync>),
}
