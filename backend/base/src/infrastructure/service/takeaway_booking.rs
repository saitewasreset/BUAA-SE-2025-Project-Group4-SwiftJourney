use crate::domain::model::order::{Order, OrderStatus, TakeawayOrder};
use crate::domain::repository::order::OrderRepository;
use crate::domain::service::takeaway_booking::{
    TakeawayBookingService, TakeawayBookingServiceError,
};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

pub struct TakeawayBookingServiceImpl<OR>
where
    OR: OrderRepository,
{
    order_repository: Arc<OR>,
}

impl<OR> TakeawayBookingServiceImpl<OR>
where
    OR: OrderRepository,
{
    pub fn new(order_repository: Arc<OR>) -> Self {
        Self { order_repository }
    }
}

#[async_trait]
impl<OR> TakeawayBookingService for TakeawayBookingServiceImpl<OR>
where
    OR: OrderRepository,
{
    #[instrument(skip(self))]
    async fn booking_takeaway(&self, order_uuid: Uuid) -> Result<(), TakeawayBookingServiceError> {
        let mut order = self
            .order_repository
            .find_takeaway_order_by_uuid(order_uuid)
            .await?
            .ok_or(TakeawayBookingServiceError::InvalidOrder(order_uuid))?;

        if order.order_status() != OrderStatus::Paid {
            return Err(TakeawayBookingServiceError::InvalidOrderStatus(
                order_uuid,
                order.order_status(),
            ));
        }

        // 外卖订单总是会成功

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
    async fn cancel_takeaway(&self, order_uuid: Uuid) -> Result<(), TakeawayBookingServiceError> {
        let mut order = self
            .order_repository
            .find_takeaway_order_by_uuid(order_uuid)
            .await?
            .ok_or(TakeawayBookingServiceError::InvalidOrder(order_uuid))?;

        if order.order_status() != OrderStatus::Ongoing {
            return Err(TakeawayBookingServiceError::InvalidOrderStatus(
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

    /// 对于合法的外卖订单，总是会成功，故本函数固定返回空的退款订单列表
    #[instrument(skip(self))]
    async fn booking_group(
        &self,
        order_uuid_list: Vec<Uuid>,
        _atomic: bool,
    ) -> Result<Vec<TakeawayOrder>, TakeawayBookingServiceError> {
        let mut success_booking_order_list = Vec::new();

        for order_uuid in order_uuid_list {
            if let Err(e) = self.booking_takeaway(order_uuid).await {
                error!("Failed to book takeaway: {:?}", e);
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
    use crate::domain::model::order::{BaseOrder, OrderTimeInfo, PaymentInfo};
    use crate::domain::model::transaction::Transaction;
    use crate::domain::repository::mock::order::MockOrderRepository;
    use crate::domain::service::takeaway_booking::TakeawayBookingService;
    use rust_decimal::Decimal;
    use std::sync::Arc;

    fn build_order(order_uuid: Uuid, status: OrderStatus) -> TakeawayOrder {
        let base_order = BaseOrder::new(
            Some(1u64.into()),
            order_uuid,
            status,
            OrderTimeInfo::new(Transaction::now(), Transaction::now(), Transaction::now()),
            Decimal::new(1000, 2),
            Decimal::ONE,
            PaymentInfo::new(Some(1u64.into()), None),
            1u64.into(),
        );
        TakeawayOrder::new(
            base_order,
            1u64.into(),
            1u64.into(),
            Decimal::new(1000, 2),
            Decimal::ONE,
        )
    }

    #[tokio::test]
    async fn test_booking_takeaway_success() {
        let order_uuid = Uuid::new_v4();
        let order = build_order(order_uuid, OrderStatus::Paid);

        let mut repo = MockOrderRepository::new();
        repo.expect_find_takeaway_order_by_uuid()
            .returning(move |_| Ok(Some(order.clone())));
        repo.expect_update().returning(|_| Ok(()));

        let service = TakeawayBookingServiceImpl::new(Arc::new(repo));
        let res = service.booking_takeaway(order_uuid).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_booking_takeaway_invalid_status() {
        let order_uuid = Uuid::new_v4();
        let order = build_order(order_uuid, OrderStatus::Cancelled);

        let mut repo = MockOrderRepository::new();
        repo.expect_find_takeaway_order_by_uuid()
            .returning(move |_| Ok(Some(order.clone())));

        let service = TakeawayBookingServiceImpl::new(Arc::new(repo));
        let res = service.booking_takeaway(order_uuid).await;

        assert!(matches!(
            res,
            Err(TakeawayBookingServiceError::InvalidOrderStatus(_, _))
        ));
    }

    #[tokio::test]
    async fn test_cancel_takeaway_success() {
        let order_uuid = Uuid::new_v4();
        let order = build_order(order_uuid, OrderStatus::Ongoing);

        let mut repo = MockOrderRepository::new();
        repo.expect_find_takeaway_order_by_uuid()
            .returning(move |_| Ok(Some(order.clone())));
        repo.expect_update().returning(|_| Ok(()));

        let service = TakeawayBookingServiceImpl::new(Arc::new(repo));
        let res = service.cancel_takeaway(order_uuid).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_cancel_takeaway_invalid_status() {
        let order_uuid = Uuid::new_v4();
        let order = build_order(order_uuid, OrderStatus::Paid);

        let mut repo = MockOrderRepository::new();
        repo.expect_find_takeaway_order_by_uuid()
            .returning(move |_| Ok(Some(order.clone())));

        let service = TakeawayBookingServiceImpl::new(Arc::new(repo));
        let res = service.cancel_takeaway(order_uuid).await;

        assert!(matches!(
            res,
            Err(TakeawayBookingServiceError::InvalidOrderStatus(_, _))
        ));
    }

    #[tokio::test]
    async fn test_booking_group_success() {
        let order_uuid1 = Uuid::new_v4();
        let order_uuid2 = Uuid::new_v4();
        let order1 = build_order(order_uuid1, OrderStatus::Paid);
        let order2 = build_order(order_uuid2, OrderStatus::Paid);

        let mut repo = MockOrderRepository::new();
        repo.expect_find_takeaway_order_by_uuid()
            .returning(move |uuid| {
                if uuid == order_uuid1 {
                    Ok(Some(order1.clone()))
                } else {
                    Ok(Some(order2.clone()))
                }
            });
        repo.expect_update().returning(|_| Ok(()));

        let service = TakeawayBookingServiceImpl::new(Arc::new(repo));
        let res = service
            .booking_group(vec![order_uuid1, order_uuid2], true)
            .await;

        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0); // 总是返回空列表
    }

    #[tokio::test]
    async fn test_booking_group_with_invalid_order() {
        let order_uuid1 = Uuid::new_v4();
        let order_uuid2 = Uuid::new_v4();
        let order1 = build_order(order_uuid1, OrderStatus::Cancelled); // 不合法
        let order2 = build_order(order_uuid2, OrderStatus::Paid);

        let mut repo = MockOrderRepository::new();
        repo.expect_find_takeaway_order_by_uuid()
            .returning(move |uuid| {
                if uuid == order_uuid1 {
                    Ok(Some(order1.clone()))
                } else {
                    Ok(Some(order2.clone()))
                }
            });

        let service = TakeawayBookingServiceImpl::new(Arc::new(repo));
        let res = service
            .booking_group(vec![order_uuid1, order_uuid2], true)
            .await;

        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0); // 即使失败也固定返回空列表
    }
}
