pub mod aggregate;
pub mod collection;
pub mod event_broker;
pub mod event_store;
pub mod read_model_updater;

// TODO
// [ ] Interact between Aggregate and another Aggregate.
// [ ] What state is held in Repository?
// [ ] How to save and load events of multiple Aggregates?
