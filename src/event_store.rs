use std::fmt::Debug;

use crate::collection::CollectionEvent;

/// Types which persist and load events for set of Aggregates.
pub trait EventStore<E: CollectionEvent> {
    /// Error type of handling EventStore.
    type Error: Debug;

    /// Save the events.
    /// Arrange to call EventBroker::publish() when saving the events.
    fn save(&mut self, events: Vec<E>) -> Result<(), Self::Error>;

    /// Load the events.
    fn load(&self, id: E::AggregateId) -> Result<Vec<E>, Self::Error>;
}

pub trait TransactionManager {
    type Error: Debug;

    fn begin(&mut self) -> Result<(), Self::Error>;
    fn commit(&mut self) -> Result<(), Self::Error>;
    fn rollback(&mut self) -> Result<(), Self::Error>;
}
