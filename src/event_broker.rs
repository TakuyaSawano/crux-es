/// Types which publish events to subscribers.
pub trait EventBroker {
    /// Associated Type representing the event for set of Aggregates.
    type SetEvent;

    /// Error type of handling EventBroker.
    type Error;

    /// Publish events to the subscribers.
    fn publish(&mut self, event: Vec<Self::SetEvent>) -> Result<(), Self::Error>;
}
