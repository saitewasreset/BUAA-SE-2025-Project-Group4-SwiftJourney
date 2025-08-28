// #![cfg(test)]
// 
// use crate::domain::model::city::CityId;
// use crate::domain::model::hotel::{Hotel, HotelId};
// use crate::domain::model::station::StationId;
// use crate::domain::{Repository, RepositoryError};
// use crate::domain::repository::hotel::HotelRepository;
// use async_trait::async_trait;
// use mockall::mock;
// use uuid::Uuid;
// 
// // 使用 for<'a> 来声明一个高阶生命周期 'a
// for<'a> mock! {
//     pub HotelRepository {}
// 
//     #[async_trait]
//     impl HotelRepository for HotelRepository {
//         async fn get_id_by_uuid(&self, uuid: Uuid) -> Result<Option<HotelId>, RepositoryError>;
//         async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<Hotel>, RepositoryError>;
// 
//         // 在 &str 上显式地使用生命周期 'a
//         async fn find_by_city(
//             &self,
//             city_id: CityId,
//             name_pattern: Option<&'a str>, // <--- 修改点
//         ) -> Result<Vec<Hotel>, RepositoryError>;
// 
//         // 在 &str 上显式地使用生命周期 'a
//         async fn find_by_station(
//             &self,
//             station_id: StationId,
//             name_pattern: Option<&'a str>, // <--- 修改点
//         ) -> Result<Vec<Hotel>, RepositoryError>;
//     }
// 
//     #[async_trait]
//     impl Repository<Hotel> for HotelRepository {
//         async fn find(&self, id: HotelId) -> Result<Option<Hotel>, RepositoryError>;
//         async fn remove(&self, aggregate: Hotel) -> Result<(), RepositoryError>;
//         async fn save(&self, aggregate: &mut Hotel) -> Result<HotelId, RepositoryError>;
//     }
// }