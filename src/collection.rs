use crate::{aggregate::Aggregate, event_store::EventStore};

pub trait CollectionEvent {
    type AggregateId;
    fn aggregate_id(&self) -> Self::AggregateId;
}

/// Types which manage the set of Aggregates.
pub trait Collection {
    /// Associated Type representing Aggregate to manage.
    type Aggregate: Aggregate;

    /// Associated Type representing the ID of Aggregates.
    type Id;

    /// Associated Type representing the event for set of Aggregates.
    type Event: CollectionEvent<AggregateId = Self::Id>;

    /// Associated Type representing the response to handle Repository.
    type Response;

    /// Error type of handling Repository.
    type Error;

    /// Associated Type representing the data to create Aggregate.
    type Data;

    /// Create a new Aggregate from Data.
    fn create<ES: EventStore<Self::Event>>(
        &mut self,
        data: Self::Data,
        store: &mut ES,
    ) -> Result<Self::Id, Self::Error>;

    fn find<ES: EventStore<Self::Event>>(
        &mut self,
        id: Self::Id,
        store: &mut ES,
    ) -> Result<Option<Self::Aggregate>, Self::Error>;

    /// Handle a command for an existing Aggregate.
    fn handle_command<ES: EventStore<Self::Event>>(
        &mut self,
        id: Self::Id,
        command: &<Self::Aggregate as Aggregate>::Command,
        store: &mut ES,
    ) -> Result<Self::Response, Self::Error>;
}
