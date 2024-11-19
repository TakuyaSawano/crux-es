use crux_es::aggregate::Aggregate;
use crux_es::collection::{Collection, CollectionEvent};
use crux_es::event_broker::EventBroker;
use crux_es::event_store::{EventStore, TransactionManager};

use std::collections::HashMap;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UserOrganizationId(pub String);

pub struct Organization {
    pub name: String,
    pub users: HashMap<UserId, UserOrganizationId>,
    pub max_users: usize,
}

pub enum OrganizationCommand {
    Rename(String),
    UserReserve(String),
    AddUser(UserId, UserOrganizationId),
    RemoveUser(UserId),
}

#[derive(Debug, Clone)]
pub enum OrganizationEvent {
    Renamed(String),
    UserReserved(UserOrganizationId),
    UserAdded(UserId, UserOrganizationId),
    UserRemoved(UserId),
}

#[derive(Debug, Clone)]
pub enum OrganizationError {
    MaxUsersReached,
    UserNotFound,
}

impl Organization {
    pub fn new(name: String, max_users: usize) -> Self {
        Organization {
            name,
            users: HashMap::new(),
            max_users,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct OrganizationId(pub String);

pub struct User {
    pub name: String,
    pub email: String,
    pub organization_id: OrganizationId,
    pub uo_id: UserOrganizationId,
}

pub enum UserCommand {
    Rename(String),
    ChangeEmail(String),
}

#[derive(Debug, Clone)]
pub enum UserEvent {
    Renamed(String),
    EmailChanged(String),
}

#[derive(Debug, Clone)]
pub enum UserError {}

impl User {
    pub fn new(
        name: String,
        email: String,
        organization_id: OrganizationId,
        uo_id: UserOrganizationId,
    ) -> Self {
        User {
            name,
            email,
            organization_id,
            uo_id,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct UserId(pub usize);

impl Aggregate for Organization {
    type Command = OrganizationCommand;
    type Event = OrganizationEvent;
    type Error = OrganizationError;
    fn apply_event(&mut self, event: &Self::Event) {
        match event {
            OrganizationEvent::Renamed(name) => {
                self.name = name.clone();
            }
            OrganizationEvent::UserReserved(_uo_id) => {}
            OrganizationEvent::UserAdded(user_id, uo_id) => {
                self.users.insert(user_id.clone(), uo_id.clone());
            }
            OrganizationEvent::UserRemoved(user_id) => {
                self.users.remove(user_id);
            }
        }
    }
    fn handle_command(&self, command: &Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            OrganizationCommand::Rename(name) => Ok(vec![OrganizationEvent::Renamed(name.clone())]),
            OrganizationCommand::UserReserve(name) => {
                let uo_id = UserOrganizationId(format!("{}-{}", self.name, name));
                Ok(vec![OrganizationEvent::UserReserved(uo_id)])
            }
            OrganizationCommand::AddUser(user_id, uo_id) => {
                if self.users.len() >= self.max_users {
                    return Err(OrganizationError::MaxUsersReached);
                }
                Ok(vec![OrganizationEvent::UserAdded(
                    user_id.clone(),
                    uo_id.clone(),
                )])
            }
            OrganizationCommand::RemoveUser(user_id) => {
                if !self.users.contains_key(user_id) {
                    return Err(OrganizationError::UserNotFound);
                }
                Ok(vec![OrganizationEvent::UserRemoved(user_id.clone())])
            }
        }
    }
}

impl Aggregate for User {
    type Command = UserCommand;
    type Event = UserEvent;
    type Error = UserError;
    fn apply_event(&mut self, event: &Self::Event) {
        match event {
            UserEvent::Renamed(name) => {
                self.name = name.clone();
            }
            UserEvent::EmailChanged(email) => {
                self.email = email.clone();
            }
        }
    }
    fn handle_command(&self, command: &Self::Command) -> Result<Vec<Self::Event>, Self::Error> {
        match command {
            UserCommand::Rename(name) => Ok(vec![UserEvent::Renamed(name.clone())]),
            UserCommand::ChangeEmail(email) => Ok(vec![UserEvent::EmailChanged(email.clone())]),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OrganizationCollectionEvent {
    Created(OrganizationId, (String, usize)),
    Aggregate(OrganizationId, OrganizationEvent),
    Deleted(OrganizationId),
}

#[derive(Debug, Clone)]
pub enum UserCollectionEvent {
    Created(UserId, (String, String, OrganizationId, UserOrganizationId)),
    Aggregate(UserId, UserEvent),
    Deleted(UserId),
}

impl CollectionEvent for OrganizationCollectionEvent {
    type AggregateId = OrganizationId;
    fn aggregate_id(&self) -> Self::AggregateId {
        match self {
            OrganizationCollectionEvent::Created(id, _) => id.clone(),
            OrganizationCollectionEvent::Aggregate(id, _) => id.clone(),
            OrganizationCollectionEvent::Deleted(id) => id.clone(),
        }
    }
}

impl CollectionEvent for UserCollectionEvent {
    type AggregateId = UserId;
    fn aggregate_id(&self) -> Self::AggregateId {
        match self {
            UserCollectionEvent::Created(id, _) => id.clone(),
            UserCollectionEvent::Aggregate(id, _) => id.clone(),
            UserCollectionEvent::Deleted(id) => id.clone(),
        }
    }
}

pub struct OrganizationCollection {}
pub struct UserCollection {
    pub length: usize,
}

#[derive(Debug, Clone)]
pub enum OrganizationCollectionError {
    NotFound(OrganizationId),
    Origanization(OrganizationError),
    Store(String),
    InvalidEvent(OrganizationCollectionEvent),
}

impl From<OrganizationError> for OrganizationCollectionError {
    fn from(e: OrganizationError) -> Self {
        OrganizationCollectionError::Origanization(e)
    }
}

#[derive(Debug, Clone)]
pub enum UserCollectionError {
    NotFound(UserId),
    User(UserError),
    Store(String),
    InvalidEvent(UserCollectionEvent),
}

impl From<UserError> for UserCollectionError {
    fn from(e: UserError) -> Self {
        UserCollectionError::User(e)
    }
}

impl Collection for OrganizationCollection {
    type Aggregate = Organization;
    type Id = OrganizationId;
    type Event = OrganizationCollectionEvent;
    type Response = String;
    type Data = (String, usize);
    type Error = OrganizationCollectionError;
    fn create<ES: EventStore<Self::Event>>(
        &mut self,
        data: Self::Data,
        store: &mut ES,
    ) -> Result<Self::Id, Self::Error> {
        let id = OrganizationId(data.0.clone());
        store
            .save(vec![OrganizationCollectionEvent::Created(id.clone(), data)])
            .map_err(|e| OrganizationCollectionError::Store(format!("{:?}", e)))?;
        Ok(id)
    }
    fn find<ES: EventStore<Self::Event>>(
        &mut self,
        id: Self::Id,
        store: &mut ES,
    ) -> Result<Option<Self::Aggregate>, Self::Error> {
        let events = store
            .load(id)
            .map_err(|e| OrganizationCollectionError::Store(format!("{:?}", e)))?;
        let mut iter = events.iter();
        let e = iter.next();
        let mut aggregate = match e {
            Some(OrganizationCollectionEvent::Created(_, (name, max_users))) => {
                Some(Organization::new(name.clone(), *max_users))
            }
            None => None,
            _ => {
                return Err(OrganizationCollectionError::InvalidEvent(
                    e.unwrap().clone(),
                ))
            }
        };
        for event in iter {
            match event {
                OrganizationCollectionEvent::Aggregate(_, event) => {
                    if let Some(aggregate) = aggregate.as_mut() {
                        aggregate.apply_event(event);
                    }
                }
                OrganizationCollectionEvent::Deleted(_) => {
                    return Ok(None);
                }
                _ => {}
            }
        }
        Ok(aggregate)
    }
    fn handle_command<ES: EventStore<Self::Event>>(
        &mut self,
        id: Self::Id,
        command: &<Self::Aggregate as Aggregate>::Command,
        store: &mut ES,
    ) -> Result<Self::Response, Self::Error> {
        let aggregate = self
            .find(id.clone(), store)?
            .ok_or_else(|| OrganizationCollectionError::NotFound(id.clone()))?;
        let events = aggregate
            .handle_command(command)?
            .iter()
            .map(|e| OrganizationCollectionEvent::Aggregate(id.clone(), e.clone()))
            .collect();
        store
            .save(events)
            .map_err(|e| OrganizationCollectionError::Store(format!("{:?}", e)))?;
        Ok("Success".to_string())
    }
}

impl Collection for UserCollection {
    type Aggregate = User;
    type Data = (String, String, OrganizationId, UserOrganizationId);
    type Error = UserCollectionError;
    type Event = UserCollectionEvent;
    type Id = UserId;
    type Response = String;
    fn create<ES: EventStore<Self::Event>>(
        &mut self,
        data: Self::Data,
        store: &mut ES,
    ) -> Result<Self::Id, Self::Error> {
        self.length += 1;
        let id = UserId(self.length);
        store
            .save(vec![UserCollectionEvent::Created(id.clone(), data)])
            .map_err(|e| UserCollectionError::Store(format!("{:?}", e)))?;
        Ok(id)
    }
    fn find<ES: EventStore<Self::Event>>(
        &mut self,
        id: Self::Id,
        store: &mut ES,
    ) -> Result<Option<Self::Aggregate>, Self::Error> {
        let events = store
            .load(id)
            .map_err(|e| UserCollectionError::Store(format!("{:?}", e)))?;
        let mut iter = events.iter();
        let e = iter.next();
        let mut aggregate = match e {
            Some(UserCollectionEvent::Created(_, (name, email, organization_id, uo_id))) => {
                Some(User::new(
                    name.clone(),
                    email.clone(),
                    organization_id.clone(),
                    uo_id.clone(),
                ))
            }
            None => None,
            _ => return Err(UserCollectionError::InvalidEvent(e.unwrap().clone())),
        };
        for event in iter {
            match event {
                UserCollectionEvent::Aggregate(_, event) => {
                    if let Some(aggregate) = aggregate.as_mut() {
                        aggregate.apply_event(event);
                    }
                }
                UserCollectionEvent::Deleted(_) => {
                    return Ok(None);
                }
                _ => {}
            }
        }
        Ok(aggregate)
    }
    fn handle_command<ES: EventStore<Self::Event>>(
        &mut self,
        id: Self::Id,
        command: &<Self::Aggregate as Aggregate>::Command,
        store: &mut ES,
    ) -> Result<Self::Response, Self::Error> {
        let aggregate = self
            .find(id.clone(), store)?
            .ok_or_else(|| UserCollectionError::NotFound(id.clone()))?;
        let events = aggregate
            .handle_command(command)?
            .iter()
            .map(|e| UserCollectionEvent::Aggregate(id.clone(), e.clone()))
            .collect();
        store
            .save(events)
            .map_err(|e| UserCollectionError::Store(format!("{:?}", e)))?;
        Ok("Success".to_string())
    }
}

pub struct OnMemoryEventBroker {}
impl EventBroker<OrganizationCollectionEvent> for OnMemoryEventBroker {
    type Error = ();
    fn publish(&mut self, events: Vec<OrganizationCollectionEvent>) -> Result<(), Self::Error> {
        for event in events {
            println!("Organization Event: {:?} {:?}", event.aggregate_id(), event);
        }
        Ok(())
    }
}
impl EventBroker<UserCollectionEvent> for OnMemoryEventBroker {
    type Error = ();
    fn publish(&mut self, events: Vec<UserCollectionEvent>) -> Result<(), Self::Error> {
        for event in events {
            println!("User Event: {:?} {:?}", event.aggregate_id(), event);
        }
        Ok(())
    }
}

pub struct OnMemoryEventStore {
    org_events: HashMap<OrganizationId, Vec<OrganizationCollectionEvent>>,
    org_uncommitted_events: HashMap<OrganizationId, Vec<OrganizationCollectionEvent>>,
    user_events: HashMap<UserId, Vec<UserCollectionEvent>>,
    user_uncommitted_events: HashMap<UserId, Vec<UserCollectionEvent>>,
    broker: OnMemoryEventBroker,
    in_transaction: bool,
}

#[derive(Debug)]
pub struct EventStoreError(pub String);

impl TransactionManager for OnMemoryEventStore {
    type Error = EventStoreError;
    fn begin(&mut self) -> Result<(), Self::Error> {
        self.in_transaction = true;
        Ok(())
    }
    fn commit(&mut self) -> Result<(), Self::Error> {
        let org_event_to_publish = self.org_uncommitted_events.clone();
        let user_event_to_publish = self.user_uncommitted_events.clone();

        for (id, events) in self.org_uncommitted_events.drain() {
            let org_events = self.org_events.entry(id).or_default();
            org_events.extend(events);
        }
        for (id, events) in self.user_uncommitted_events.drain() {
            let user_events = self.user_events.entry(id).or_default();
            user_events.extend(events);
        }

        self.in_transaction = false;

        // Publish events
        // TODO: Order of events is not guaranteed.
        for (_, events) in org_event_to_publish {
            self.broker.publish(events).unwrap();
        }
        for (_, events) in user_event_to_publish {
            self.broker.publish(events).unwrap();
        }

        Ok(())
    }
    fn rollback(&mut self) -> Result<(), Self::Error> {
        self.in_transaction = false;
        Ok(())
    }
}

impl EventStore<OrganizationCollectionEvent> for OnMemoryEventStore {
    type Error = EventStoreError;
    fn save(&mut self, events: Vec<OrganizationCollectionEvent>) -> Result<(), Self::Error> {
        if self.in_transaction {
            for event in events {
                self.org_uncommitted_events
                    .entry(event.aggregate_id())
                    .or_default()
                    .push(event);
            }
        } else {
            return Err(EventStoreError("Not in transaction".to_string()));
        }
        Ok(())
    }
    fn load(&self, id: OrganizationId) -> Result<Vec<OrganizationCollectionEvent>, Self::Error> {
        Ok(self.org_events.get(&id).cloned().unwrap_or_default())
    }
}

impl EventStore<UserCollectionEvent> for OnMemoryEventStore {
    type Error = EventStoreError;
    fn save(&mut self, events: Vec<UserCollectionEvent>) -> Result<(), Self::Error> {
        if self.in_transaction {
            for event in events {
                self.user_uncommitted_events
                    .entry(event.aggregate_id())
                    .or_default()
                    .push(event);
            }
        } else {
            return Err(EventStoreError("Not in transaction".to_string()));
        }
        Ok(())
    }
    fn load(
        &self,
        id: <UserCollectionEvent as CollectionEvent>::AggregateId,
    ) -> Result<Vec<UserCollectionEvent>, Self::Error> {
        Ok(self.user_events.get(&id).cloned().unwrap_or_default())
    }
}

pub fn user_add_usecase<ES>(
    name: String,
    email: String,
    org_id: OrganizationId,
    org_collection: &mut OrganizationCollection,
    user_collection: &mut UserCollection,
    event_store: &mut ES,
) -> Result<(), String>
where
    ES: EventStore<OrganizationCollectionEvent>
        + EventStore<UserCollectionEvent>
        + TransactionManager<Error = EventStoreError>,
{
    event_store.begin().map_err(|e| format!("{:?}", e))?;

    let uo_id = match org_collection.handle_command(
        org_id.clone(),
        &OrganizationCommand::UserReserve(name.clone()),
        event_store,
    ) {
        Ok(_) => UserOrganizationId(format!("{}-{}", org_id.0, name)),
        Err(e) => {
            event_store.rollback().unwrap();
            return Err(format!("{:?}", e));
        }
    };

    let user_id =
        match user_collection.create((name, email, org_id.clone(), uo_id.clone()), event_store) {
            Ok(id) => id,
            Err(e) => {
                event_store.rollback().unwrap();
                return Err(format!("{:?}", e));
            }
        };
    match org_collection.handle_command(
        org_id.clone(),
        &OrganizationCommand::AddUser(user_id.clone(), uo_id.clone()),
        event_store,
    ) {
        Ok(_) => {}
        Err(e) => {
            event_store.rollback().unwrap();
            return Err(format!("{:?}", e));
        }
    }
    event_store.commit().map_err(|e| format!("{:?}", e))?;
    Ok(())
}

fn main() {
    let mut org_collection = OrganizationCollection {};
    let mut user_collection = UserCollection { length: 0 };
    let mut event_store = OnMemoryEventStore {
        org_events: HashMap::new(),
        org_uncommitted_events: HashMap::new(),
        user_events: HashMap::new(),
        user_uncommitted_events: HashMap::new(),
        broker: OnMemoryEventBroker {},
        in_transaction: false,
    };

    event_store.begin().unwrap();
    let org_id = org_collection
        .create(("Example Corp.".to_string(), 3), &mut event_store)
        .unwrap();
    event_store.commit().unwrap();

    user_add_usecase(
        "Alice".to_string(),
        "alice@example.com".to_string(),
        org_id.clone(),
        &mut org_collection,
        &mut user_collection,
        &mut event_store,
    )
    .unwrap();

    user_add_usecase(
        "Bob".to_string(),
        "bob@example.com".to_string(),
        org_id.clone(),
        &mut org_collection,
        &mut user_collection,
        &mut event_store,
    )
    .unwrap();

    user_add_usecase(
        "Charlie".to_string(),
        "char@example.com".to_string(),
        org_id.clone(),
        &mut org_collection,
        &mut user_collection,
        &mut event_store,
    )
    .unwrap();

    let res = user_add_usecase(
        "David".to_string(),
        "david@exapmle.com".to_string(),
        org_id.clone(),
        &mut org_collection,
        &mut user_collection,
        &mut event_store,
    );

    assert!(res.is_err());
    assert_eq!(event_store.user_events.len(), 3);
}
