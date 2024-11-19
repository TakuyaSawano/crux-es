use crate::collection::CollectionEvent;

/// Types which publish events to subscribers.
pub trait EventBroker<E: CollectionEvent> {
    /// Error type of handling EventBroker.
    type Error;

    /// Publish events to the subscribers.
    // TODO: Use `&[E]` instead of `Vec<E>`.
    fn publish(&mut self, event: Vec<E>) -> Result<(), Self::Error>;
}
