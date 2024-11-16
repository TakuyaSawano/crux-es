/// Types which persist and load events for set of Aggregates.
pub trait EventStore {
    /// Associated Type representing Aggregate ID.
    type Id;

    /// Associated Type representing the event for set of Aggregates.
    type SetEvent;

    /// Error type of handling EventStore.
    type Error;

    /// Save the events.
    /// Arrange to call EventBroker::publish() when saving the events.
    fn save(&mut self, id: Self::Id, events: Vec<Self::SetEvent>) -> Result<(), Self::Error>;

    /// Load the events.
    fn load(&self, id: Self::Id) -> Result<Vec<Self::SetEvent>, Self::Error>;
}
