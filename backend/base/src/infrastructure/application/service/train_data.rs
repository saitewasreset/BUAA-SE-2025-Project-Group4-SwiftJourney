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

use crate::application::commands::train_data::{
    LoadCityCommand, LoadDishTakeawayCommand, LoadStationCommand, LoadTrainNumberCommand,
    LoadTrainTypeCommand,
};
use crate::application::service::train_data::TrainDataService;
use crate::application::{ApplicationError, GeneralError, ModeError};
use crate::domain::repository::city::CityRepository;
use crate::domain::repository::dish::DishRepository;
use crate::domain::repository::route::RouteRepository;
use crate::domain::repository::station::StationRepository;
use crate::domain::repository::takeaway::TakeawayShopRepository;
use crate::domain::repository::train::TrainRepository;
use crate::domain::service::object_storage::ObjectStorageService;
use async_trait::async_trait;
use shared::data::{DishData, TakeawayData};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, instrument, warn};

/// 火车数据加载服务实现
///
/// 实现了`TrainDataService`接口，协调城市仓储、车站仓储、列车仓储和路线仓储
/// 来完成火车数据的加载操作。
///
/// # 类型参数
/// - `C`: 城市仓储类型，需实现`CityRepository` trait
/// - `S`: 火车站仓储类型，需实现`StationRepository` trait
/// - `T`: 列车仓储类型，需实现`TrainRepository` trait
/// - `R`: 路线仓储类型，需实现`RouteRepository` trait
///
/// # 字段
/// - `debug`: 是否启用调试模式
/// - `city_repository`: 城市仓储
/// - `station_repository`: 火车站仓储
/// - `train_repository`: 列车仓储
/// - `route_repository`: 路线仓储
pub struct TrainDataServiceImpl<C, S, T, R, D, TS, OSS>
where
    C: CityRepository,
    S: StationRepository,
    T: TrainRepository,
    R: RouteRepository,
    D: DishRepository,
    TS: TakeawayShopRepository,
    OSS: ObjectStorageService,
{
    debug: bool,
    data_path: PathBuf,
    city_repository: Arc<C>,
    station_repository: Arc<S>,
    train_repository: Arc<T>,
    route_repository: Arc<R>,
    dish_repository: Arc<D>,
    takeaway_shop_repository: Arc<TS>,
    object_storage_service: Arc<OSS>,
}

impl<C, S, T, R, D, TS, OSS> TrainDataServiceImpl<C, S, T, R, D, TS, OSS>
where
    C: CityRepository,
    S: StationRepository,
    T: TrainRepository,
    R: RouteRepository,
    D: DishRepository,
    TS: TakeawayShopRepository,
    OSS: ObjectStorageService,
{
    /// 创建新的火车数据加载服务实例
    ///
    /// # Arguments
    /// * `debug` - 是否启用调试模式
    /// * `city_repository` - 城市仓储
    /// * `station_repository` - 火车站仓储
    /// * `train_repository` - 列车仓储
    /// * `route_repository` - 路线仓储
    ///
    /// # Returns
    /// 返回新的`TrainDataServiceImpl`实例
    pub fn new(
        debug: bool,
        data_path: PathBuf,
        city_repository: Arc<C>,
        station_repository: Arc<S>,
        train_repository: Arc<T>,
        route_repository: Arc<R>,
        dish_repository: Arc<D>,
        takeaway_shop_repository: Arc<TS>,
        object_storage_service: Arc<OSS>,
    ) -> Self {
        Self {
            debug,
            data_path,
            city_repository,
            station_repository,
            train_repository,
            route_repository,
            dish_repository,
            takeaway_shop_repository,
            object_storage_service,
        }
    }

    /// 检查是否启用调试模式
    ///
    /// # Returns
    /// * `Ok(())` - 调试模式已启用
    /// * `Err(Box<dyn ApplicationError>)` - 调试模式未启用
    ///
    /// # Errors
    /// * `ModeError` - 调试模式未启用
    #[instrument(skip_all)]
    pub fn check_debug_mode(&self) -> Result<(), Box<dyn ApplicationError>> {
        if self.debug {
            Ok(())
        } else {
            warn!("Debug mode is not enabled");
            Err(Box::new(ModeError))
        }
    }
}

#[async_trait]
impl<C, S, T, R, D, TS, OSS> TrainDataService for TrainDataServiceImpl<C, S, T, R, D, TS, OSS>
where
    C: CityRepository,
    S: StationRepository,
    T: TrainRepository,
    R: RouteRepository,
    D: DishRepository,
    TS: TakeawayShopRepository,
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

    /// 加载列车类型数据
    ///
    /// # Arguments
    /// * `command` - 加载列车类型命令
    ///
    /// # Returns
    /// * `Ok(())` - 加载成功
    /// * `Err(Box<dyn ApplicationError>)` - 加载失败及原因
    ///
    /// # Errors
    /// * `ModeError` - 调试模式未启用
    /// * `GeneralError::InternalServerError` - 底层基础设施错误
    #[instrument(skip_all)]
    async fn load_train_type(
        &self,
        command: LoadTrainTypeCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        self.check_debug_mode()?;

        self.train_repository
            .save_raw_train_type(command)
            .await
            .map_err(|e| {
                error!("Error saving train type: {:?}", e);
                GeneralError::InternalServerError
            })?;

        Ok(())
    }

    /// 加载列车编号数据
    ///
    /// # Arguments
    /// * `command` - 加载列车编号命令
    ///
    /// # Returns
    /// * `Ok(())` - 加载成功
    /// * `Err(Box<dyn ApplicationError>)` - 加载失败及原因
    ///
    /// # Errors
    /// * `ModeError` - 调试模式未启用
    /// * `GeneralError::InternalServerError` - 底层基础设施错误
    #[instrument(skip_all)]
    async fn load_train_number(
        &self,
        command: LoadTrainNumberCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        self.check_debug_mode()?;

        self.train_repository
            .save_raw_train_number(command, Arc::clone(&self.route_repository))
            .await
            .map_err(|e| {
                error!("Error saving train number: {:?}", e);
                GeneralError::InternalServerError
            })?;

        Ok(())
    }

    async fn load_dish_takeaway(
        &self,
        command: LoadDishTakeawayCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let mut dish_data: DishData = HashMap::new();
        let mut takeaway_data: TakeawayData = HashMap::new();

        let mut station_name_shop_name_set: HashSet<(String, String)> = HashSet::new();

        for item in command {
            dish_data.insert(item.train_number.clone(), item.dish_info);

            for (station_name, inner_map) in item.takeaway_info {
                for (shop_name, takeaway_list) in inner_map {
                    let id = (station_name.clone(), shop_name.clone());

                    if !station_name_shop_name_set.contains(&id) {
                        takeaway_data
                            .entry(station_name.clone())
                            .or_default()
                            .insert(shop_name.clone(), takeaway_list);

                        station_name_shop_name_set.insert(id);
                    }
                }
            }
        }

        self.dish_repository
            .save_raw_dish(
                dish_data,
                &self.data_path,
                Arc::clone(&self.train_repository),
                Arc::clone(&self.object_storage_service),
            )
            .await
            .inspect_err(|e| {
                error!("Error saving dish data: {:?}", e);
            })
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        self.takeaway_shop_repository
            .save_raw_takeaway(
                takeaway_data,
                &self.data_path,
                Arc::clone(&self.station_repository),
                Arc::clone(&self.object_storage_service),
            )
            .await
            .inspect_err(|e| {
                error!("Error saving takeaway data: {:?}", e);
            })
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        Ok(())
    }
}
