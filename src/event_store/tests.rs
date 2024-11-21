use std::collections::HashMap;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct OrderId(String);

#[derive(Debug, Clone, PartialEq)]
enum OrderStatus {
    Pending,
    Shipped,
    Delivered,
}

#[derive(Debug)]
struct Order {
    _id: OrderId,
    status: OrderStatus,
}

#[derive(Debug, Clone)]
struct CreateOrderEvent {
    id: OrderId,
}

#[derive(Debug, Clone)]
enum OrderAction {
    Ship,
    Deliver,
}

#[derive(Debug, Clone)]
struct OrderResolveEvent {
    id: OrderId,
    data: OrderAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PaymentId(String);

#[derive(Debug, Clone, PartialEq)]
struct PaymentStatus(i32);

#[derive(Debug)]
struct Payment {
    _id: PaymentId,
    status: PaymentStatus,
}

#[derive(Debug, Clone)]
struct CreatePaymentEvent {
    id: PaymentId,
    price: i32,
}

#[derive(Debug, Clone)]
enum PaymentAction {
    Capture(i32),
    Refund(i32),
}

#[derive(Debug, Clone)]
struct PaymentResolveEvent {
    id: PaymentId,
    data: PaymentAction,
}

#[derive(Debug, Clone)]
struct OnMemoryEventMetadata((String));

#[derive(Debug, Clone)]
enum OnMemoryPersistableEvent {
    OrderCreate(CreateOrderEvent, OnMemoryEventMetadata),
    OrderResolve(OrderResolveEvent, OnMemoryEventMetadata),
    PaymentCreate(CreatePaymentEvent, OnMemoryEventMetadata),
    PaymentResolve(PaymentResolveEvent, OnMemoryEventMetadata),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum OnMemoryPersistableEventId {
    Order(OrderId),
    Payment(PaymentId),
}

struct OnMemoryEventStore {
    is_transaction_active: bool,
    events: HashMap<OnMemoryPersistableEventId, Vec<OnMemoryPersistableEvent>>,
}

impl OnMemoryEventStore {
    fn new() -> Self {
        Self {
            is_transaction_active: false,
            events: HashMap::new(),
        }
    }
}

impl EventStore for OnMemoryEventStore {
    type Persistable = OnMemoryPersistableEvent;
    type Error = OnMemoryEventStoreError;

    fn save(&mut self, events: &[Self::Persistable]) -> Result<(), Self::Error> {
        for event in events {
            let id = match event {
                OnMemoryPersistableEvent::OrderCreate(event, _) => {
                    OnMemoryPersistableEventId::Order(event.id.clone())
                }
                OnMemoryPersistableEvent::OrderResolve(event, _) => {
                    OnMemoryPersistableEventId::Order(event.id.clone())
                }
                OnMemoryPersistableEvent::PaymentCreate(event, _) => {
                    OnMemoryPersistableEventId::Payment(event.id.clone())
                }
                OnMemoryPersistableEvent::PaymentResolve(event, _) => {
                    OnMemoryPersistableEventId::Payment(event.id.clone())
                }
            };
            let events = self.events.entry(id).or_insert_with(Vec::new);
            events.push(event.clone());
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

#[derive(Debug)]
struct OnMemoryEventStoreError;

impl std::fmt::Display for OnMemoryEventStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OnMemoryEventStoreError")
    }
}

impl std::error::Error for OnMemoryEventStoreError {}

struct OrderQuery {
    id: OrderId,
}

impl QueryHandler<OrderQuery> for OnMemoryEventStore {
    type Response = Option<Order>;
    type Error = OnMemoryEventStoreError;

