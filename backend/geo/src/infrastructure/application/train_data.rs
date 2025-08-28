//! 火车数据加载应用服务实现模块
//!
//! 本模块提供了`TrainDataService`接口的具体实现。
//!
//! ## 主要组件
//! - `TrainDataServiceImpl`: 火车数据加载服务的具体实现
//! - 依赖四个核心组件:
//!   - 城市仓储(`CityRepository`)
//!   - 火车站仓储(`StationRepository`)
//!   - 列车仓储(`TrainRepository`)
//!   - 路线仓储(`RouteRepository`)

use crate::application::{ApplicationError, GeneralError, ModeError};
use crate::domain::repository::city::CityRepository;
use crate::domain::repository::station::StationRepository;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use shared::data::{DishData, TakeawayData};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, instrument, warn};

#[async_trait]
impl<C, S, OSS> TrainDataService for TrainDataServiceImpl<C, S, OSS>
where
    C: CityRepository,
    S: StationRepository,
    OSS: ObjectStorageService,
{
    /// 检查是否启用调试模式
    ///
    /// # Returns
    /// 是否启用调试模式
    fn is_debug_mode(&self) -> bool {
        self.debug
    }

    /// 加载城市数据
    ///
    /// # Arguments
    /// * `command` - 加载城市命令
    ///
    /// # Returns
    /// * `Ok(())` - 加载成功
    /// * `Err(Box<dyn ApplicationError>)` - 加载失败及原因
    ///
    /// # Errors
    /// * `ModeError` - 调试模式未启用
    /// * `GeneralError::InternalServerError` - 底层基础设施错误
    #[instrument(skip_all)]
    async fn load_city(&self, command: LoadCityCommand) -> Result<(), Box<dyn ApplicationError>> {
        self.check_debug_mode()?;
        self.city_repository.save_raw(command).await.map_err(|e| {
            error!("Error saving city: {:?}", e);
            GeneralError::InternalServerError
        })?;

        Ok(())
    }

    /// 加载车站数据
    ///
    /// # Arguments
    /// * `command` - 加载车站命令
    ///
    /// # Returns
    /// * `Ok(())` - 加载成功
    /// * `Err(Box<dyn ApplicationError>)` - 加载失败及原因
    ///
    /// # Errors
    /// * `ModeError` - 调试模式未启用
    /// * `GeneralError::InternalServerError` - 底层基础设施错误
    #[instrument(skip_all)]
    async fn load_station(
        &self,
        command: LoadStationCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        self.check_debug_mode()?;

        self.station_repository
            .save_raw(command)
            .await
            .map_err(|e| {
                error!("Error saving station: {:?}", e);
                GeneralError::InternalServerError
            })?;

        Ok(())
    }
}
