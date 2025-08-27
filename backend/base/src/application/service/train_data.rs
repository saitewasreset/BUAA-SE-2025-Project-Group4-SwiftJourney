//! 列车数据服务模块
//!
//! 该模块提供列车基础数据相关的服务接口，包括：
//! - 城市数据的加载
//! - 车站数据的加载
//! - 列车类型数据的加载
//! - 列车车次数据的加载
//!
//! 主要功能通过[`TrainDataService`] trait定义，包含各类基础数据的加载方法。
//! 该服务由基础设施层实现，供应用层调用以初始化系统基础数据。
use crate::application::ApplicationError;
use crate::application::commands::train_data::{
    LoadCityCommand, LoadDishTakeawayCommand, LoadStationCommand, LoadTrainNumberCommand,
    LoadTrainTypeCommand,
};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;

/// 列车数据服务接口
///
/// 定义了加载各类列车基础数据的核心操作，所有实现都应该是线程安全的。
/// 通常用于系统初始化或数据批量导入场景。
///
/// # Methods
/// - `is_debug_mode`: 检查服务是否处于调试模式
/// - `load_city`: 加载城市数据
/// - `load_station`: 加载车站数据
/// - `load_train_type`: 加载列车类型数据
/// - `load_train_number`: 加载列车车次数据
#[async_trait]
pub trait TrainDataService: 'static + Send + Sync {
    /// 检查服务是否处于调试模式
    fn is_debug_mode(&self) -> bool;
    /// 加载城市数据
    ///
    /// 根据[`LoadCityCommand`]提供的信息创建或更新城市数据。
    ///
    /// # Arguments
    /// * `command` - 包含城市数据的命令对象
    async fn load_city(&self, command: LoadCityCommand) -> Result<(), Box<dyn ApplicationError>>;

    /// 加载车站数据
    ///
    /// 根据[`LoadStationCommand`]提供的信息创建或更新车站数据。
    /// 车站必须关联到已存在的城市。
    ///
    /// # Arguments
    /// * `command` - 包含车站数据的命令对象
    async fn load_station(
        &self,
        command: LoadStationCommand,
    ) -> Result<(), Box<dyn ApplicationError>>;

    /// 加载列车类型数据
    ///
    /// 根据[`LoadTrainTypeCommand`]提供的信息创建或更新列车类型数据。
    ///
    /// # Arguments
    /// * `command` - 包含列车类型数据的命令对象
    async fn load_train_type(
        &self,
        command: LoadTrainTypeCommand,
        db: DatabaseConnection,
    ) -> Result<(), Box<dyn ApplicationError>>;

    /// 加载列车车次数据
    ///
    /// 根据[`LoadTrainNumberCommand`]提供的信息创建或更新列车车次数据。
    /// 车次必须关联到已存在的列车类型。
    ///
    /// # Arguments
    /// * `command` - 包含列车车次数据的命令对象
    async fn load_train_number(
        &self,
        command: LoadTrainNumberCommand,
        db: DatabaseConnection,
    ) -> Result<(), Box<dyn ApplicationError>>;

    async fn load_dish_takeaway(
        &self,
        command: LoadDishTakeawayCommand,
        db: DatabaseConnection,
    ) -> Result<(), Box<dyn ApplicationError>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use shared::data::{StationDataItem, TrainTypeInfoItem, TrainNumberInfoItem};

    #[derive(Debug)]
    struct MockError;
    impl std::fmt::Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Mock error")
        }
    }
    impl std::error::Error for MockError {}
    impl ApplicationError for MockError {
        fn error_code(&self) -> u32 { 9999 }
        fn error_message(&self) -> String { "Mock error".to_string() }
    }

    struct TestTrainDataService {
        fail: bool,
        debug: bool,
    }

    #[async_trait]
    impl TrainDataService for TestTrainDataService {
        fn is_debug_mode(&self) -> bool { self.debug }

        async fn load_city(&self, _command: LoadCityCommand) -> Result<(), Box<dyn ApplicationError>> {
            if self.fail { Err(Box::new(MockError)) } else { Ok(()) }
        }

        async fn load_station(&self, _command: LoadStationCommand) -> Result<(), Box<dyn ApplicationError>> {
            if self.fail { Err(Box::new(MockError)) } else { Ok(()) }
        }

        async fn load_train_type(&self, _command: LoadTrainTypeCommand) -> Result<(), Box<dyn ApplicationError>> {
            if self.fail { Err(Box::new(MockError)) } else { Ok(()) }
        }

        async fn load_train_number(&self, _command: LoadTrainNumberCommand) -> Result<(), Box<dyn ApplicationError>> {
            if self.fail { Err(Box::new(MockError)) } else { Ok(()) }
        }

        async fn load_dish_takeaway(&self, _command: LoadDishTakeawayCommand) -> Result<(), Box<dyn ApplicationError>> {
            if self.fail { Err(Box::new(MockError)) } else { Ok(()) }
        }
    }

    #[tokio::test]
    async fn test_is_debug_mode() {
        let service = TestTrainDataService { fail: false, debug: true };
        assert!(service.is_debug_mode());

        let service2 = TestTrainDataService { fail: false, debug: false };
        assert!(!service2.is_debug_mode());
    }

    #[tokio::test]
    async fn test_load_city_success_and_failure() {
        let service = TestTrainDataService { fail: false, debug: false };
        let city_data: LoadCityCommand = HashMap::new(); // 空数据也可
        assert!(service.load_city(city_data.clone()).await.is_ok());

        let service_fail = TestTrainDataService { fail: true, debug: false };
        assert!(service_fail.load_city(city_data).await.is_err());
    }

    #[tokio::test]
    async fn test_load_station_success_and_failure() {
        let service = TestTrainDataService { fail: false, debug: false };
        let station_data: LoadStationCommand = vec![StationDataItem { name: "StationA".into(), city: "CityX".into() }];
        assert!(service.load_station(station_data.clone()).await.is_ok());

        let service_fail = TestTrainDataService { fail: true, debug: false };
        assert!(service_fail.load_station(station_data).await.is_err());
    }

    #[tokio::test]
    async fn test_load_train_type_success_and_failure() {
        let service = TestTrainDataService { fail: false, debug: false };
        let train_type_data: LoadTrainTypeCommand = vec![TrainTypeInfoItem {
            id: "T1".into(),
            name: "Express".into(),
            seat: HashMap::new(),
        }];
        assert!(service.load_train_type(train_type_data.clone()).await.is_ok());

        let service_fail = TestTrainDataService { fail: true, debug: false };
        assert!(service_fail.load_train_type(train_type_data).await.is_err());
    }

    #[tokio::test]
    async fn test_load_train_number_success_and_failure() {
        let service = TestTrainDataService { fail: false, debug: false };
        let train_number_data: LoadTrainNumberCommand = vec![TrainNumberInfoItem {
            train_number: "G123".into(),
            train_type: "Express".into(),
            origin_departure_time: 800,
            route: vec![],
        }];
        assert!(service.load_train_number(train_number_data.clone()).await.is_ok());

        let service_fail = TestTrainDataService { fail: true, debug: false };
        assert!(service_fail.load_train_number(train_number_data).await.is_err());
    }

    #[tokio::test]
    async fn test_load_dish_takeaway_success_and_failure() {
        let service = TestTrainDataService { fail: false, debug: false };
        let dish_data: LoadDishTakeawayCommand = vec![]; // 空数据
        assert!(service.load_dish_takeaway(dish_data.clone()).await.is_ok());

        let service_fail = TestTrainDataService { fail: true, debug: false };
        assert!(service_fail.load_dish_takeaway(dish_data).await.is_err());
    }
}