    fn handle(&self, query: OrderQuery) -> Result<Self::Response, Self::Error> {
        let id = OnMemoryPersistableEventId::Order(query.id.clone());
        let events = self.events.get(&id).unwrap();
        let mut order = None;
        for event in events {
            match event {
                OnMemoryPersistableEvent::OrderCreate(_, _) => {
                    order = Some(Order {
                        _id: query.id.clone(),
                        status: OrderStatus::Pending,
                    });
                }
                OnMemoryPersistableEvent::OrderResolve(event, _) => match order.as_mut() {
                    Some(order) => {
                        order.status = match event.data {
                            OrderAction::Ship => OrderStatus::Shipped,
                            OrderAction::Deliver => OrderStatus::Delivered,
                        };
                    }
                    None => {
                        return Err(OnMemoryEventStoreError);
                    }
                },
                _ => {}
            }
        }
        Ok(order)
    }
}

struct PaymentQuery {
    id: PaymentId,
}

impl QueryHandler<PaymentQuery> for OnMemoryEventStore {
    type Response = Option<Payment>;
    type Error = OnMemoryEventStoreError;

    fn handle(&self, query: PaymentQuery) -> Result<Self::Response, Self::Error> {
        let id = OnMemoryPersistableEventId::Payment(query.id.clone());
        let events = self.events.get(&id).unwrap();
        let mut payment = None;
        for event in events {
            match event {
                OnMemoryPersistableEvent::PaymentCreate(event, _) => {
                    payment = Some(Payment {
                        _id: query.id.clone(),
                        status: PaymentStatus(event.price),
                    });
                }
                OnMemoryPersistableEvent::PaymentResolve(event, _) => match payment.as_mut() {
                    Some(payment) => {
                        payment.status = match event.data {
                            PaymentAction::Capture(p) => PaymentStatus(payment.status.0 - p),
                            PaymentAction::Refund(p) => PaymentStatus(payment.status.0 + p),
                        };
                    }
                    None => {
                        return Err(OnMemoryEventStoreError);
                    }
                },
                _ => {}
            }
        }
        Ok(payment)
    }
}

#[test]
fn test_order_backlog() {
    let order_id = OrderId("order-1".to_string());
    let create_order_event = CreateOrderEvent {
        id: order_id.clone(),
    };
    let ship_order_event = OrderResolveEvent {
        id: order_id.clone(),
        data: OrderAction::Ship,
    };
    let deliver_order_event = OrderResolveEvent {
        id: order_id.clone(),
        data: OrderAction::Deliver,
    };

    let mut event_store = OnMemoryEventStore::new();
    event_store.begin().unwrap();
    event_store
        .save(&[
            OnMemoryPersistableEvent::OrderCreate(
                create_order_event.clone(),
                OnMemoryEventMetadata("".to_string()),
            ),
            OnMemoryPersistableEvent::OrderResolve(
                ship_order_event.clone(),
                OnMemoryEventMetadata("".to_string()),
            ),
            OnMemoryPersistableEvent::OrderResolve(
                deliver_order_event.clone(),
                OnMemoryEventMetadata("".to_string()),
            ),
        ])
        .unwrap();
    event_store.commit().unwrap();

    let query = OrderQuery {
        id: order_id.clone(),
    };
    let order = event_store.handle(query).unwrap().unwrap();
    assert_eq!(order.status, OrderStatus::Delivered);
}

#[test]
fn test_payment_backlog() {
    let payment_id = PaymentId("payment-1".to_string());
    let create_payment_event = CreatePaymentEvent {
        id: payment_id.clone(),
        price: 100,
    };
    let capture_payment_event = PaymentResolveEvent {
        id: payment_id.clone(),
        data: PaymentAction::Capture(110),
    };
    let refund_payment_event = PaymentResolveEvent {
        id: payment_id.clone(),
        data: PaymentAction::Refund(10),
    };

    let mut event_store = OnMemoryEventStore::new();
    event_store.begin().unwrap();
    event_store
        .save(&[
            OnMemoryPersistableEvent::PaymentCreate(
                create_payment_event.clone(),
                OnMemoryEventMetadata("".to_string()),
            ),
            OnMemoryPersistableEvent::PaymentResolve(
                capture_payment_event.clone(),
                OnMemoryEventMetadata("".to_string()),
            ),
            OnMemoryPersistableEvent::PaymentResolve(
                refund_payment_event.clone(),
                OnMemoryEventMetadata("".to_string()),
            ),
        ])
        .unwrap();
    event_store.commit().unwrap();

    let query = PaymentQuery {
        id: payment_id.clone(),
    };
    let payment = event_store.handle(query).unwrap().unwrap();
    assert_eq!(payment.status, PaymentStatus(0));
}
