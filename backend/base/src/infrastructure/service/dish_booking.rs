use crate::domain::model::order::{DishOrder, Order, OrderStatus};
use crate::domain::repository::order::OrderRepository;
use crate::domain::service::dish_booking::{DishBookingService, DishBookingServiceError};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

pub struct DishBookingServiceImpl<OR>
where
    OR: OrderRepository,
{
    order_repository: Arc<OR>,
}

impl<OR> DishBookingServiceImpl<OR>
where
    OR: OrderRepository,
{
    pub fn new(order_repository: Arc<OR>) -> Self {
        Self { order_repository }
    }
}

#[async_trait]
impl<OR> DishBookingService for DishBookingServiceImpl<OR>
where
    OR: OrderRepository,
{
    #[instrument(skip(self))]
    async fn booking_dish(&self, order_uuid: Uuid) -> Result<(), DishBookingServiceError> {
        let mut order = self
            .order_repository
            .find_dish_order_by_uuid(order_uuid)
            .await?
            .ok_or(DishBookingServiceError::InvalidOrder(order_uuid))?;

        if order.order_status() != OrderStatus::Paid {
            return Err(DishBookingServiceError::InvalidOrderStatus(
                order_uuid,
                order.order_status(),
            ));
        }

        // 火车餐订单总是会成功

        order.set_status(OrderStatus::Ongoing);

        self.order_repository
            .update(Box::new(order))
            .await
            .inspect_err(|e| {
                error!("Failed to update order status: {}", e);
            })?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn cancel_dish(&self, order_uuid: Uuid) -> Result<(), DishBookingServiceError> {
        let mut order = self
            .order_repository
            .find_dish_order_by_uuid(order_uuid)
            .await?
            .ok_or(DishBookingServiceError::InvalidOrder(order_uuid))?;

        if order.order_status() != OrderStatus::Ongoing {
            return Err(DishBookingServiceError::InvalidOrderStatus(
                order_uuid,
                order.order_status(),
            ));
        }

        order.set_status(OrderStatus::Cancelled);

        self.order_repository
            .update(Box::new(order))
            .await
            .inspect_err(|e| {
                error!("Failed to update order status: {}", e);
            })?;

        Ok(())
    }

    /// 对于合法的火车餐订单，总是会成功，故本函数固定返回空的退款订单列表
    #[instrument(skip(self))]
    async fn booking_group(
        &self,
        order_uuid_list: Vec<Uuid>,
        _atomic: bool,
    ) -> Result<Vec<DishOrder>, DishBookingServiceError> {
        let mut success_booking_order_list = Vec::new();

        for order_uuid in order_uuid_list {
            if let Err(e) = self.booking_dish(order_uuid).await {
                error!("Failed to book dish: {:?}", e);
                break;
            } else {
                success_booking_order_list.push(order_uuid);
            }
        }

        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::order::{
        BaseOrder, DishOrder, OrderStatus, OrderTimeInfo, PaymentInfo,
    };
    use crate::domain::model::transaction::Transaction;
    use crate::domain::repository::mock::order::MockOrderRepository;
    use crate::domain::RepositoryError;
    use anyhow::anyhow;
    use rust_decimal::Decimal;
    use std::sync::Arc;
    use uuid::Uuid;

    fn make_dish_order(uuid: Uuid, status: OrderStatus) -> DishOrder {
        let train_order_id = 1u64.into();
        let dish_id = 1u64.into();

        let base_order = BaseOrder::new(
            Some(1u64.into()),
            uuid,
            status,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );

        DishOrder::new(
            base_order.clone(),
            train_order_id,
            dish_id,
            Decimal::new(1000, 2),
            Decimal::ONE,
        )
    }

    // -------- booking_dish --------
    #[tokio::test]
    async fn test_booking_dish_success() {
        let mut mock_repo = MockOrderRepository::new();
        let uuid = Uuid::new_v4();

        mock_repo
            .expect_find_dish_order_by_uuid()
            .returning(move |_| Ok(Some(make_dish_order(uuid, OrderStatus::Paid))));
        mock_repo.expect_update().returning(|_| Ok(()));

        let service = DishBookingServiceImpl::new(Arc::new(mock_repo));

        assert!(service.booking_dish(uuid).await.is_ok());
    }

    #[tokio::test]
    async fn test_booking_dish_invalid_order_status() {
        let mut mock_repo = MockOrderRepository::new();
        let uuid = Uuid::new_v4();

        mock_repo
            .expect_find_dish_order_by_uuid()
            .returning(move |_| Ok(Some(make_dish_order(uuid, OrderStatus::Cancelled))));

        let service = DishBookingServiceImpl::new(Arc::new(mock_repo));

        let result = service.booking_dish(uuid).await;
        assert!(matches!(
            result,
            Err(DishBookingServiceError::InvalidOrderStatus(_, _))
        ));
    }

    #[tokio::test]
    async fn test_booking_dish_order_not_found() {
        let mut mock_repo = MockOrderRepository::new();
        let uuid = Uuid::new_v4();

        mock_repo
            .expect_find_dish_order_by_uuid()
            .returning(|_| Ok(None));

        let service = DishBookingServiceImpl::new(Arc::new(mock_repo));

        let result = service.booking_dish(uuid).await;
        assert!(matches!(
            result,
            Err(DishBookingServiceError::InvalidOrder(_))
        ));
    }

    #[tokio::test]
    async fn test_booking_dish_update_failed() {
        let mut mock_repo = MockOrderRepository::new();
        let uuid = Uuid::new_v4();

        mock_repo
            .expect_find_dish_order_by_uuid()
            .returning(move |_| Ok(Some(make_dish_order(uuid, OrderStatus::Paid))));
        mock_repo
            .expect_update()
            .returning(|_| Err(RepositoryError::Db(anyhow!("DB error"))));

        let service = DishBookingServiceImpl::new(Arc::new(mock_repo));

        let result = service.booking_dish(uuid).await;
        assert!(result.is_err());
    }

    // -------- cancel_dish --------
    #[tokio::test]
    async fn test_cancel_dish_success() {
        let mut mock_repo = MockOrderRepository::new();
        let uuid = Uuid::new_v4();

        mock_repo
            .expect_find_dish_order_by_uuid()
            .returning(move |_| Ok(Some(make_dish_order(uuid, OrderStatus::Ongoing))));
        mock_repo.expect_update().returning(|_| Ok(()));

        let service = DishBookingServiceImpl::new(Arc::new(mock_repo));

        assert!(service.cancel_dish(uuid).await.is_ok());
    }

    #[tokio::test]
    async fn test_cancel_dish_invalid_status() {
        let mut mock_repo = MockOrderRepository::new();
        let uuid = Uuid::new_v4();

        mock_repo
            .expect_find_dish_order_by_uuid()
            .returning(move |_| Ok(Some(make_dish_order(uuid, OrderStatus::Paid))));

        let service = DishBookingServiceImpl::new(Arc::new(mock_repo));

        let result = service.cancel_dish(uuid).await;
        assert!(matches!(
            result,
            Err(DishBookingServiceError::InvalidOrderStatus(_, _))
        ));
    }

    #[tokio::test]
    async fn test_cancel_dish_order_not_found() {
        let mut mock_repo = MockOrderRepository::new();
        let uuid = Uuid::new_v4();

        mock_repo
            .expect_find_dish_order_by_uuid()
            .returning(|_| Ok(None));

        let service = DishBookingServiceImpl::new(Arc::new(mock_repo));

        let result = service.cancel_dish(uuid).await;
        assert!(matches!(
            result,
            Err(DishBookingServiceError::InvalidOrder(_))
        ));
    }

    #[tokio::test]
    async fn test_cancel_dish_update_failed() {
        let mut mock_repo = MockOrderRepository::new();
        let uuid = Uuid::new_v4();

        mock_repo
            .expect_find_dish_order_by_uuid()
            .returning(move |_| Ok(Some(make_dish_order(uuid, OrderStatus::Ongoing))));
        mock_repo
            .expect_update()
            .returning(|_| Err(RepositoryError::Db(anyhow!("DB error"))));

        let service = DishBookingServiceImpl::new(Arc::new(mock_repo));

        let result = service.cancel_dish(uuid).await;
        assert!(result.is_err());
    }

    // -------- booking_group --------
    #[tokio::test]
    async fn test_booking_group_all_success() {
        let mut mock_repo = MockOrderRepository::new();
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();

        // 两个订单都是 Paid
        mock_repo
            .expect_find_dish_order_by_uuid()
            .returning(move |_| Ok(Some(make_dish_order(Uuid::new_v4(), OrderStatus::Paid))));
        mock_repo.expect_update().returning(|_| Ok(()));

        let service = DishBookingServiceImpl::new(Arc::new(mock_repo));

        let result = service.booking_group(vec![uuid1, uuid2], true).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0); // 总是返回空列表
    }

    #[tokio::test]
    async fn test_booking_group_partial_fail() {
        let mut mock_repo = MockOrderRepository::new();
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();

        // 第一个 ok，第二个 fail
        let mut call_count = 0;
        mock_repo
            .expect_find_dish_order_by_uuid()
            .returning(move |_| {
                call_count += 1;
                if call_count == 1 {
                    Ok(Some(make_dish_order(uuid1, OrderStatus::Paid)))
                } else {
                    Ok(Some(make_dish_order(uuid2, OrderStatus::Cancelled)))
                }
            });
        mock_repo.expect_update().returning(|_| Ok(()));

        let service = DishBookingServiceImpl::new(Arc::new(mock_repo));

        let result = service.booking_group(vec![uuid1, uuid2], false).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0); // 即使有失败，也返回空
    }
}
