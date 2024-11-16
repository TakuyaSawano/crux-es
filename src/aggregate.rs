/// Types which manage the state of aggregate.
pub trait Aggregate: Sized {
    /// Associated type representing the command type to change state of Aggregate.
    type Command;

    /// Associated type representing the event type for the state change of Aggregate.
    type Event;

    // Associated type representing the error type to handle Command.
    type Error;

    /// Handle Command and return the result of events.
    fn handle_command(&self, command: &Self::Command) -> Result<Vec<Self::Event>, Self::Error>;

    /// Apply an Event to update the state of the Aggregate.
    fn apply_event(&mut self, event: &Self::Event);
}
