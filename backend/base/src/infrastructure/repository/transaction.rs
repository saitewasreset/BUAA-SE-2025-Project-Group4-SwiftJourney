use std::any::{Any, TypeId};
use crate::domain::model::transaction::{Transaction, TransactionId};
use crate::domain::service::AggregateManagerImpl;
use crate::domain::{DbRepositorySupport, MultiEntityDiff, RepositoryError};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::{Arc, Mutex};
use crate::domain::model::order::{DishOrder, HotelOrder, Order, TakeawayOrder, TrainOrder};

pub struct TransactionDataConverter;

pub struct OrderPack {
    pub train_orders: Vec<TrainOrder>,
    pub hotel_orders: Vec<HotelOrder>,
    pub dish_orders: Vec<DishOrder>,
    pub takeaway_orders: Vec<TakeawayOrder>,
}

impl From<Vec<Box<dyn Order>>> for OrderPack {
    fn from(orders: Vec<Box<dyn Order>>) -> Self {
        let mut train_orders = Vec::new();
        let mut hotel_orders = Vec::new();
        let mut dish_orders = Vec::new();
        let mut takeaway_orders = Vec::new();

        for order in orders {
            let order = order as Box<dyn Any>;
            match order.as_ref().type_id() {
                id if id == TypeId::of::<TrainOrder>() => {
                    train_orders.push(*order.downcast::<TrainOrder>().unwrap());
                }
                id if id == TypeId::of::<HotelOrder>() => {
                    hotel_orders.push(*order.downcast::<HotelOrder>().unwrap());
                }
                id if id == TypeId::of::<DishOrder>() => {
                    dish_orders.push(*order.downcast::<DishOrder>().unwrap());
                }
                id if id == TypeId::of::<TakeawayOrder>() => {
                    takeaway_orders.push(*order.downcast::<TakeawayOrder>().unwrap());
                }
                _ => panic!("Unknown order type"),
            }
        }

        OrderPack {
            train_orders,
            hotel_orders,
            dish_orders,
            takeaway_orders,
        }
    }
}

impl From<OrderPack> for Vec<Box<dyn Order>> {
    fn from(value: OrderPack) -> Self {
        let mut result: Vec<Box<dyn Order>> = Vec::new();

        for order in value.train_orders {
            result.push(Box::new(order));
        }

        for order in value.hotel_orders {
            result.push(Box::new(order));
        }

        for order in value.dish_orders {
            result.push(Box::new(order));
        }

        for order in value.takeaway_orders {
            result.push(Box::new(order));
        }

        result
    }
}

pub struct TransactionDoPack {
    pub transaction: crate::models::transaction::Model,
}

impl TransactionDataConverter {
    pub fn make_from_do();

    pub fn transform_to_do(transaction: Transaction)
}

pub struct TransactionRepositoryImpl {
    db: DatabaseConnection,
    aggregate_manager: Arc<Mutex<AggregateManagerImpl<Transaction>>>,
}

#[async_trait]
impl DbRepositorySupport<Transaction> for TransactionRepositoryImpl {
    type Manager = AggregateManagerImpl<Transaction>;

    fn get_aggregate_manager(&self) -> Arc<Mutex<Self::Manager>> {
        self.aggregate_manager.clone()
    }

    async fn on_insert(&self, aggregate: Transaction) -> Result<TransactionId, RepositoryError> {
        todo!()
    }

    async fn on_select(&self, id: TransactionId) -> Result<Option<Transaction>, RepositoryError> {
        todo!()
    }

    async fn on_update(&self, diff: MultiEntityDiff) -> Result<(), RepositoryError> {
        todo!()
    }

    async fn on_delete(&self, aggregate: Transaction) -> Result<(), RepositoryError> {
        todo!()
    }
}
