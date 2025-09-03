use crate::domain::model::order::{Order, OrderStatus};
use crate::domain::model::transaction::{Transaction, TransactionAmountAbs, TransactionError};
use crate::domain::model::user::UserId;
use crate::domain::repository::transaction::TransactionRepository;
use crate::domain::repository::user::UserRepository;
use crate::domain::service::order::order_dto::TransactionDataDto;
use crate::domain::service::order::OrderService;
use crate::domain::service::order_status::OrderStatusManagerService;
use crate::domain::service::transaction::{TransactionService, TransactionServiceError};
use async_trait::async_trait;
use rust_decimal::prelude::{ToPrimitive, Zero};
use rust_decimal::Decimal;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

pub struct TransactionServiceImpl<U, R, O, OS>
where
    U: UserRepository,
    R: TransactionRepository,
    O: OrderService,
    OS: OrderStatusManagerService,
{
    user_repository: Arc<U>,
    transaction_repository: Arc<R>,
    order_service: Arc<O>,
    order_status_manager_service: Arc<OS>,
}

impl<U, R, O, OS> TransactionServiceImpl<U, R, O, OS>
where
    U: UserRepository,
    R: TransactionRepository,
    O: OrderService,
    OS: OrderStatusManagerService,
{
    pub fn new(
        user_repository: Arc<U>,
        transaction_repository: Arc<R>,
        order_service: Arc<O>,
        order_status_manager_service: Arc<OS>,
    ) -> Self {
        Self {
            user_repository,
            transaction_repository,
            order_service,
            order_status_manager_service,
        }
    }
}

