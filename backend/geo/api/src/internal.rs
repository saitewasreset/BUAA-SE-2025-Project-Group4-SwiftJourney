use actix_web::{get, post, web};
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use shared::api::{ApiResponse, ApplicationErrorBox, parse_request_body};
use shared::application_error::ApplicationError;
use shared::internal::geo::command::{SaveCityProvinceMapCommand, SaveStationCityMapCommand};
use shared::internal::geo::dto::{DbCityDTO, DbStationDTO};

#[get("/db_get_cities")]
pub async fn db_get_cities(
    db: web::Data<DatabaseConnection>,
) -> Result<ApiResponse<Vec<DbCityDTO>>, ApplicationErrorBox> {
    let items = geo_base::models::city::Entity::find()
        .all(db.as_ref())
        .await
        .map_err(|_e| {
            ApplicationErrorBox::from(Box::new(
                shared::application_error::GeneralError::InternalServerError,
            ) as Box<dyn ApplicationError>)
        })?
        .into_iter()
        .map(|m| DbCityDTO {
            id: m.id,
            name: m.name,
            province: m.province,
        })
        .collect();
    ApiResponse::ok(items)
}

#[get("/db_get_stations")]
pub async fn db_get_stations(
    db: web::Data<DatabaseConnection>,
) -> Result<ApiResponse<Vec<DbStationDTO>>, ApplicationErrorBox> {
    let items = geo_base::models::station::Entity::find()
        .all(db.as_ref())
        .await
        .map_err(|_e| {
            ApplicationErrorBox::from(Box::new(
                shared::application_error::GeneralError::InternalServerError,
            ) as Box<dyn ApplicationError>)
        })?
        .into_iter()
        .map(|m| DbStationDTO {
            id: m.id,
            name: m.name,
            city_id: m.city_id,
        })
        .collect();
    ApiResponse::ok(items)
}

#[post("/save_city_province_map")]
pub async fn save_city_province_map(
    db: web::Data<DatabaseConnection>,
    body: web::Bytes,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let cmd: SaveCityProvinceMapCommand = parse_request_body(body)?;
    use sea_orm::sea_query::OnConflict;
    use sea_orm::{ActiveValue, EntityTrait};

    let models = cmd
        .city_province_map
        .into_iter()
        .map(|(city, province)| geo_base::models::city::ActiveModel {
            id: ActiveValue::NotSet,
            name: ActiveValue::Set(city),
            province: ActiveValue::Set(province),
        })
        .collect::<Vec<_>>();

    geo_base::models::city::Entity::insert_many(models)
        .on_conflict(
            OnConflict::column(geo_base::models::city::Column::Name)
                .update_columns([geo_base::models::city::Column::Province])
                .to_owned(),
        )
        .exec(db.as_ref())
        .await
        .map_err(|_e| {
            ApplicationErrorBox::from(Box::new(
                shared::application_error::GeneralError::InternalServerError,
            ) as Box<dyn ApplicationError>)
        })?;

    ApiResponse::ok(())
}

#[post("/save_station_city_map")]
pub async fn save_station_city_map(
    db: web::Data<DatabaseConnection>,
    body: web::Bytes,
) -> Result<ApiResponse<()>, ApplicationErrorBox> {
    let cmd: SaveStationCityMapCommand = parse_request_body(body)?;
    use sea_orm::sea_query::OnConflict;
    use sea_orm::{ActiveValue, EntityTrait};

    // 需要根据 city 表先拿到 city_id 映射
    let city_map = geo_base::models::city::Entity::find()
        .all(db.as_ref())
        .await
        .map_err(|_e| {
            ApplicationErrorBox::from(Box::new(
                shared::application_error::GeneralError::InternalServerError,
            ) as Box<dyn ApplicationError>)
        })?
        .into_iter()
        .map(|m| (m.name, m.id))
        .collect::<std::collections::HashMap<_, _>>();

    let models = cmd
        .station_city_map
        .into_iter()
        .filter_map(|(station, city)| {
            city_map
                .get(&city)
                .copied()
                .map(|city_id| geo_base::models::station::ActiveModel {
                    id: ActiveValue::NotSet,
                    name: ActiveValue::Set(station),
                    city_id: ActiveValue::Set(city_id),
                })
        })
        .collect::<Vec<_>>();

    geo_base::models::station::Entity::insert_many(models)
        .on_conflict(
            OnConflict::column(geo_base::models::station::Column::Name)
                .update_columns([geo_base::models::station::Column::CityId])
                .to_owned(),
        )
        .exec(db.as_ref())
        .await
        .map_err(|_e| {
            ApplicationErrorBox::from(Box::new(
                shared::application_error::GeneralError::InternalServerError,
            ) as Box<dyn ApplicationError>)
        })?;

    ApiResponse::ok(())
}

pub fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(db_get_cities)
        .service(db_get_stations)
        .service(save_city_province_map)
        .service(save_station_city_map);
}
