//! TrainQueryServiceImpl 实现
//!
//! 该文件位于 `backend/base/src/infrastructure/application/service/train_query.rs`，
//! 负责把 HTTP / RPC 层收到的 CQRS 命令简单校验后转交给真正的
//! `TrainQueryService`（领域 / 应用服务）。
//!
//! * **为什么需要 `Arc<T>`?**
//!   应用服务通常是无状态、线程安全的，把它包进 `Arc` 便于在整个线程池里共享，
//!   无需再加 `Mutex`（不会修改内部状态）。
//! * **为什么要 `T: Send + Sync + 'static`?**
//!   Actix‑web、Tokio 等异步运行时会把服务对象跨线程移动，
//!   因此需要 `Send`；并发访问需要 `Sync`；`'static` 避免悬垂引用。

/// 火车查询应用服务实现
///
/// 此服务负责处理火车查询相关的应用逻辑，包括直达车次查询和中转车次查询。
/// 主要职责是验证输入参数，然后将请求委派给实际的领域服务处理。
///
/// # 功能
/// - 直达车次查询：根据出发站、目的站和出发日期查询直达火车
/// - 中转车次查询：根据出发城市、目的城市和出发日期查询需要换乘的火车方案
///
/// # 错误处理
/// - 输入验证：检查车站ID和城市ID是否为空
/// - 委托错误：传递领域服务可能返回的错误
///
/// # 实现细节
/// 该服务是对实际查询逻辑的一个适配层，主要负责：
/// 1. 参数校验 - 确保输入参数符合业务规则
/// 2. 错误转换 - 将领域错误映射为应用错误
/// 3. 转发调用 - 将实际查询工作委托给内部服务
///
/// # 示例
/// ```
/// let train_query_service = TrainQueryServiceImpl::new(Arc::new(real_service));
/// let command = DirectTrainQueryCommand {
///     session_id: "sessionId".to_string(),
///     departure_station: "1".to_string(),
///     arrival_station: "2".to_string(),
///     departure_time: NaiveDate::from_ymd_opt(2023, 5, 1).unwrap(),
/// };
/// let result = train_query_service.query_direct_trains(command).await;
/// ```
// Step 1: Define `TrainQueryServiceImpl` application service implementation
// Step 2: Choose correct generics parameter according to data you need
// Exercise 1.2.1D - 5: Your code here. (1 / 6)
// HINT: You may refer to `UserManagerServiceImpl` for example
use std::sync::Arc;

use async_trait::async_trait;

use crate::application::commands::train_query::{
    DirectTrainQueryCommand,
    // TransferTrainQueryCommand,
};
use crate::application::service::train_query::{
    DirectTrainQueryDTO, TrainInfoDTO, TrainQueryService, TrainQueryServiceError,
};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::Identifiable;
use crate::domain::model::station::StationId;
use crate::domain::repository::city::CityRepository;
use crate::domain::service::route::RouteService;
use crate::domain::service::station::StationService;
use crate::domain::service::train_schedule::TrainScheduleService;
use crate::domain::service::train_type::TrainTypeConfigurationService;

// Thinking 1.2.1D - 4: 为何需要使用`+ 'static + Send + Sync`约束泛型参数？
// Thinking 1.2.1D - 5: 为何需要使用`Arc<T>`存储领域服务？为何无需使用`Arc<Mutex<T>>`？
pub struct TrainQueryServiceImpl<T, U, V, W, X>
where
    T: TrainScheduleService + 'static + Send + Sync,
    U: StationService + 'static + Send + Sync,
    V: TrainTypeConfigurationService + 'static + Send + Sync,
    W: RouteService + 'static + Send + Sync,
    X: CityRepository + 'static + Send + Sync,
{
    // Step 3: Store service instance you need using `Arc<T>` and generics parameter
    // HINT: You may refer to `UserManagerServiceImpl` for example
    // Exercise 1.2.1D - 5: Your code here. (2 / 6)
    train_schedule_service: Arc<T>,
    station_service: Arc<U>,
    train_type_service: Arc<V>,
    route_service: Arc<W>,
    city_service: Arc<X>,
}

