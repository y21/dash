use tokio::sync::mpsc::UnboundedSender;

use crate::runtime::Runtime;

pub enum EventMessage {
    /// Schedules a callback to be executed on the runtime.
    ///
    /// The callback function will run on the same thread as the VM and must be used when calling into JS
    ScheduleCallback(Box<dyn FnOnce(&mut Runtime) + Send + Sync>),
    RemoveTask(u64),
}

#[derive(Debug, Clone)]
pub struct EventSender(UnboundedSender<EventMessage>);

impl EventSender {
    pub fn new(tx: UnboundedSender<EventMessage>) -> Self {
        Self(tx)
    }

    pub fn send(&self, msg: EventMessage) {
        if self.0.send(msg).is_err() {
            tracing::error!("Failed to send message because event receiver was dropped");
        }
    }
}
