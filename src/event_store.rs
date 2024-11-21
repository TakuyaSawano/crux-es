#[cfg(test)]
mod tests;

use std::error::Error;

/// Types which have transaction management capabilities.
pub trait TransactionManager {
    /// Associated Type representing the error type.
    type Error: Error;

    /// Begin a transaction.
    fn begin(&mut self) -> Result<(), Self::Error>;

    /// Commit the transaction.
    fn commit(&mut self) -> Result<(), Self::Error>;

    /// Rollback the transaction.
    fn rollback(&mut self) -> Result<(), Self::Error>;
}

/// Types which represent an event store.
pub trait EventStore {
    /// Associated Type representing the query to persist event.
    type Persistable;
    /// Associated Type representing the error type.
    type Error: Error;

    /// Save the events.
    fn save(&mut self, events: &[Self::Persistable]) -> Result<(), Self::Error>;
}

/// Types which represent a handler for a query to the event store.
pub trait QueryHandler<Query> {
    /// Associated Type representing the response type.
    type Response;
    /// Associated Type representing the error type.
    type Error: Error;

    /// Handle the query.
    fn handle(&self, query: Query) -> Result<Self::Response, Self::Error>;
}