#[async_trait]
impl<U, R, O, OS> TransactionService for TransactionServiceImpl<U, R, O, OS>
where
    U: UserRepository,
    R: TransactionRepository,
    O: OrderService,
    OS: OrderStatusManagerService,
{
    #[instrument(skip(self))]
    async fn recharge(
        &self,
        user_id: UserId,
        amount: TransactionAmountAbs,
    ) -> Result<Uuid, TransactionServiceError> {
        if self
            .user_repository
            .find(user_id)
            .await
            .inspect_err(|e| {
                error!("Failed to find user: {:?}", e);
            })?
            .is_none()
        {
            return Err(TransactionServiceError::InvalidUser(user_id));
        }

        let mut tx = Transaction::new_recharge(user_id, amount);

        self.transaction_repository
            .save(&mut tx)
            .await
            .inspect_err(|e| error!("failed to save transaction: {}", e))?;

        Ok(tx.uuid())
    }

    #[instrument(skip(self))]
    async fn get_balance(&self, user_id: UserId) -> Result<Decimal, TransactionServiceError> {
        Ok(self
            .transaction_repository
            .get_user_balance(user_id)
            .await
            .inspect_err(|e| {
                error!("Failed to get user balance: {:?}", e);
            })?
            .unwrap_or(Decimal::zero()))
    }

    #[instrument(skip(self))]
    async fn new_transaction(
        &self,
        user_id: UserId,
        orders: Vec<Box<dyn Order>>,
        atomic: bool,
    ) -> Result<Uuid, TransactionServiceError> {
        for order in &*orders {
            if order.order_status() != OrderStatus::Unpaid {
                return Err(TransactionServiceError::InvalidOrderStatus {
                    op: "new",
                    status: order.order_status(),
                    order_id: order.uuid(),
                    transaction_id: None,
                });
            }
        }

        if self
            .user_repository
            .find(user_id)
            .await
            .inspect_err(|e| error!("Failed to find user: {:?}", e))?
            .is_none()
        {
            return Err(TransactionServiceError::InvalidUser(user_id));
        }

        let mut tx = Transaction::new(user_id, orders.clone(), atomic);

        self.transaction_repository
            .save(&mut tx)
            .await
            .inspect_err(|e| {
                error!("Failed to save transaction: {:?}", e);
            })?;

        Ok(tx.uuid())
    }

    #[instrument(skip(self))]
    async fn pay_transaction(&self, transaction_id: Uuid) -> Result<(), TransactionServiceError> {
        info!("Paying transaction: {}", transaction_id);
        let mut tx = self
            .transaction_repository
            .find_by_uuid(transaction_id)
            .await
            .inspect_err(|e| {
                error!("Failed to find transaction: {:?}", e);
            })?
            .ok_or(TransactionServiceError::InvalidTransactionId(
                transaction_id,
            ))
            .inspect_err(|e| {
                error!("No transaction found: {:?}", e);
            })?;

        info!("Transaction loaded");

        let available_balance = self.get_balance(tx.user_id()).await.inspect_err(|e| {
            error!("Failed to get user balance: {:?}", e);
        })?;

        // 注意，若交易是充值/退款交易，其raw_amount < 0，将通过此检查，而在之后的pay方法中被拒绝
        if available_balance < tx.raw_amount() {
            return Err(TransactionServiceError::InsufficientFunds {
                transaction_id: tx.uuid(),
                balance: available_balance,
                amount: TransactionAmountAbs::from(tx.raw_amount()),
            });
        }

        info!("Balance check passed");

        tx.pay().map_err(|e| match e {
            TransactionError::AlreadyPaid(_) => TransactionServiceError::InvalidTransactionStatus {
                op: "pay",
                status: tx.status(),
                transaction_id: tx.uuid(),
            },
            _ => panic!("Unexpected error: {:?}", e),
        })?;

        debug!("saving paid transaction: {:?}", tx);

        for order in tx.orders_mut() {
            order.set_status(OrderStatus::Paid);
        }

        self.transaction_repository
            .save(&mut tx)
            .await
            .inspect_err(|e| {
                error!("Failed to save transaction: {:?}", e);
            })?;

        let orders = tx
            .orders()
            .iter()
            .map(|order| order.as_ref())
            .collect::<Vec<_>>();

        self.order_status_manager_service
            .notify_status_change(transaction_id, tx.atomic(), &orders, OrderStatus::Paid)
            .await;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn refund_transaction(
        &self,
        transaction_id: Uuid,
        to_refund_orders: &[Box<dyn Order>],
    ) -> Result<Uuid, TransactionServiceError> {
        let mut tx = self
            .transaction_repository
            .find_by_uuid(transaction_id)
            .await
            .inspect_err(|e| {
                error!("Failed to find transaction: {:?}", e);
            })?
            .ok_or(TransactionServiceError::InvalidTransactionId(
                transaction_id,
            ))?;

        let to_refund_order_uuid_set = to_refund_orders
            .iter()
            .map(|o| o.uuid())
            .collect::<HashSet<_>>();

        let mut refund_tx = tx.refund_transaction_partial(to_refund_orders)?;

        let refund_tx_id = self
            .transaction_repository
            .save(&mut refund_tx)
            .await
            .inspect_err(|e| {
                error!("Failed to save transaction: {:?}", e);
            })?;

        for order in tx.orders_mut() {
            if to_refund_order_uuid_set.contains(&order.uuid()) {
                order
                    .payment_info_mut()
                    .set_refund_transaction_id(refund_tx_id);
            }
        }

        self.transaction_repository
            .save(&mut tx)
            .await
            .inspect_err(|e| {
                error!("Failed to save transaction: {:?}", e);
            })?;

        let orders = tx
            .orders()
            .iter()
            .filter(|order| to_refund_order_uuid_set.contains(&order.uuid()))
            .map(|order| order.as_ref())
            .collect::<Vec<_>>();

        self.order_status_manager_service
            .notify_status_change(transaction_id, tx.atomic(), &orders, OrderStatus::Cancelled)
            .await;

        Ok(refund_tx.uuid())
    }

    #[instrument(skip(self))]
    async fn convert_transaction_to_dto(
        &self,
        transaction: Transaction,
    ) -> Result<TransactionDataDto, TransactionServiceError> {
        let mut dto = TransactionDataDto {
            transaction_id: transaction.uuid().to_string(),
            status: transaction.status().to_string(),
            create_time: transaction.create_time().to_rfc3339(),
            pay_time: transaction.finish_time().map(|dt| dt.to_rfc3339()),
            amount: transaction.amount().to_f64().unwrap_or(0.0),
            orders: Vec::new(),
        };

        let origin_orders = transaction.into_orders();

        let mut orders = Vec::with_capacity(origin_orders.len());

        for order in origin_orders {
            debug!("Converting order to DTO: {:?}", order);
            orders.push(self.order_service.convert_order_to_dto(order).await?)
        }

        dto.orders = orders;

        Ok(dto)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::order::{
        BaseOrder, Order, OrderStatus, OrderTimeInfo, PaymentInfo, TrainOrder,
    };
    use crate::domain::model::password::{HashedPassword, PasswordSalt};
    use crate::domain::model::personal_info::PreferredSeatLocation;
    use crate::domain::model::train::{SeatType, SeatTypeName};
    use crate::domain::model::train_schedule::{Seat, SeatLocationInfo, SeatStatus, StationRange};
    use crate::domain::model::transaction::{Transaction, TransactionAmountAbs};
    use crate::domain::model::user::{
        IdentityCardId, PasswordAttempts, Phone, RealName, User, UserInfo, Username,
    };
    use crate::domain::repository::mock::transaction::MockTransactionRepository;
    use crate::domain::repository::mock::user::MockUserRepository;
    use crate::domain::service::mock::order::MockOrderService;
    use crate::domain::service::mock::order_status::mock_order_status_manager_service;
    use crate::domain::service::order::order_dto::{BaseOrderDto, OrderInfoDto, TrainOrderDto};
    use rust_decimal::Decimal;
    use std::sync::Arc;
    use uuid::Uuid;

    fn create_train_order(status: OrderStatus) -> TrainOrder {
        let base_order = BaseOrder::new(
            Some(1u64.into()),
            Uuid::new_v4(),
            status,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        TrainOrder::new(
            base_order,
            1u64.into(),
            Some(Seat::new(
                1u64.into(),
                SeatType::new(
                    Some(1u64.into()),
                    SeatTypeName::from_unchecked("二等座".to_string()),
                    1,
                    Decimal::new(1000, 2),
                ),
                SeatLocationInfo {
                    carriage: 3,
                    row: 11,
                    location: 'A',
                },
                SeatStatus::Occupied,
            )),
            SeatTypeName::from_unchecked("二等座".to_string()),
            Some(PreferredSeatLocation::A),
            StationRange::from_unchecked(1u64.into(), 1u64.into()),
        )
    }

    #[tokio::test]
    async fn test_recharge_success() {
        let mut user_repo = MockUserRepository::new();
        let mut tx_repo = MockTransactionRepository::new();

        user_repo.expect_find().returning(|_| {
            Ok(Some(User::new(
                Some(1u64.into()),
                Username::try_from("日升".to_string()).unwrap(),
                HashedPassword {
                    hashed_password: vec![],
                    salt: PasswordSalt::from(vec![]),
                },
                None,
                PasswordAttempts::default(),
                UserInfo::new(
                    RealName::try_from("日升".to_string()).unwrap(),
                    None,
                    None,
                    Phone::try_from("17188888888".to_string()).unwrap(),
                    None,
                    IdentityCardId::try_from("11010519491231002X".to_string()).unwrap(),
                ),
            )))
        });
        tx_repo.expect_save().returning(|_| Ok(1u64.into()));

        let service = TransactionServiceImpl::new(
            Arc::new(user_repo),
            Arc::new(tx_repo),
            Arc::new(MockOrderService::new()),
            Arc::new(mock_order_status_manager_service()),
        );

        let res = service
            .recharge(
                1u64.into(),
                TransactionAmountAbs::from(Decimal::new(1000, 2)),
            )
            .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_recharge_fail_invalid_user() {
        let mut user_repo = MockUserRepository::new();
        user_repo.expect_find().returning(|_| Ok(None));

        let service = TransactionServiceImpl::new(
            Arc::new(user_repo),
            Arc::new(MockTransactionRepository::new()),
            Arc::new(MockOrderService::new()),
            Arc::new(mock_order_status_manager_service()),
        );

        let res = service
            .recharge(
                1u64.into(),
                TransactionAmountAbs::from(Decimal::new(1000, 2)),
            )
            .await;
        assert!(matches!(res, Err(TransactionServiceError::InvalidUser(_))));
    }

    #[tokio::test]
    async fn test_get_balance_success() {
        let mut tx_repo = MockTransactionRepository::new();
        tx_repo
            .expect_get_user_balance()
            .returning(|_| Ok(Some(Decimal::new(5000, 2))));

        let service = TransactionServiceImpl::new(
            Arc::new(MockUserRepository::new()),
            Arc::new(tx_repo),
            Arc::new(MockOrderService::new()),
            Arc::new(mock_order_status_manager_service()),
        );

        let balance = service.get_balance(1u64.into()).await.unwrap();
        assert_eq!(balance, Decimal::new(5000, 2));
    }

    #[tokio::test]
    async fn test_get_balance_none_returns_zero() {
        let mut tx_repo = MockTransactionRepository::new();
        tx_repo.expect_get_user_balance().returning(|_| Ok(None));

        let service = TransactionServiceImpl::new(
            Arc::new(MockUserRepository::new()),
            Arc::new(tx_repo),
            Arc::new(MockOrderService::new()),
            Arc::new(mock_order_status_manager_service()),
        );

        let balance = service.get_balance(1u64.into()).await.unwrap();
        assert_eq!(balance, Decimal::zero());
    }

    #[tokio::test]
    async fn test_new_transaction_success() {
        let mut user_repo = MockUserRepository::new();
        let mut tx_repo = MockTransactionRepository::new();
        user_repo.expect_find().returning(|_| {
            Ok(Some(User::new(
                Some(1u64.into()),
                Username::try_from("日升".to_string()).unwrap(),
                HashedPassword {
                    hashed_password: vec![],
                    salt: PasswordSalt::from(vec![]),
                },
                None,
                PasswordAttempts::default(),
                UserInfo::new(
                    RealName::try_from("日升".to_string()).unwrap(),
                    None,
                    None,
                    Phone::try_from("17188888888".to_string()).unwrap(),
                    None,
                    IdentityCardId::try_from("11010519491231002X".to_string()).unwrap(),
                ),
            )))
        });
        tx_repo.expect_save().returning(|_| Ok(1u64.into()));

        let service = TransactionServiceImpl::new(
            Arc::new(user_repo),
            Arc::new(tx_repo),
            Arc::new(MockOrderService::new()),
            Arc::new(mock_order_status_manager_service()),
        );

        let orders: Vec<Box<dyn Order>> = vec![Box::new(create_train_order(OrderStatus::Unpaid))];
        let tx_id = service
            .new_transaction(1u64.into(), orders, false)
            .await
            .unwrap();
        assert!(!tx_id.is_nil());
    }

    #[tokio::test]
    async fn test_new_transaction_fail_invalid_order_status() {
        let service = TransactionServiceImpl::new(
            Arc::new(MockUserRepository::new()),
            Arc::new(MockTransactionRepository::new()),
            Arc::new(MockOrderService::new()),
            Arc::new(mock_order_status_manager_service()),
        );

        let orders: Vec<Box<dyn Order>> = vec![Box::new(create_train_order(OrderStatus::Paid))];
        let res = service.new_transaction(1u64.into(), orders, false).await;
        assert!(matches!(
            res,
            Err(TransactionServiceError::InvalidOrderStatus { .. })
        ));
    }

    #[tokio::test]
    async fn test_new_transaction_fail_invalid_user() {
        let mut user_repo = MockUserRepository::new();
        user_repo.expect_find().returning(|_| Ok(None));

        let service = TransactionServiceImpl::new(
            Arc::new(user_repo),
            Arc::new(MockTransactionRepository::new()),
            Arc::new(MockOrderService::new()),
            Arc::new(mock_order_status_manager_service()),
        );

        let orders: Vec<Box<dyn Order>> = vec![Box::new(create_train_order(OrderStatus::Unpaid))];
        let res = service.new_transaction(1u64.into(), orders, false).await;
        assert!(matches!(res, Err(TransactionServiceError::InvalidUser(_))));
    }

    #[tokio::test]
    async fn test_pay_transaction_success() {
        let mut tx_repo = MockTransactionRepository::new();
        let order_status_mgr = mock_order_status_manager_service();

        let tx = Transaction::new(
            1u64.into(),
            vec![Box::new(create_train_order(OrderStatus::Unpaid))],
            false,
        );
        let tx_uuid = tx.uuid();

        tx_repo
            .expect_find_by_uuid()
            .returning(move |_| Ok(Some(tx.clone())));
        tx_repo
            .expect_get_user_balance()
            .returning(|_| Ok(Some(Decimal::new(1000, 0))));
        tx_repo.expect_save().returning(|_| Ok(1u64.into()));

        let service = TransactionServiceImpl::new(
            Arc::new(MockUserRepository::new()),
            Arc::new(tx_repo),
            Arc::new(MockOrderService::new()),
            Arc::new(order_status_mgr),
        );

        let res = service.pay_transaction(tx_uuid).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_pay_transaction_fail_invalid_transaction() {
        let mut tx_repo = MockTransactionRepository::new();
        let tx_uuid = Uuid::new_v4();
        tx_repo.expect_find_by_uuid().returning(move |_| Ok(None));

        let service = TransactionServiceImpl::new(
            Arc::new(MockUserRepository::new()),
            Arc::new(tx_repo),
            Arc::new(MockOrderService::new()),
            Arc::new(mock_order_status_manager_service()),
        );

        let res = service.pay_transaction(tx_uuid).await;
        assert!(matches!(
            res,
            Err(TransactionServiceError::InvalidTransactionId(_))
        ));
    }

    #[tokio::test]
    async fn test_refund_transaction_success() {
        let mut tx_repo = MockTransactionRepository::new();
        let order_status_mgr = mock_order_status_manager_service();

        let order = Box::new(create_train_order(OrderStatus::Paid));
        let mut tx = Transaction::new(1u64.into(), vec![order.clone()], false);
        tx.pay().unwrap();
        let tx_uuid = tx.uuid();

        tx_repo
            .expect_find_by_uuid()
            .returning(move |_| Ok(Some(tx.clone())));
        tx_repo.expect_save().returning(|_| Ok(1u64.into()));

        let service = TransactionServiceImpl::new(
            Arc::new(MockUserRepository::new()),
            Arc::new(tx_repo),
            Arc::new(MockOrderService::new()),
            Arc::new(order_status_mgr),
        );

        let res = service.refund_transaction(tx_uuid, &[order]).await;

        println!("res: {:?}", res);

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_convert_transaction_to_dto_success() {
        let mut order_svc = MockOrderService::new();
        let order = Box::new(create_train_order(OrderStatus::Paid));

        let base_order_dto = BaseOrderDto {
            order_id: "1".to_string(),
            status: "Paid".to_string(),
            unit_price: 0.0,
            amount: 0,
            can_cancel: false,
            reason: None,
            order_type: "Train".to_string(),
        };

        order_svc.expect_convert_order_to_dto().returning(move |_| {
            Ok(OrderInfoDto::Train(TrainOrderDto {
                base: base_order_dto.clone(),
                train_number: "G1".to_string(),
                departure_station: "北京南".to_string(),
                arrival_station: "上海虹桥".to_string(),
                departure_time: "1".to_string(),
                arrival_time: "2".to_string(),
                origin_station: "北京南".to_string(),
                terminal_station: "上海虹桥".to_string(),
                origin_departure_time: "1".to_string(),
                terminal_arrival_time: "2".to_string(),
                name: "日升".to_string(),
                seat: None,
            }))
        });

        let service = TransactionServiceImpl::new(
            Arc::new(MockUserRepository::new()),
            Arc::new(MockTransactionRepository::new()),
            Arc::new(order_svc),
            Arc::new(mock_order_status_manager_service()),
        );

        let tx = Transaction::new(1u64.into(), vec![order], false);
        let dto = service.convert_transaction_to_dto(tx).await.unwrap();
        assert_eq!(dto.orders.len(), 1);
    }
}
