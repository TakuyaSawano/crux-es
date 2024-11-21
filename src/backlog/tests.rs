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
    id: OrderId,
    status: OrderStatus,
}

#[derive(Debug)]
struct CreateOrderEvent {
    id: OrderId,
}

#[derive(Debug)]
enum OrderAction {
    Ship,
    Deliver,
}

#[derive(Debug)]
struct OrderResolveData {
    action: OrderAction,
}

#[derive(Debug)]
struct OrderResolveEvent {
    id: OrderId,
    data: OrderResolveData,
}

impl Backlog for Order {
    type Id = OrderId;
    type Status = OrderStatus;
    type CreateEvent = CreateOrderEvent;
    type ResolveEvent = OrderResolveEvent;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }

    fn create(event: Self::CreateEvent) -> Self {
        Order {
            id: event.id,
            status: OrderStatus::Pending,
        }
    }

    fn resolve(&mut self, event: Self::ResolveEvent) -> &Self::Status {
        self.status = match event.data.action {
            OrderAction::Ship => OrderStatus::Shipped,
            OrderAction::Deliver => OrderStatus::Delivered,
        };
        &self.status
    }

    fn status(&self) -> &Self::Status {
        &self.status
    }
}

#[test]
fn test_order_backlog() {
    let order_id = OrderId("order-1".to_string());
    let create_event = CreateOrderEvent {
        id: order_id.clone(),
    };
    let resolve_event = OrderResolveEvent {
        id: order_id.clone(),
        data: OrderResolveData {
            action: OrderAction::Ship,
        },
    };

    let mut order = Order::create(create_event);
    assert_eq!(*order.status(), OrderStatus::Pending);

    let status = order.resolve(resolve_event);
    assert_eq!(*status, OrderStatus::Shipped);

    let resolve_event = OrderResolveEvent {
        id: order_id.clone(),
        data: OrderResolveData {
            action: OrderAction::Deliver,
        },
    };
    let status = order.resolve(resolve_event);
    assert_eq!(*status, OrderStatus::Delivered);
}
