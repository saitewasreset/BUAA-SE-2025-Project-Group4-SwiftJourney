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
use crate::domain::repository::route::RouteRepository;
use crate::domain::repository::station::StationRepository;
use crate::domain::repository::takeaway::TakeawayShopRepository;
use crate::domain::repository::train::TrainRepository;
use crate::domain::service::object_storage::ObjectStorageService;
use crate::infrastructure::repository::dish::save_raw_dish;
use crate::infrastructure::repository::takeaway::save_raw_takeaway;
use crate::infrastructure::repository::train::{save_raw_train_number, save_raw_train_type};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
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
pub struct TrainDataServiceImpl<C, S, T, R, TS, OSS>
where
    C: CityRepository,
    S: StationRepository,
    T: TrainRepository,
    R: RouteRepository,
    TS: TakeawayShopRepository,
    OSS: ObjectStorageService,
{
    debug: bool,
    data_path: PathBuf,
    city_repository: Arc<C>,
    station_repository: Arc<S>,
    train_repository: Arc<T>,
    route_repository: Arc<R>,
    takeaway_shop_repository: Arc<TS>,
    object_storage_service: Arc<OSS>,
}

impl<C, S, T, R, TS, OSS> TrainDataServiceImpl<C, S, T, R, TS, OSS>
where
    C: CityRepository,
    S: StationRepository,
    T: TrainRepository,
    R: RouteRepository,
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        debug: bool,
        data_path: PathBuf,
        city_repository: Arc<C>,
        station_repository: Arc<S>,
        train_repository: Arc<T>,
        route_repository: Arc<R>,
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
impl<C, S, T, R, TS, OSS> TrainDataService for TrainDataServiceImpl<C, S, T, R, TS, OSS>
where
    C: CityRepository,
    S: StationRepository,
    T: TrainRepository,
    R: RouteRepository,
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
        db: &DatabaseConnection,
    ) -> Result<(), Box<dyn ApplicationError>> {
        self.check_debug_mode()?;

        save_raw_train_type(db, command).await.map_err(|e| {
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
        db: &DatabaseConnection,
    ) -> Result<(), Box<dyn ApplicationError>> {
        self.check_debug_mode()?;

        save_raw_train_number(command, Arc::clone(&self.route_repository), db)
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
        db: &DatabaseConnection,
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

        save_raw_dish(
            dish_data,
            &self.data_path,
            Arc::clone(&self.train_repository),
            Arc::clone(&self.object_storage_service),
            db,
        )
        .await
        .inspect_err(|e| {
            error!("Error saving dish data: {:?}", e);
        })
        .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        save_raw_takeaway(
            takeaway_data,
            &self.data_path,
            Arc::clone(&self.takeaway_shop_repository),
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


#[cfg(test)]
mod tests {
    use super::*;
    // 引入 mock
    use crate::domain::repository::mock::{
        city::mock_city_repository,
        route::mock_route_repository,
        station::mock_station_repository,
        takeaway::mock_takeaway_shop_repo,
        train::mock_train_repo,
    };
    use crate::domain::service::mock::object_storage::mock_object_storage_service;
    use crate::domain::RepositoryError;
    use anyhow::anyhow;
    use sea_orm::Database;
    use shared::data::{StationDataItem, TrainNumberInfoItem, TrainTypeInfoItem};
    use tempfile::tempdir;
    use tokio;

    fn setup_service(debug: bool) -> TrainDataServiceImpl<
        impl CityRepository,
        impl StationRepository,
        impl TrainRepository,
        impl RouteRepository,
        impl TakeawayShopRepository,
        impl ObjectStorageService,
    > {
        let temp = tempdir().unwrap();
        TrainDataServiceImpl::new(
            debug,
            temp.path().to_path_buf(),
            Arc::new(mock_city_repository()),
            Arc::new(mock_station_repository()),
            Arc::new(mock_train_repo()),
            Arc::new(mock_route_repository()),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_object_storage_service()),
        )
    }

    #[tokio::test]
    async fn test_load_city_success() {
        // 创建 Mock 并设置 expectation
        let mut mock_city_repo = crate::domain::repository::mock::city::MockCityRepository::new();

        // 对 save_raw 的调用设置期望
        mock_city_repo
            .expect_save_raw()
            .withf(|city_data| city_data.get("Beijing").map(|v| v == "BJP").unwrap_or(false))
            .returning(|_city_data| Ok(()));

        let service = TrainDataServiceImpl::new(
            true,
            std::path::PathBuf::from("/tmp"),
            Arc::new(mock_city_repo),
            Arc::new(mock_station_repository()),
            Arc::new(mock_train_repo()),
            Arc::new(mock_route_repository()),
            Arc::new(mock_takeaway_shop_repo()),
            Arc::new(mock_object_storage_service()),
        );

        let cmd: HashMap<String, String> = [("Beijing".to_string(), "BJP".to_string())]
            .into_iter()
            .collect();

        let result = service.load_city(cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_city_fail_debug_off() {
        let service = setup_service(false);
        let cmd = [("Beijing".to_string(), "BJP".to_string())]
            .into_iter()
            .collect::<HashMap<_, _>>();
        let result = service.load_city(cmd).await;
        assert!(result.is_err()); // 因为 debug = false
    }

    #[tokio::test]
    async fn test_load_station_success() {
        // 创建 mock
        let mut station_repo = mock_station_repository();

        // 设置 expectation：save_raw 被调用一次，返回 Ok(())
        station_repo
            .expect_save_raw()
            .times(1)
            .returning(|_cmd| Ok::<(), RepositoryError>(()));


        let service = TrainDataServiceImpl::new(
            true, // debug = true
            tempdir().unwrap().path().to_path_buf(),
            Arc::new(mock_city_repository()), // city repo 不会被调用
            Arc::new(station_repo),
            Arc::new(mock_train_repo()),      // train repo
            Arc::new(mock_route_repository()),// route repo
            Arc::new(mock_takeaway_shop_repo()), // takeaway shop
            Arc::new(mock_object_storage_service()), // object storage
        );

        let cmd = vec![StationDataItem {
            name: "上海虹桥".to_string(),
            city: "上海".to_string(),
        }];

        let result = service.load_station(cmd).await;
        assert!(result.is_ok());
    }


    #[tokio::test]
    async fn test_load_station_fail_debug_off() {
        let service = setup_service(false);
        let cmd = vec![StationDataItem {
            name: "上海虹桥".to_string(),
            city: "上海".to_string(),
        }];
        let result = service.load_station(cmd).await;
        assert!(result.is_err());
    }

    // #[tokio::test]
    // async fn test_load_train_type_success() {
    //     // Mock TrainRepository
    //     let mut train_repo = mock_train_repo();
    //     train_repo.expect_get_trains().returning(move || Ok(vec![]));
    //     train_repo.expect_find_by_train_type().returning(|_cmd| Ok(vec![]));
    //     train_repo.expect_get_verified_train_type().returning(move || Ok(HashSet::new()));
    //     train_repo.expect_get_verified_seat_type().returning(|_cmd| Ok(HashSet::new()));
    //     train_repo.expect_get_seat_id_map().returning(|_cmd| Ok(HashMap::new()));
    //     train_repo.expect_save().returning(|_train| Ok(1u64.into()));
    //     train_repo.expect_remove().returning(|_train| Ok(()));
    //     train_repo.expect_find().returning(|_train| {Ok(Some(crate::domain::model::train::Train::new(
    //         Some(1u64.into()),
    //         TrainNumber::from_unchecked("G111".to_string()),
    //         TrainType::from_unchecked("G".to_string()),
    //         HashMap::new(),
    //         1u64.into(),
    //         0,
    //     )))});
    //
    //     // Mock StationRepository
    //     let mut station_repo = mock_station_repository();
    //     station_repo.expect_load().returning(|| Ok(vec![]));
    //
    //     // Mock CityRepository
    //     let mut city_repo = mock_city_repository();
    //     city_repo.expect_load().returning(|| Ok(vec![]));
    //
    //     // Mock TakeawayShopRepository 并设置 save_many_atomic 返回 Ok
    //     let takeaway_repo_mock = mock_takeaway_shop_repo();
    //
    //     let takeaway_repo = Arc::new(takeaway_repo_mock);
    //
    //     // 其他 repo/service
    //     let route_repo = Arc::new(mock_route_repository());
    //     let object_storage = Arc::new(mock_object_storage_service());
    //
    //     // 构造服务
    //     let service = TrainDataServiceImpl::new(
    //         true,
    //         tempdir().unwrap().path().to_path_buf(),
    //         Arc::new(city_repo),
    //         Arc::new(station_repo),
    //         Arc::new(train_repo),
    //         route_repo,
    //         takeaway_repo,
    //         object_storage,
    //     );
    //
    //     // 构造命令数据
    //     let cmd = vec![TrainTypeInfoItem {
    //         id: "1".to_string(),
    //         name: "日升".to_string(),
    //         seat: Default::default(),
    //     }];
    //
    //     let db: DatabaseConnection = Database::connect("sqlite::memory:").await.unwrap();
    //
    //     let result = service.load_train_type(cmd, &db).await;
    //
    //     assert!(result.is_err());
    // }



    #[tokio::test]
    async fn test_load_train_type_fail_debug_off() {
        let service = setup_service(false);
        let db: DatabaseConnection = Database::connect("sqlite::memory:").await.unwrap();
        let cmd = vec![
            TrainTypeInfoItem {
                id: "1".to_string(),
                name: "日升".to_string(),
                seat: Default::default(),
            }
        ];
        let result = service.load_train_type(cmd, &db).await;
        assert!(result.is_err());
    }

    // #[tokio::test]
    // async fn test_load_train_number_success() {
    //     let service = setup_service(true);
    //     let db: DatabaseConnection = Database::connect("sqlite::memory:").await.unwrap();
    //     let cmd = vec![
    //         TrainNumberInfoItem {
    //             train_number: "1".to_string(),
    //             train_type: "G".to_string(),
    //             origin_departure_time: 0,
    //             route: vec![
    //                 RouteStationInfo{
    //                     order: 0,
    //                     station: "北京南".to_string(),
    //                     arrival_time: 0,
    //                     departure_time: 0,
    //                 }
    //             ],
    //         }
    //     ];
    //     let result = service.load_train_number(cmd, &db).await;
    //     assert!(result.is_ok());
    // }

    #[tokio::test]
    async fn test_load_train_number_fail_debug_off() {
        let service = setup_service(false);
        let db: DatabaseConnection = Database::connect("sqlite::memory:").await.unwrap();
        let cmd = vec![
            TrainNumberInfoItem {
                train_number: "1".to_string(),
                train_type: "G".to_string(),
                origin_departure_time: 0,
                route: vec![],
            }
        ];
        let result = service.load_train_number(cmd, &db).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_load_dish_takeaway_success() {
        use std::collections::HashMap;
        use std::sync::Arc;
        use tempfile::tempdir;
        use sea_orm::{Database, DatabaseConnection};
        use crate::domain::model::train::{Train, TrainNumber, TrainType};
        use crate::domain::repository::mock::{
            city::mock_city_repository,
            route::mock_route_repository,
            station::mock_station_repository,
            takeaway::mock_takeaway_shop_repo,
            train::mock_train_repo,
        };
        use crate::domain::service::mock::object_storage::mock_object_storage_service;
        use shared::data::RawDishTakeawayInfo;

        // 构造 Train 的 mock
        let mock_train = Train::new(
            Some(1u64.into()),
            TrainNumber::from_unchecked("G111".to_string()),
            TrainType::from_unchecked("G".to_string()),
            HashMap::new(),
            1u64.into(),
            0,
        );

        // Mock TrainRepository
        let mut train_repo = mock_train_repo();
        train_repo.expect_get_trains().returning(move || Ok(vec![mock_train.clone()]));

        // Mock StationRepository
        let mut station_repo = mock_station_repository();
        station_repo.expect_load().returning(|| Ok(vec![]));

        // Mock CityRepository
        let mut city_repo = mock_city_repository();
        city_repo.expect_load().returning(|| Ok(vec![]));

        // Mock TakeawayShopRepository 并设置 save_many_atomic 返回 Ok
        let mut takeaway_repo_mock = mock_takeaway_shop_repo();
        takeaway_repo_mock
            .expect_save_many_atomic()
            .returning(|_items| Ok(()));

        let takeaway_repo = Arc::new(takeaway_repo_mock);

        // 其他 repo/service
        let route_repo = Arc::new(mock_route_repository());
        let object_storage = Arc::new(mock_object_storage_service());

        // 构造服务
        let service = TrainDataServiceImpl::new(
            true,
            tempdir().unwrap().path().to_path_buf(),
            Arc::new(city_repo),
            Arc::new(station_repo),
            Arc::new(train_repo),
            route_repo,
            takeaway_repo,
            object_storage,
        );

        let db: DatabaseConnection = Database::connect("sqlite::memory:").await.unwrap();

        let cmd = vec![RawDishTakeawayInfo {
            train_number: "G111".to_string(),
            dish_info: vec![],
            takeaway_info: Default::default(),
        }];

        let result = service.load_dish_takeaway(cmd, &db).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_dish_takeaway_fail_debug_off() {
        use std::collections::HashMap;
        use std::sync::Arc;
        use tempfile::tempdir;
        use sea_orm::{Database, DatabaseConnection};
        use crate::domain::model::train::{Train, TrainNumber, TrainType};
        use crate::domain::repository::mock::{
            city::mock_city_repository,
            route::mock_route_repository,
            station::mock_station_repository,
            takeaway::mock_takeaway_shop_repo,
            train::mock_train_repo,
        };
        use crate::domain::service::mock::object_storage::mock_object_storage_service;
        use shared::data::RawDishTakeawayInfo;

        // 构造 Train 的 mock
        let mock_train = Train::new(
            Some(1u64.into()),
            TrainNumber::from_unchecked("G111".to_string()),
            TrainType::from_unchecked("G".to_string()),
            HashMap::new(),
            1u64.into(),
            0,
        );

        // Mock TrainRepository
        let mut train_repo = mock_train_repo();
        train_repo.expect_get_trains().returning(move || Ok(vec![mock_train.clone()]));

        // Mock StationRepository
        let mut station_repo = mock_station_repository();
        station_repo.expect_load().returning(|| Ok(vec![]));

        // Mock CityRepository
        let mut city_repo = mock_city_repository();
        city_repo.expect_load().returning(|| Ok(vec![]));

        // Mock TakeawayShopRepository，返回错误以模拟 debug = false 的失败
        let mut takeaway_repo_mock = mock_takeaway_shop_repo();
        takeaway_repo_mock
            .expect_save_many_atomic()
            .returning(|_items| Err(RepositoryError::Db(anyhow!("simulated failure"))));

        let takeaway_repo = Arc::new(takeaway_repo_mock);

        // 其他 repo/service
        let route_repo = Arc::new(mock_route_repository());
        let object_storage = Arc::new(mock_object_storage_service());

        // 构造服务
        let service = TrainDataServiceImpl::new(
            false, // debug = false
            tempdir().unwrap().path().to_path_buf(),
            Arc::new(city_repo),
            Arc::new(station_repo),
            Arc::new(train_repo),
            route_repo,
            takeaway_repo,
            object_storage,
        );

        let db: DatabaseConnection = Database::connect("sqlite::memory:").await.unwrap();

        let cmd = vec![RawDishTakeawayInfo {
            train_number: "G111".to_string(),
            dish_info: vec![],
            takeaway_info: Default::default(),
        }];

        // 执行并验证返回 Err
        let result = service.load_dish_takeaway(cmd, &db).await;
        assert!(result.is_err());
    }

}
