#[cfg(test)]
mod tests;

/// Types which represent a backlog.
pub trait Backlog {
    /// Associated Type representing the ID of the backlog.
    type Id;
    /// Associated Type representing the status of the backlog.
    type Status;
    /// Associated Type representing the event for creating the backlog.
    type CreateEvent;
    /// Associated Type representing the event for resolving the backlog.
    type ResolveEvent;

    /// Get the ID of the backlog.
    fn id(&self) -> Self::Id;
    /// Create a new backlog.
    fn create(event: Self::CreateEvent) -> Self;
    /// Resolve the backlog.
    fn resolve(&mut self, event: Self::ResolveEvent) -> &Self::Status;
    /// Get the status of the backlog.
    fn status(&self) -> &Self::Status;
}
