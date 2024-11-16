use crate::aggregate::Aggregate;

/// Types which manage the set of Aggregates.
pub trait Repository {
    /// Associated Type representing Aggregate to manage.
    type Aggregate: Aggregate;

    /// Associated Type representing the response of handling Command.
    type CommandResponse;

    /// Associated Type representing the identifier of Aggregate.
    type Id;

    /// Associated Type representing the event to set.
    /// SetEvent include Event of Aggregate by enum.
    type SetEvent;

    /// Error type of handling Repository.
    type Error;

    /// Associated Type representing the data to create Aggregate.
    type Data;

    /// Create a new Aggregate from Data.
    fn create(&mut self, data: Self::Data) -> Result<Self::Id, Self::Error>;

    fn find(&mut self, id: Self::Id) -> Result<Option<Self::Aggregate>, Self::Error>;

    /// Handle a command for an existing Aggregate.
    fn handle_command(
        &mut self,
        id: Self::Id,
        command: &<Self::Aggregate as Aggregate>::Command,
    ) -> Result<Self::CommandResponse, Self::Error>;
}
