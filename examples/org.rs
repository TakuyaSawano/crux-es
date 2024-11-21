use std::collections::HashMap;

use crux_es::{backlog::*, event_store::*};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct OrgId(String);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UserId(String);

struct Org {
    id: OrgId,
    name: String,
    users: Vec<UserId>,
    max_users: usize,
    reserved_id: Option<UserAddId>,
}

struct OrgService {
    orgs: HashMap<OrgId, Org>,
}

impl OrgService {
    fn reserve_user(&mut self, id: OrgId, user_add_id: UserAddId) -> Result<(), String> {
        let org = self.orgs.get_mut(&id).ok_or("Org not found")?;
        if org.reserved_id.is_some() {
            return Err("User already reserved".to_string());
        }
        if org.users.len() >= org.max_users {
            return Err("Max users reached".to_string());
        }
        org.reserved_id = Some(user_add_id);
        Ok(())
    }
    fn add_user(&mut self, id: OrgId, user_id: UserId) -> Result<(), String> {
        let org = self.orgs.get_mut(&id).ok_or("Org not found")?;
        if org.reserved_id.is_none() {
            return Err("No user reserved".to_string());
        }
        org.users.push(user_id.clone());
        org.reserved_id = None;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct UserData(pub String);

struct User {
    id: UserId,
    data: UserData,
    org_id: OrgId,
}

struct UserService {
    users: HashMap<UserId, User>,
}
impl UserService {
    fn create_user(&mut self, data: UserData, org_id: OrgId) -> Result<UserId, String> {
        let id = UserId(data.0.clone());
        if self.users.contains_key(&id) {
            return Err("User already exists".to_string());
        }
        self.users.insert(
            id.clone(),
            User {
                id: id.clone(),
                data,
                org_id,
            },
        );
        Ok(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UserAddId(String);

#[derive(Debug, Clone)]
enum UserAddBacklogStatus {
    Created(UserData, OrgId),
    Reserved(OrgId),
    UserCreated(UserId, UserData),
    UserAdded(UserId, OrgId),
}

struct UserAddBacklog {
    id: UserAddId,
    status: UserAddBacklogStatus,
}

#[derive(Debug, Clone)]
struct UserAddCreatedEvent {
    id: UserAddId,
    data: UserData,
    org_id: OrgId,
}

#[derive(Debug, Clone)]
enum UserAddEvent {
    Reserved(UserAddId, OrgId),
    UserCreated(UserAddId, UserId, UserData),
    UserAdded(UserAddId, UserId, OrgId),
}

impl Backlog for UserAddBacklog {
    type Id = UserAddId;
    type Status = UserAddBacklogStatus;
    type CreateEvent = UserAddCreatedEvent;
    type ResolveEvent = UserAddEvent;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }

    fn create(event: Self::CreateEvent) -> Self {
        UserAddBacklog {
            id: event.id,
            status: UserAddBacklogStatus::Created(event.data, event.org_id),
        }
    }

    fn resolve(&mut self, event: Self::ResolveEvent) -> &Self::Status {
        match event {
            UserAddEvent::Reserved(_, org_id) => {
                self.status = UserAddBacklogStatus::Reserved(org_id);
            }
            UserAddEvent::UserCreated(_, user_id, data) => {
                self.status = UserAddBacklogStatus::UserCreated(user_id, data);
            }
            UserAddEvent::UserAdded(_, user_id, org_id) => {
                self.status = UserAddBacklogStatus::UserAdded(user_id, org_id);
            }
        }
        &self.status
    }

    fn status(&self) -> &Self::Status {
        &self.status
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum PersistableEventId {
    UserAdd(UserAddId),
}

#[derive(Debug, Clone)]
enum PersistableEvent {
    UserAddCreated(UserAddCreatedEvent),
    UserAdd(UserAddEvent),
}

struct OnMemoryEventStore {
    uncommitted_events: HashMap<PersistableEventId, Vec<PersistableEvent>>,
    is_transaction_active: bool,
    events: HashMap<PersistableEventId, Vec<PersistableEvent>>,
}

#[derive(Debug)]
struct OnMemoryEventStoreError;
impl std::fmt::Display for OnMemoryEventStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OnMemoryEventStoreError")
    }
}
impl std::error::Error for OnMemoryEventStoreError {}

impl EventStore for OnMemoryEventStore {
    type Persistable = PersistableEvent;
    type Error = OnMemoryEventStoreError;

    fn save(&mut self, events: &[Self::Persistable]) -> Result<(), Self::Error> {
        for event in events {
            match event {
                PersistableEvent::UserAddCreated(event) => {
                    let id = PersistableEventId::UserAdd(event.id.clone());
                    let events = self.uncommitted_events.entry(id).or_default();
                    events.push(PersistableEvent::UserAddCreated(event.clone()));
                }
                PersistableEvent::UserAdd(event) => {
                    let id = match event {
                        UserAddEvent::Reserved(id, _) => PersistableEventId::UserAdd(id.clone()),
                        UserAddEvent::UserCreated(id, _, _) => {
                            PersistableEventId::UserAdd(id.clone())
                        }
                        UserAddEvent::UserAdded(id, _, _) => {
                            PersistableEventId::UserAdd(id.clone())
                        }
                    };
                    let events = self.uncommitted_events.entry(id).or_default();
                    events.push(PersistableEvent::UserAdd(event.clone()));
                }
            }
        }
        Ok(())
    }
}

impl TransactionManager for OnMemoryEventStore {
    type Error = OnMemoryEventStoreError;

    fn begin(&mut self) -> Result<(), Self::Error> {
        self.is_transaction_active = true;
        Ok(())
    }

    fn commit(&mut self) -> Result<(), Self::Error> {
        if !self.is_transaction_active {
            return Err(OnMemoryEventStoreError);
        }
        for (id, unc_events) in self.uncommitted_events.drain() {
            let events = self.events.entry(id).or_default();
            events.extend(unc_events);
        }
        self.is_transaction_active = false;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), Self::Error> {
        if !self.is_transaction_active {
            return Err(OnMemoryEventStoreError);
        }
        self.is_transaction_active = false;
        Ok(())
    }
}

fn create_user<ES: EventStore<Persistable = PersistableEvent> + TransactionManager>(
    userdata: UserData,
    org_id: OrgId,
    us: &mut UserService,
    os: &mut OrgService,
    es: &mut ES,
) -> Result<String, String> {
    let user_add_id = UserAddId(userdata.0.clone());
    let event = UserAddCreatedEvent {
        id: user_add_id.clone(),
        data: userdata.clone(),
        org_id: org_id.clone(),
    };
    let _backlog = UserAddBacklog::create(event.clone());
    es.begin().map_err(|e| e.to_string())?;
    es.save(&[PersistableEvent::UserAddCreated(event.clone())])
        .map_err(|e| e.to_string())?;
    es.commit().map_err(|e| e.to_string())?;

    os.reserve_user(org_id.clone(), user_add_id.clone())
        .map_err(|e| e.to_string())?;
    es.begin().map_err(|e| e.to_string())?;
    es.save(&[PersistableEvent::UserAdd(UserAddEvent::Reserved(
        user_add_id.clone(),
        org_id.clone(),
    ))])
    .map_err(|e| e.to_string())?;
    es.commit().map_err(|e| e.to_string())?;

    let user_id = us
        .create_user(userdata.clone(), org_id.clone())
        .map_err(|e| e.to_string())?;
    es.begin().map_err(|e| e.to_string())?;
    es.save(&[PersistableEvent::UserAdd(UserAddEvent::UserCreated(
        user_add_id.clone(),
        user_id.clone(),
        userdata.clone(),
    ))])
    .map_err(|e| e.to_string())?;
    es.commit().map_err(|e| e.to_string())?;

    os.add_user(org_id.clone(), user_id.clone())
        .map_err(|e| e.to_string())?;
    es.begin().map_err(|e| e.to_string())?;
    es.save(&[PersistableEvent::UserAdd(UserAddEvent::UserAdded(
        user_add_id.clone(),
        user_id.clone(),
        org_id,
    ))])
    .map_err(|e| e.to_string())?;
    es.commit().map_err(|e| e.to_string())?;
    Ok(user_add_id.0)
}

fn main() {
    let mut us = UserService {
        users: HashMap::new(),
    };

    let org_id = OrgId("org-1".to_string());
    let org = Org {
        id: org_id.clone(),
        name: "Org 1".to_string(),
        users: vec![],
        max_users: 3,
        reserved_id: None,
    };
    let mut os = OrgService {
        orgs: HashMap::new(),
    };
    os.orgs.insert(org_id.clone(), org);

    let mut es = OnMemoryEventStore {
        uncommitted_events: HashMap::new(),
        is_transaction_active: false,
        events: HashMap::new(),
    };

    let userdata = UserData("user-1".to_string());
    let user_add_id = create_user(userdata, org_id.clone(), &mut us, &mut os, &mut es).unwrap();
    println!("User Add ID: {}", user_add_id);

    let userdata = UserData("user-2".to_string());
    let user_add_id = create_user(userdata, org_id.clone(), &mut us, &mut os, &mut es).unwrap();
    println!("User Add ID: {}", user_add_id);

    let userdata = UserData("user-3".to_string());
    let user_add_id = create_user(userdata, org_id.clone(), &mut us, &mut os, &mut es).unwrap();
    println!("User Add ID: {}", user_add_id);

    let userdata = UserData("user-4".to_string());
    let user_add_id = create_user(userdata, org_id.clone(), &mut us, &mut os, &mut es);
    assert_eq!(user_add_id, Err("Max users reached".to_string()));
}
