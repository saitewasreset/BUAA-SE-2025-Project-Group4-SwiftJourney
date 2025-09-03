//! 列车类型配置服务基础设施实现模块
//!
//! 提供`TrainTypeConfigurationService` trait的具体实现，将列车类型领域逻辑与底层仓储连接起来。
//! 本实现是泛型的，可以适配不同的列车仓储实现。
//!
//! # 主要功能
//! - 列车类型和车次编号的验证
//! - 座位类型配置管理
//! - 列车车次的增删改查操作
use crate::domain::model::route::RouteId;
use crate::domain::model::train::{SeatType, SeatTypeName, Train, TrainId, TrainNumber, TrainType};
use crate::domain::model::train_schedule::{SeatId, SeatLocationInfo};
use crate::domain::repository::train::TrainRepository;
use crate::domain::service::train_type::{
    TrainTypeConfigurationService, TrainTypeConfigurationServiceError,
};
use crate::domain::Identifiable;
use crate::{Unverified, Verified};
use async_trait::async_trait;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

/// 列车类型配置服务具体实现
///
/// 泛型参数：
/// - `R`: 列车仓储实现
///
/// # 类型约束
/// - `R`必须实现`TrainRepository` trait
pub struct TrainTypeConfigurationServiceImpl<R>
where
    R: TrainRepository,
{
    /// 列车仓储实例
    train_repository: Arc<R>,
}

impl<R> TrainTypeConfigurationServiceImpl<R>
where
    R: TrainRepository,
{
    /// 创建新的列车类型配置服务实例
    ///
    /// # Arguments
    /// * `train_repository` - 列车仓储实现
    pub fn new(train_repository: Arc<R>) -> Self {
        Self { train_repository }
    }
}