// Step 4: Implement `new` associate function for `TrainQueryServiceImpl`
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (3 / 6)
impl<T, U, V, W, X> TrainQueryServiceImpl<T, U, V, W, X>
where
    T: TrainScheduleService + 'static + Send + Sync,
    U: StationService + 'static + Send + Sync,
    V: TrainTypeConfigurationService + 'static + Send + Sync,
    W: RouteService + 'static + Send + Sync,
    X: CityRepository + 'static + Send + Sync,
{
    pub fn new(
        train_schedule_service: Arc<T>,
        station_service: Arc<U>,
        train_type_service: Arc<V>,
        route_service: Arc<W>,
        city_service: Arc<X>,
    ) -> Self {
        TrainQueryServiceImpl {
            train_schedule_service,
            station_service,
            train_type_service,
            route_service,
            city_service,
        }
    }

    /// 构建 TrainInfoDTO
    ///
    /// 这个辅助方法负责构建完整的列车信息DTO，从以下服务获取数据：
    /// - TrainScheduleService: 获取列车班次基本信息
    /// - RouteService: 获取路线和停靠站信息
    /// - StationService: 获取车站名称等信息
    /// - TrainTypeConfigurationService: 获取列车座位配置、车型信息等
    async fn build_train_info_dto(
        &self,
        schedule: crate::domain::model::train_schedule::TrainSchedule,
        routes: &[crate::domain::model::route::Route],
        departure_date: chrono::NaiveDate,
    ) -> Result<TrainInfoDTO, Box<dyn ApplicationError>> {
        // 获取列车ID和路线ID
        let train_id = schedule.train_id();
        let route_id = schedule.route_id();

        // 路线信息
        let route_opt = routes.iter().find(|r| r.get_id() == Some(route_id));
        let route = match route_opt {
            Some(r) => r,
            None => {
                let err: Box<dyn ApplicationError> =
                    Box::new(TrainQueryServiceError::InvalidStationId);
                return Err(err);
            }
        };

        // 车站停靠信息
        let mut stopping_stations = Vec::new();
        for stop in route.stops() {
            let station_id = stop.station_id();
            let station_name = match self
                .station_service
                .as_ref()
                .get_station_by_name(station_id.to_string())
                .await
            {
                Ok(Some(station)) => station.name().to_string(),
                _ => format!("Station {}", station_id),
            };

            let base_time = departure_date.and_hms_opt(0, 0, 0).unwrap();
            let arrival_time =
                (base_time + chrono::Duration::seconds(stop.arrival_time() as i64)).to_string();
            let departure_time =
                (base_time + chrono::Duration::seconds(stop.departure_time() as i64)).to_string();

            stopping_stations.push(
                crate::application::service::train_query::StoppingStationInfo {
                    station_name,
                    arrival_time,
                    departure_time,
                },
            );
        }

        if stopping_stations.len() < 2 {
            return Err(Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>);
        }

        // 列车类型信息
        let train_number =
            crate::domain::model::train::TrainNumber::from_unchecked(train_id.to_string());
        let train = match self
            .train_type_service
            .as_ref()
            .get_train_by_number(train_number)
            .await
        {
            Ok(t) => t,
            Err(_) => {
                let err: Box<dyn ApplicationError> =
                    Box::new(TrainQueryServiceError::InvalidStationId);
                return Err(err);
            }
        };
        // 座位信息
        let mut seat_info = std::collections::HashMap::new();
        for seat_type in train.seats().values() {
            let available_count = seat_type.capacity();

            let seat_price = seat_type
                .unit_price()
                .to_string()
                .parse::<u32>()
                .unwrap_or(0);

            seat_info.insert(
                seat_type.name().to_string(),
                crate::application::service::train_query::SeatInfoDTO {
                    seat_type: seat_type.name().to_string(),
                    left: available_count,
                    price: seat_price,
                },
            );
        }

        // 出发站和到达站
        let departure_station = stopping_stations.first().unwrap().station_name.clone();
        let arrival_station = stopping_stations.last().unwrap().station_name.clone();

        // 出发时间和到达时间
        let departure_time = stopping_stations.first().unwrap().departure_time.clone();
        let arrival_time = stopping_stations.last().unwrap().arrival_time.clone();

        // 始发站和终点站信息
        let origin_station = stopping_stations.first().unwrap().station_name.clone();
        let origin_departure_time = stopping_stations.first().unwrap().departure_time.clone();
        let terminal_station = stopping_stations.last().unwrap().station_name.clone();
        let terminal_arrival_time = stopping_stations.last().unwrap().arrival_time.clone();

        // 车次号
        let train_number = train.number().to_string();
        let departure_datetime =
            chrono::NaiveDateTime::parse_from_str(&departure_time, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_else(|_| departure_date.and_hms_opt(0, 0, 0).unwrap());
        let arrival_datetime =
            chrono::NaiveDateTime::parse_from_str(&arrival_time, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_else(|_| {
                    departure_date.and_hms_opt(0, 0, 0).unwrap() + chrono::Duration::hours(2)
                });

        let travel_time = (arrival_datetime - departure_datetime).num_seconds() as u32;

        // 票价
        let price = seat_info.values().map(|info| info.price).min().unwrap_or(0);

        Ok(TrainInfoDTO::new(
            departure_station,
            departure_time,
            arrival_station,
            arrival_time,
            origin_station,
            origin_departure_time,
            terminal_station,
            terminal_arrival_time,
            train_number,
            travel_time,
            price,
            stopping_stations,
            seat_info,
        ))
    }
}

// Step 5: Implement `TrainQueryService` trait for `TrainQueryServiceImpl`
// HINT: You need to use `async_trait` macro
// HINT: You should check user input in application service function,
// return error in validate failed
// HINT: You SHOULD NOT perform business logic in application service
// just delegate business logic to other service
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (4 / 6)
#[async_trait]
impl<T, U, V, W, X> TrainQueryService for TrainQueryServiceImpl<T, U, V, W, X>
where
    T: TrainScheduleService + 'static + Send + Sync,
    U: StationService + 'static + Send + Sync,
    V: TrainTypeConfigurationService + 'static + Send + Sync,
    W: RouteService + 'static + Send + Sync,
    X: CityRepository + 'static + Send + Sync,
{
    async fn query_direct_trains(
        &self,
        command: DirectTrainQueryCommand,
    ) -> Result<DirectTrainQueryDTO, Box<dyn ApplicationError>> {
        // 验证查询一致性：departure_station和departure_city有且仅有一个存在
        let has_departure_station = command.departure_station.is_some()
            && !command
                .departure_station
                .as_ref()
                .unwrap()
                .trim()
                .is_empty();
        let has_departure_city = command.departure_city.is_some()
            && !command.departure_city.as_ref().unwrap().trim().is_empty();

        if has_departure_station == has_departure_city {
            return Err(Box::new(TrainQueryServiceError::InconsistentQuery));
        }

        // 验证查询一致性：arrival_station和arrival_city有且仅有一个存在
        let has_arrival_station = command.arrival_station.is_some()
            && !command.arrival_station.as_ref().unwrap().trim().is_empty();
        let has_arrival_city = command.arrival_city.is_some()
            && !command.arrival_city.as_ref().unwrap().trim().is_empty();

        if has_arrival_station == has_arrival_city {
            return Err(Box::new(TrainQueryServiceError::InconsistentQuery));
        }
        // 车站查询
        let schedules = if has_departure_station && has_arrival_station {
            let departure_station_name = command.departure_station.as_ref().unwrap().trim();
            let departure_station_opt = self
                .station_service
                .as_ref()
                .get_station_by_name(departure_station_name.to_string())
                .await
                .map_err(|_| {
                    Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                })?;

            // 检查是否找到了出发站点
            let departure_station = match departure_station_opt {
                Some(station) => station,
                None => {
                    return Err(Box::new(TrainQueryServiceError::InvalidStationId)
                        as Box<dyn ApplicationError>);
                }
            };

            // 根据站点名称查询到达站点信息
            let arrival_station_name = command.arrival_station.as_ref().unwrap().trim();
            let arrival_station_opt = self
                .station_service
                .as_ref()
                .get_station_by_name(arrival_station_name.to_string())
                .await
                .map_err(|_| {
                    Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                })?;

            // 检查是否找到了到达站点
            let arrival_station = match arrival_station_opt {
                Some(station) => station,
                None => {
                    return Err(Box::new(TrainQueryServiceError::InvalidStationId)
                        as Box<dyn ApplicationError>);
                }
            };

            // 使用站点ID进行查询
            self.train_schedule_service
                .as_ref()
                .find_schedules(
                    command.departure_time,
                    departure_station.get_id().unwrap(),
                    arrival_station.get_id().unwrap(),
                )
                .await
                .map_err(|err| {
                    // 根据领域错误类型返回不同的应用错误
                    match err {
                        crate::domain::service::train_schedule::TrainScheduleServiceError::InvalidStationId(_) => {
                            Box::new(TrainQueryServiceError::InvalidStationId) as Box<dyn ApplicationError>
                        },
                        crate::domain::service::train_schedule::TrainScheduleServiceError::InfrastructureError(_) => {
                            Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                        }
                    }
                })?
        } else {
            // 城市查询
            let departure_city_name = command.departure_city.as_ref().unwrap().trim();
            // 使用 city_service 查询城市信息
            let departure_cities = self
                .city_service
                .find_by_name(departure_city_name)
                .await
                .map_err(|_| {
                    Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                })?;

            if departure_cities.is_empty() {
                return Err(
                    Box::new(TrainQueryServiceError::InvalidCityId) as Box<dyn ApplicationError>
                );
            }

            let departure_city_id = departure_cities[0].get_id().unwrap();

            let arrival_city_name = command.arrival_city.as_ref().unwrap().trim();
            let arrival_cities = self
                .city_service
                .find_by_name(arrival_city_name)
                .await
                .map_err(|_| {
                    Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                })?;

            if arrival_cities.is_empty() {
                return Err(
                    Box::new(TrainQueryServiceError::InvalidCityId) as Box<dyn ApplicationError>
                );
            }

            let arrival_city_id = arrival_cities[0].get_id().unwrap();

            // 使用 StationService 查询城市所有车站
            let departure_stations = self
                .station_service
                .get_station_by_city(departure_city_id)
                .await
                .map_err(|_| {
                    Box::new(TrainQueryServiceError::InvalidCityId) as Box<dyn ApplicationError>
                })?;

            let arrival_stations = self
                .station_service
                .get_station_by_city(arrival_city_id)
                .await
                .map_err(|_| {
                    Box::new(TrainQueryServiceError::InvalidCityId) as Box<dyn ApplicationError>
                })?;

            // 检查查询到的车站是否为空
            if departure_stations.is_empty() {
                return Err(
                    Box::new(TrainQueryServiceError::InvalidCityId) as Box<dyn ApplicationError>
                );
            }

            if arrival_stations.is_empty() {
                return Err(
                    Box::new(TrainQueryServiceError::InvalidCityId) as Box<dyn ApplicationError>
                );
            }

            // 查询所有可能的出发站和到达站组合
            // 创建一个向量来收集所有查询结果
            let mut all_schedules = Vec::new();

            // 遍历所有出发站和到达站的组合
            for departure_station in &departure_stations {
                for arrival_station in &arrival_stations {
                    // 对每一对站点组合进行查询
                    match self
                        .train_schedule_service
                        .as_ref()
                        .find_schedules(
                            command.departure_time,
                            departure_station.get_id().unwrap(),
                            arrival_station.get_id().unwrap(),
                        )
                        .await
                    {
                        Ok(schedules) => {
                            all_schedules.extend(schedules);
                        }
                        Err(err) => {
                            match err {
                                crate::domain::service::train_schedule::TrainScheduleServiceError::InvalidStationId(_) => {
                                    return Err(Box::new(TrainQueryServiceError::InvalidStationId)
                                        as Box<dyn ApplicationError>);
                                }
                                crate::domain::service::train_schedule::TrainScheduleServiceError::InfrastructureError(_) => {
                                    return Err(Box::new(GeneralError::InternalServerError)
                                        as Box<dyn ApplicationError>);
                                }
                            }
                        }
                    }
                }
            }

            all_schedules
        };

        // 获取路线信息
        let routes = self.route_service.get_routes().await.map_err(|_| {
            Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
        })?;

        // 收集所有异步构建的 TrainInfoDTO
        let mut solutions = Vec::new();
        let mut build_errors = Vec::new();
        for schedule in schedules {
            match self
                .build_train_info_dto(schedule, &routes, command.departure_time)
                .await
            {
                Ok(train_info) => solutions.push(train_info),
                Err(e) => {
                    let error_msg = format!("{:?}", e);

                    if error_msg.contains("NotFound") {
                        build_errors.push(e);
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        if solutions.is_empty() && !build_errors.is_empty() {
            return Err(build_errors.remove(0));
        }

        // 将领域模型转换为DTO返回
        Ok(DirectTrainQueryDTO { solutions })
    }

    // async fn query_transfer_trains(
    //     &self,
    //     command: TransferTrainQueryCommand,
    // ) -> Result<TransferTrainQueryDTO, Box<dyn ApplicationError>> {
    //     // 参数验证
    //     if command.from_city_id.trim().is_empty() || command.to_city_id.trim().is_empty() {
    //         return Err(GeneralError::BadRequest("city id 不能为空".into()).into());
    //     }

    //     // 目前没有实现转接列车查询，返回空结果
    //     Ok(TransferTrainQueryDTO { solutions: vec![] })
    // }
}

// Step 6: Add unit test for your implementation
// HINT: You may use `mockall` crate to "mock" other service you depend on
// HINT: You may use AI tools to generate unit test
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (5 / 6)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::city::CityId;
    use crate::domain::model::station::Station;
    use crate::domain::service::station::StationServiceError;
    use chrono::NaiveDate;
    use mockall::{mock, predicate::*};
    use std::sync::Arc;

    mock! {
        pub TrainScheduleService {}
        #[async_trait]
        impl TrainScheduleService for TrainScheduleService {
            async fn add_schedule(
                &self,
                train_id: crate::domain::model::train::TrainId,
                date: NaiveDate,
            ) -> Result<(), crate::domain::service::train_schedule::TrainScheduleServiceError>;

            async fn get_schedules(
                &self,
                date: NaiveDate,
            ) -> Result<Vec<crate::domain::model::train_schedule::TrainSchedule>, crate::domain::service::train_schedule::TrainScheduleServiceError>;

            async fn find_schedules(
                &self,
                date: NaiveDate,
                departure_station: crate::domain::model::station::StationId,
                arrival_station: crate::domain::model::station::StationId,
            ) -> Result<Vec<crate::domain::model::train_schedule::TrainSchedule>, crate::domain::service::train_schedule::TrainScheduleServiceError>;
        }
    }

    mock! {
        pub TrainTypeConfigurationService {}
        #[async_trait]
        impl TrainTypeConfigurationService for TrainTypeConfigurationService {
            async fn verify_seat_type_name(
                &self,
                train_id: crate::domain::model::train::TrainId,
                seat_type_name: crate::domain::model::train::SeatTypeName<crate::Unverified>,
            ) -> Result<crate::domain::model::train::SeatTypeName<crate::Verified>, crate::domain::service::train_type::TrainTypeConfigurationServiceError>;

            async fn verify_train_number(
                &self,
                train_number: crate::domain::model::train::TrainNumber<crate::Unverified>,
            ) -> Result<crate::domain::model::train::TrainNumber<crate::Verified>, crate::domain::service::train_type::TrainTypeConfigurationServiceError>;

            async fn verify_train_type(
                &self,
                train_type: crate::domain::model::train::TrainType<crate::Unverified>,
            ) -> Result<crate::domain::model::train::TrainType<crate::Verified>, crate::domain::service::train_type::TrainTypeConfigurationServiceError>;

            async fn get_seat_id_map(
                &self,
                train_id: crate::domain::model::train::TrainId,
            ) -> Result<std::collections::HashMap<crate::domain::model::train::SeatTypeName<crate::Verified>, Vec<crate::domain::model::train_schedule::SeatId>>, crate::domain::service::train_type::TrainTypeConfigurationServiceError>;

            async fn get_trains(&self) -> Result<Vec<crate::domain::model::train::Train>, crate::domain::service::train_type::TrainTypeConfigurationServiceError>;

            async fn get_train_by_number(
                &self,
                train_number: crate::domain::model::train::TrainNumber<crate::Verified>,
            ) -> Result<crate::domain::model::train::Train, crate::domain::service::train_type::TrainTypeConfigurationServiceError>;

            async fn add_train_type(
                &self,
                train_number: crate::domain::model::train::TrainNumber<crate::Verified>,
                train_type: crate::domain::model::train::TrainType<crate::Verified>,
                seat_configuration: Vec<crate::domain::model::train::SeatType>,
                default_route_id: crate::domain::model::route::RouteId,
                default_origin_departure_time: i32,
            ) -> Result<crate::domain::model::train::TrainId, crate::domain::service::train_type::TrainTypeConfigurationServiceError>;

            async fn modify_train_type(
                &self,
                train_id: crate::domain::model::train::TrainId,
                train_number: crate::domain::model::train::TrainNumber<crate::Verified>,
                train_type: crate::domain::model::train::TrainType<crate::Verified>,
                seat_configuration: Vec<crate::domain::model::train::SeatType>,
                default_route_id: crate::domain::model::route::RouteId,
                default_origin_departure_time: i32,
            ) -> Result<(), crate::domain::service::train_type::TrainTypeConfigurationServiceError>;

            async fn remove_train_type(
                &self,
                train: crate::domain::model::train::Train,
            ) -> Result<(), crate::domain::service::train_type::TrainTypeConfigurationServiceError>;
        }
    }

    mock! {
        pub RouteService {}
        #[async_trait]
        impl RouteService for RouteService {
            async fn get_route_map(&self) -> Result<crate::domain::service::route::RouteGraph, crate::domain::service::route::RouteServiceError>;

            async fn add_route(&self, stops: Vec<crate::domain::model::route::Stop>) -> Result<crate::domain::model::route::RouteId, crate::domain::service::route::RouteServiceError>;

            async fn get_routes(&self) -> Result<Vec<crate::domain::model::route::Route>, crate::domain::service::route::RouteServiceError>;
        }
    }

    mock! {
        pub StationService {}
        #[async_trait]
        impl StationService for StationService {
            async fn get_stations(&self) -> Result<Vec<Station>, StationServiceError>;
            async fn get_station_by_city(&self, city_id: CityId) -> Result<Vec<Station>, StationServiceError>;
            async fn get_station_by_name(&self, station_name: String) -> Result<Option<Station>, StationServiceError>;
            async fn add_station(&self, station_name: String, city_name: String) -> Result<StationId, StationServiceError>;
            async fn modify_station(&self, station_id: StationId, station_name: String, city_name: String) -> Result<(), StationServiceError>;
            async fn delete_station(&self, station: Station) -> Result<(), StationServiceError>;
        }
    }

    mock! {
        pub CityRepository {}
        #[async_trait]
        impl CityRepository for CityRepository {
            async fn load(&self) -> Result<Vec<crate::domain::model::city::City>, crate::domain::RepositoryError>;
            async fn find_by_name(&self, city_name: &str) -> Result<Vec<crate::domain::model::city::City>, crate::domain::RepositoryError>;
            async fn find_by_province(
                &self,
                province_name: crate::domain::model::city::ProvinceName,
            ) -> Result<Vec<crate::domain::model::city::City>, crate::domain::RepositoryError>;
            async fn save_raw(&self, city_data: shared::data::CityData) -> Result<(), crate::domain::RepositoryError>;
        }
    }

    #[tokio::test]
    async fn delegating_direct_trains() {
        let mut mock_train = MockTrainScheduleService::new();
        let mock_station = MockStationService::new();
        let mock_train_type = MockTrainTypeConfigurationService::new();
        let mock_route = MockRouteService::new();
        let mock_city = MockCityRepository::new();
        let cmd = DirectTrainQueryCommand {
            session_id: Default::default(),
            departure_station: Some("1".to_string()),
            arrival_station: Some("2".to_string()),
            departure_city: None,
            arrival_city: None,
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };

        // 从数字创建StationId
        let departure_station = StationId::from(1u64);
        let arrival_station = StationId::from(2u64);

        // 模拟领域服务返回空结果
        mock_train
            .expect_find_schedules()
            .withf(move |date, from_id, to_id| {
                from_id == &departure_station
                    && to_id == &arrival_station
                    && *date == NaiveDate::from_ymd_opt(2025, 5, 1).unwrap()
            })
            .times(1)
            .returning(|_, _, _| Ok(vec![]));

        let svc = TrainQueryServiceImpl::new(
            Arc::new(mock_train),
            Arc::new(mock_station),
            Arc::new(mock_train_type),
            Arc::new(mock_route),
            Arc::new(mock_city),
        );
        let res = svc.query_direct_trains(cmd).await.unwrap();
        assert!(res.solutions.is_empty());
    }

    #[tokio::test]
    async fn test_query_direct_with_empty_from_station() {
        let mock_train = MockTrainScheduleService::new();
        let mock_station = MockStationService::new();
        let mock_train_type = MockTrainTypeConfigurationService::new();
        let mock_route = MockRouteService::new();
        let mock_city = MockCityRepository::new();
        let cmd = DirectTrainQueryCommand {
            session_id: Default::default(),
            departure_station: Some("".to_string()), // 空出发站
            arrival_station: Some("2".to_string()),
            departure_city: None,
            arrival_city: None,
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };

        // 我们期望服务本身拒绝这个请求，不会将其传递到内部服务
        let svc = TrainQueryServiceImpl::new(
            Arc::new(mock_train),
            Arc::new(mock_station),
            Arc::new(mock_train_type),
            Arc::new(mock_route),
            Arc::new(mock_city),
        );
        let result = svc.query_direct_trains(cmd).await;

        // 应该返回错误，表示查询不一致
        assert!(result.is_err());
        if let Err(e) = result {
            // 检查错误类型是否正确
            let err_str = format!("{:?}", e);
            assert!(err_str.contains("Inconsistent query"));
        }
    }

    #[tokio::test]
    async fn test_query_direct_with_empty_to_station() {
        let mock_train = MockTrainScheduleService::new();
        let mock_station = MockStationService::new();
        let mock_train_type = MockTrainTypeConfigurationService::new();
        let mock_route = MockRouteService::new();
        let mock_city = MockCityRepository::new();
        let cmd = DirectTrainQueryCommand {
            session_id: Default::default(),
            departure_station: Some("1".to_string()),
            arrival_station: Some("".to_string()), // 空目的站
            departure_city: None,
            arrival_city: None,
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };

        // 我们期望服务本身拒绝这个请求，不会将其传递到内部服务
        let svc = TrainQueryServiceImpl::new(
            Arc::new(mock_train),
            Arc::new(mock_station),
            Arc::new(mock_train_type),
            Arc::new(mock_route),
            Arc::new(mock_city),
        );
        let result = svc.query_direct_trains(cmd).await;

        // 应该返回错误，表示查询不一致
        assert!(result.is_err());
        if let Err(e) = result {
            // 检查错误类型是否正确
            let err_str = format!("{:?}", e);
            assert!(err_str.contains("Inconsistent query"));
        }
    }

    #[tokio::test]
    async fn test_query_direct_with_invalid_station_id() {
        let mock_train = MockTrainScheduleService::new();
        let mock_station = MockStationService::new();
        let mock_train_type = MockTrainTypeConfigurationService::new();
        let mock_route = MockRouteService::new();
        let mock_city = MockCityRepository::new();
        let cmd = DirectTrainQueryCommand {
            session_id: Default::default(),
            departure_station: Some("invalid".to_string()), // 非数字ID
            arrival_station: Some("2".to_string()),
            departure_city: None,
            arrival_city: None,
            departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
        };

        // 我们需要模拟station_service的行为，因为现在代码使用get_station_by_name
        mock_station
            .expect_get_station_by_name()
            .with(eq("invalid".to_string()))
            .times(1)
            .returning(|_| Ok(None)); // 返回None表示未找到站点

        // 我们期望服务返回站点未找到的错误
        let svc = TrainQueryServiceImpl::new(
            Arc::new(mock_train),
            Arc::new(mock_station),
            Arc::new(mock_train_type),
            Arc::new(mock_route),
            Arc::new(mock_city),
        );
        let result = svc.query_direct_trains(cmd).await;

        // 应该返回错误，表示站点未找到
        assert!(result.is_err());
        if let Err(e) = result {
            // 检查错误类型是否正确
            let err_str = format!("{:?}", e);
            assert!(err_str.contains("NotFound"));
        }
    }

    // #[tokio::test]
    // async fn test_query_transfer_with_empty_from_city() {
    //     let mock = MockTrainScheduleService::new();
    //     let cmd = TransferTrainQueryCommand {
    //         session_id: Default::default(),
    //         from_city_id: "".into(), // 空出发城市
    //         to_city_id: "C2".into(),
    //         departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
    //     };

    //     // 我们期望服务本身拒绝这个请求，不会将其传递到内部服务
    //     let svc = TrainQueryServiceImpl::new(Arc::new(mock));
    //     let result = svc.query_transfer_trains(cmd).await;

    //     // 应该返回错误，表示城市ID不能为空
    //     assert!(result.is_err());
    //     if let Err(e) = result {
    //         // 检查错误类型是否正确
    //         let err_str = format!("{:?}", e);
    //         assert!(err_str.contains("city id 不能为空"));
    //     }
    // }

    // #[tokio::test]
    // async fn test_query_transfer_with_empty_to_city() {
    //     let mock = MockTrainScheduleService::new();
    //     let cmd = TransferTrainQueryCommand {
    //         session_id: Default::default(),
    //         from_city_id: "C1".into(),
    //         to_city_id: "".into(), // 空目的城市
    //         departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
    //     };

    //     // 我们期望服务本身拒绝这个请求，不会将其传递到内部服务
    //     let svc = TrainQueryServiceImpl::new(Arc::new(mock));
    //     let result = svc.query_transfer_trains(cmd).await;

    //     // 应该返回错误，表示城市ID不能为空
    //     assert!(result.is_err());
    //     if let Err(e) = result {
    //         // 检查错误类型是否正确
    //         let err_str = format!("{:?}", e);
    //         assert!(err_str.contains("city id 不能为空"));
    //     }
    // }

    // #[tokio::test]
    // async fn test_query_transfer_success() {
    //     let mock = MockTrainScheduleService::new();
    //     let cmd = TransferTrainQueryCommand {
    //         session_id: Default::default(),
    //         from_city_id: "C1".into(),
    //         to_city_id: "C2".into(),
    //         departure_time: NaiveDate::from_ymd_opt(2025, 5, 1).unwrap(),
    //     };

    //     // 测试简单的成功路径，转接列车查询目前返回空结果
    //     let svc = TrainQueryServiceImpl::new(Arc::new(mock));
    //     let res = svc.query_transfer_trains(cmd).await.unwrap();
    //     assert!(res.solutions.is_empty());
    // }
}

// Step 7: Write document comment and mod comment for your implementation
// HINT: You may use AI tools to generate comment
// HINT: You may refer to `UserManagerServiceImpl` for example
// Exercise 1.2.1D - 5: Your code here. (6 / 6)

// Good! Next, register your application service in `api::main`
