/// Types which update the read model.
pub trait ReadModelUpdater {
    /// Associated Type representing the event for set of Aggregates.
    type SetEvent;

    /// Error type of handling ReadModelUpdater.
    type Error;

    /// Update the read model.
    fn update(&mut self, event: Vec<Self::SetEvent>) -> Result<(), Self::Error>;
}