#[async_trait]
impl<R> TrainTypeConfigurationService for TrainTypeConfigurationServiceImpl<R>
where
    R: TrainRepository,
{
    /// 验证座位类型名称实现
    ///
    /// # Arguments
    /// * `train_id` - 车次ID
    /// * `seat_type_name` - 未验证的座位类型名称
    ///
    /// # Returns
    /// * `Ok(SeatTypeName<Verified>)` - 验证成功的座位类型名称
    /// * `Err(TrainTypeConfigurationServiceError)` - 验证失败及原因
    ///
    /// # Errors
    /// * `InvalidSeatType` - 座位类型名称无效
    /// * `InfrastructureError` - 仓储访问错误
    async fn verify_seat_type_name(
        &self,
        train_id: TrainId,
        seat_type_name: SeatTypeName<Unverified>,
    ) -> Result<SeatTypeName<Verified>, TrainTypeConfigurationServiceError> {
        let verified_seat_type = self
            .train_repository
            .get_verified_seat_type(train_id)
            .await?;

        if verified_seat_type.contains(seat_type_name.deref()) {
            Ok(SeatTypeName::from_unchecked(seat_type_name.to_string()))
        } else {
            Err(TrainTypeConfigurationServiceError::InvalidSeatType(
                train_id.into(),
                seat_type_name.to_string(),
            ))
        }
    }

    /// 验证车次编号实现
    ///
    /// # Arguments
    /// * `train_number` - 未验证的车次编号
    ///
    /// # Returns
    /// * `Ok(TrainNumber<Verified>)` - 验证成功的车次编号
    /// * `Err(TrainTypeConfigurationServiceError)` - 验证失败及原因
    ///
    /// # Errors
    /// * `InvalidTrainNumber` - 车次编号无效
    /// * `InfrastructureError` - 仓储访问错误
    async fn verify_train_number(
        &self,
        train_number: TrainNumber<Unverified>,
    ) -> Result<TrainNumber<Verified>, TrainTypeConfigurationServiceError> {
        let verified_train_number = self.train_repository.get_verified_train_number().await?;

        if verified_train_number.contains(train_number.deref()) {
            Ok(TrainNumber::from_unchecked(train_number.to_string()))
        } else {
            Err(TrainTypeConfigurationServiceError::InvalidTrainNumber(
                train_number.to_string(),
            ))
        }
    }

    /// 验证列车类型实现
    ///
    /// # Arguments
    /// * `train_type` - 未验证的列车类型
    ///
    /// # Returns
    /// * `Ok(TrainType<Verified>)` - 验证成功的列车类型
    /// * `Err(TrainTypeConfigurationServiceError)` - 验证失败及原因
    ///
    /// # Errors
    /// * `InvalidTrainType` - 列车类型无效
    /// * `InfrastructureError` - 仓储访问错误
    async fn verify_train_type(
        &self,
        train_type: TrainType<Unverified>,
    ) -> Result<TrainType<Verified>, TrainTypeConfigurationServiceError> {
        let verified_train_type = self.train_repository.get_verified_train_type().await?;

        if verified_train_type.contains(train_type.deref()) {
            Ok(TrainType::from_unchecked(train_type.to_string()))
        } else {
            Err(TrainTypeConfigurationServiceError::InvalidTrainType(
                train_type.to_string(),
            ))
        }
    }

    /// 获取座位ID映射实现
    ///
    /// # Arguments
    /// * `train_id` - 车次ID
    ///
    /// # Returns
    /// * `Ok(HashMap<SeatTypeName<Verified>, Vec<SeatId>>)` - 座位类型到座位ID列表的映射
    /// * `Err(TrainTypeConfigurationServiceError)` - 获取失败及原因
    ///
    /// # Errors
    /// * `NoSuchTrainId` - 车次不存在
    /// * `InfrastructureError` - 仓储访问错误
    async fn get_seat_id_map(
        &self,
        train_id: TrainId,
    ) -> Result<
        HashMap<SeatTypeName<Verified>, Vec<(SeatId, SeatLocationInfo)>>,
        TrainTypeConfigurationServiceError,
    > {
        if self.train_repository.find(train_id).await?.is_some() {
            let result = self.train_repository.get_seat_id_map(train_id).await?;

            Ok(result)
        } else {
            Err(TrainTypeConfigurationServiceError::NoSuchTrainId(
                train_id.into(),
            ))
        }
    }

    /// 获取所有列车实现
    ///
    /// # Returns
    /// * `Ok(Vec<Train>)` - 所有列车的列表
    /// * `Err(TrainTypeConfigurationServiceError)` - 获取失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 仓储访问错误
    async fn get_trains(&self) -> Result<Vec<Train>, TrainTypeConfigurationServiceError> {
        let result = self.train_repository.get_trains().await?;

        Ok(result)
    }

    /// 根据车次编号获取列车实现
    ///
    /// # Arguments
    /// * `train_number` - 已验证的车次编号
    ///
    /// # Returns
    /// * `Ok(Train)` - 匹配的列车
    /// * `Err(TrainTypeConfigurationServiceError)` - 获取失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 仓储访问错误
    async fn get_train_by_number(
        &self,
        train_number: TrainNumber<Verified>,
    ) -> Result<Train, TrainTypeConfigurationServiceError> {
        let result = self
            .train_repository
            .find_by_train_number(train_number)
            .await?;

        Ok(result)
    }

    /// 添加新列车类型实现
    ///
    /// # Arguments
    /// * `train_number` - 已验证的车次编号
    /// * `train_type` - 已验证的列车类型
    /// * `seat_configuration` - 座位配置列表
    /// * `default_route_id` - 默认路线ID
    ///
    /// # Returns
    /// * `Ok(TrainId)` - 新添加的列车ID
    /// * `Err(TrainTypeConfigurationServiceError)` - 添加失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 仓储访问错误
    async fn add_train_type(
        &self,
        train_number: TrainNumber<Verified>,
        train_type: TrainType<Verified>,
        seat_configuration: Vec<SeatType>,
        default_route_id: RouteId,
        default_origin_departure_time: i32,
    ) -> Result<TrainId, TrainTypeConfigurationServiceError> {
        let seat_map = seat_configuration
            .into_iter()
            .map(|x| (x.name().to_string(), x))
            .collect::<HashMap<_, _>>();

        let mut train = Train::new(
            None,
            train_number,
            train_type,
            seat_map,
            default_route_id,
            default_origin_departure_time,
        );

        self.train_repository.save(&mut train).await?;

        Ok(train.get_id().expect("saved train should have id"))
    }

    /// 修改列车类型实现
    ///
    /// # Arguments
    /// * `train_id` - 车次ID
    /// * `train_number` - 已验证的车次编号
    /// * `train_type` - 已验证的列车类型
    /// * `seat_configuration` - 座位配置列表
    /// * `default_route_id` - 默认路线ID
    ///
    /// # Returns
    /// * `Ok(())` - 修改成功
    /// * `Err(TrainTypeConfigurationServiceError)` - 修改失败及原因
    ///
    /// # Errors
    /// * `NoSuchTrainId` - 车次不存在
    /// * `InfrastructureError` - 仓储访问错误
    async fn modify_train_type(
        &self,
        train_id: TrainId,
        train_number: TrainNumber<Verified>,
        train_type: TrainType<Verified>,
        seat_configuration: Vec<SeatType>,
        default_route_id: RouteId,
        default_origin_departure_time: i32,
    ) -> Result<(), TrainTypeConfigurationServiceError> {
        if self.train_repository.find(train_id).await?.is_some() {
            let seat_map = seat_configuration
                .into_iter()
                .map(|x| (x.name().to_string(), x))
                .collect::<HashMap<_, _>>();

            let mut train = Train::new(
                None,
                train_number,
                train_type,
                seat_map,
                default_route_id,
                default_origin_departure_time,
            );

            self.train_repository.save(&mut train).await?;

            Ok(())
        } else {
            Err(TrainTypeConfigurationServiceError::NoSuchTrainId(
                train_id.into(),
            ))
        }
    }

    /// 删除列车类型实现
    ///
    /// # Arguments
    /// * `train` - 要删除的列车实体
    ///
    /// # Returns
    /// * `Ok(())` - 删除成功
    /// * `Err(TrainTypeConfigurationServiceError)` - 删除失败及原因
    ///
    /// # Errors
    /// * `InfrastructureError` - 仓储访问错误
    async fn remove_train_type(
        &self,
        train: Train,
    ) -> Result<(), TrainTypeConfigurationServiceError> {
        self.train_repository.remove(train).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::route::RouteId;
    use crate::domain::model::train::{
        SeatType, SeatTypeName, Train, TrainId, TrainNumber, TrainType,
    };
    use crate::domain::model::train_schedule::SeatLocationInfo;
    use crate::domain::repository::mock::train::MockTrainRepository;
    use crate::domain::service::train_type::TrainTypeConfigurationServiceError;
    use rust_decimal::Decimal;
    use std::collections::{HashMap, HashSet};

    fn sample_train() -> Train {
        let seat_type = SeatType::new(
            Some(1u64.into()),
            SeatTypeName::from_unchecked("二等座".to_string()),
            100,
            Decimal::new(5000, 2),
        );
        let seat_map = vec![(seat_type.name().to_string(), seat_type)]
            .into_iter()
            .collect::<HashMap<_, _>>();
        Train::new(
            Some(1u64.into()),
            TrainNumber::from_unchecked("G123".to_string()),
            TrainType::from_unchecked("高铁".to_string()),
            seat_map,
            RouteId::from(1u64),
            0,
        )
    }

    #[tokio::test]
    async fn test_verify_seat_type_name_success() {
        let mut repo = MockTrainRepository::new();
        repo.expect_get_verified_seat_type()
            .returning(|_| Ok(HashSet::from(["二等座".to_string()])));
        let service = TrainTypeConfigurationServiceImpl::new(Arc::new(repo));

        let res = service
            .verify_seat_type_name(
                TrainId::from(1u64),
                SeatTypeName::from("二等座".to_string()),
            )
            .await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_verify_seat_type_name_fail() {
        let mut repo = MockTrainRepository::new();
        repo.expect_get_verified_seat_type()
            .returning(|_| Ok(HashSet::from(["一等座".to_string()])));
        let service = TrainTypeConfigurationServiceImpl::new(Arc::new(repo));

        let res = service
            .verify_seat_type_name(
                TrainId::from(1u64),
                SeatTypeName::from("二等座".to_string()),
            )
            .await;

        assert!(matches!(
            res,
            Err(TrainTypeConfigurationServiceError::InvalidSeatType(_, _))
        ));
    }

    #[tokio::test]
    async fn test_verify_train_number_success() {
        let mut repo = MockTrainRepository::new();
        repo.expect_get_verified_train_number()
            .returning(|| Ok(HashSet::from(["G123".to_string()])));
        let service = TrainTypeConfigurationServiceImpl::new(Arc::new(repo));

        let res = service
            .verify_train_number(TrainNumber::from("G123".to_string()))
            .await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_verify_train_number_fail() {
        let mut repo = MockTrainRepository::new();
        repo.expect_get_verified_train_number()
            .returning(|| Ok(HashSet::from(["D456".to_string()])));
        let service = TrainTypeConfigurationServiceImpl::new(Arc::new(repo));

        let res = service
            .verify_train_number(TrainNumber::from("G123".to_string()))
            .await;

        assert!(matches!(
            res,
            Err(TrainTypeConfigurationServiceError::InvalidTrainNumber(_))
        ));
    }

    #[tokio::test]
    async fn test_verify_train_type_success() {
        let mut repo = MockTrainRepository::new();
        repo.expect_get_verified_train_type()
            .returning(|| Ok(HashSet::from(["高铁".to_string()])));
        let service = TrainTypeConfigurationServiceImpl::new(Arc::new(repo));

        let res = service
            .verify_train_type(TrainType::from("高铁".to_string()))
            .await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_verify_train_type_fail() {
        let mut repo = MockTrainRepository::new();
        repo.expect_get_verified_train_type()
            .returning(|| Ok(HashSet::from(["动车".to_string()])));
        let service = TrainTypeConfigurationServiceImpl::new(Arc::new(repo));

        let res = service
            .verify_train_type(TrainType::from("高铁".to_string()))
            .await;

        assert!(matches!(
            res,
            Err(TrainTypeConfigurationServiceError::InvalidTrainType(_))
        ));
    }

    #[tokio::test]
    async fn test_get_seat_id_map_success() {
        let mut repo = MockTrainRepository::new();
        repo.expect_find().returning(|_| Ok(Some(sample_train())));
        repo.expect_get_seat_id_map().returning(|_| {
            Ok(HashMap::from([(
                SeatTypeName::from_unchecked("二等座".into()),
                vec![(
                    1u64.into(),
                    SeatLocationInfo {
                        carriage: 1,
                        row: 1,
                        location: 'A',
                    },
                )],
            )]))
        });
        let service = TrainTypeConfigurationServiceImpl::new(Arc::new(repo));

        let res = service.get_seat_id_map(TrainId::from(1u64)).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_get_seat_id_map_fail_no_train() {
        let mut repo = MockTrainRepository::new();
        repo.expect_find().returning(|_| Ok(None));
        let service = TrainTypeConfigurationServiceImpl::new(Arc::new(repo));

        let res = service.get_seat_id_map(TrainId::from(1u64)).await;
        assert!(matches!(
            res,
            Err(TrainTypeConfigurationServiceError::NoSuchTrainId(_))
        ));
    }

    #[tokio::test]
    async fn test_get_trains_success() {
        let mut repo = MockTrainRepository::new();
        repo.expect_get_trains()
            .returning(|| Ok(vec![sample_train()]));
        let service = TrainTypeConfigurationServiceImpl::new(Arc::new(repo));

        let res = service.get_trains().await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_get_train_by_number_success() {
        let mut repo = MockTrainRepository::new();
        repo.expect_find_by_train_number()
            .returning(|_| Ok(sample_train()));
        let service = TrainTypeConfigurationServiceImpl::new(Arc::new(repo));

        let res = service
            .get_train_by_number(TrainNumber::from_unchecked("G123".into()))
            .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_add_train_type_success() {
        let mut repo = MockTrainRepository::new();
        repo.expect_save().returning(|train| {
            train.set_id(TrainId::from(1u64));
            Ok(TrainId::from(1u64))
        });
        let service = TrainTypeConfigurationServiceImpl::new(Arc::new(repo));

        let seat_type = SeatType::new(
            Some(1u64.into()),
            SeatTypeName::from_unchecked("二等座".to_string()),
            100,
            Decimal::new(5000, 2),
        );

        let res = service
            .add_train_type(
                TrainNumber::from_unchecked("G123".into()),
                TrainType::from_unchecked("高铁".into()),
                vec![seat_type],
                RouteId::from(1u64),
                0,
            )
            .await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_modify_train_type_success() {
        let mut repo = MockTrainRepository::new();
        repo.expect_find().returning(|_| Ok(Some(sample_train())));
        repo.expect_save().returning(|_| Ok(TrainId::from(1u64)));
        let service = TrainTypeConfigurationServiceImpl::new(Arc::new(repo));

        let seat_type = SeatType::new(
            Some(1u64.into()),
            SeatTypeName::from_unchecked("二等座".to_string()),
            100,
            Decimal::new(5000, 2),
        );

        let res = service
            .modify_train_type(
                TrainId::from(1u64),
                TrainNumber::from_unchecked("G123".into()),
                TrainType::from_unchecked("高铁".into()),
                vec![seat_type],
                RouteId::from(1u64),
                0,
            )
            .await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_modify_train_type_fail_no_train() {
        let mut repo = MockTrainRepository::new();
        repo.expect_find().returning(|_| Ok(None));
        let service = TrainTypeConfigurationServiceImpl::new(Arc::new(repo));

        let res = service
            .modify_train_type(
                TrainId::from(1u64),
                TrainNumber::from_unchecked("G123".into()),
                TrainType::from_unchecked("高铁".into()),
                vec![],
                RouteId::from(1u64),
                0,
            )
            .await;

        assert!(matches!(
            res,
            Err(TrainTypeConfigurationServiceError::NoSuchTrainId(_))
        ));
    }

    #[tokio::test]
    async fn test_remove_train_type_success() {
        let mut repo = MockTrainRepository::new();
        repo.expect_remove().returning(|_| Ok(()));
        let service = TrainTypeConfigurationServiceImpl::new(Arc::new(repo));

        let res = service.remove_train_type(sample_train()).await;
        assert!(res.is_ok());
    }
}
