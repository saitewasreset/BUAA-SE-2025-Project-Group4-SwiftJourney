use crate::application::commands::train_data::{LoadCityCommand, LoadStationCommand};
use crate::domain::Identifiable;
use crate::domain::model::city::{City, CityId, CityName, ProvinceName};
use crate::domain::model::station::{Station, StationId};
use crate::domain::repository::city::CityRepository;
use crate::domain::repository::station::StationRepository;
use actix_web::{HttpResponse, Scope, get, post, web};
use shared::dto::{CityDTO, SaveResultDTO, StationDTO};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub city_repo: Arc<dyn CityRepository>,
    pub station_repo: Arc<dyn StationRepository>,
    pub debug: bool,
}

#[get("/city")]
async fn get_city_info(state: web::Data<AppState>) -> HttpResponse {
    match state.city_repo.load().await {
        Ok(cities) => {
            let mut map: HashMap<String, Vec<String>> = HashMap::new();
            for city in cities {
                map.entry(city.province().to_string())
                    .or_default()
                    .push(city.name().to_string());
            }
            HttpResponse::Ok().json(map)
        }
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/city-stations")]
async fn get_city_stations(state: web::Data<AppState>) -> HttpResponse {
    // Build city id->name map
    let Ok(cities) = state.city_repo.load().await else {
        return HttpResponse::InternalServerError().finish();
    };
    let mut city_id_to_name: HashMap<u64, String> = HashMap::new();
    for city in cities {
        if let Some(id) = city.get_id() {
            city_id_to_name.insert(id.into(), city.name().to_string());
        }
    }

    let Ok(stations) = state.station_repo.load().await else {
        return HttpResponse::InternalServerError().finish();
    };
    let mut city_station_map: HashMap<String, Vec<String>> = HashMap::new();
    for st in stations {
        if let Some(city_name) = city_id_to_name.get(&st.city_id().into()) {
            city_station_map
                .entry(city_name.clone())
                .or_default()
                .push(st.name().to_string());
        }
    }
    HttpResponse::Ok().json(city_station_map)
}

#[post("/load/city")]
async fn post_load_city(
    state: web::Data<AppState>,
    payload: web::Json<LoadCityCommand>,
) -> HttpResponse {
    if !state.debug {
        return HttpResponse::Forbidden().finish();
    }
    match state.city_repo.save_raw(payload.into_inner()).await {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/load/station")]
async fn post_load_station(
    state: web::Data<AppState>,
    payload: web::Json<LoadStationCommand>,
) -> HttpResponse {
    if !state.debug {
        return HttpResponse::Forbidden().finish();
    }
    match state.station_repo.save_raw(payload.into_inner()).await {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}

pub fn scope() -> Scope {
    web::scope("/geo")
        .service(get_city_info)
        .service(get_city_stations)
        .service(post_load_city)
        .service(post_load_station)
        // CRUD style endpoints used by base via HTTP repos
        .service(
            web::resource("/cities")
                .route(web::get().to(get_cities))
                .route(web::post().to(upsert_city)),
        )
        .service(
            web::resource("/cities/{id}")
                .route(web::get().to(get_city_by_id))
                .route(web::delete().to(delete_city)),
        )
        .service(
            web::resource("/stations")
                .route(web::get().to(get_stations))
                .route(web::post().to(upsert_station)),
        )
        .service(
            web::resource("/stations/{id}")
                .route(web::get().to(get_station_by_id))
                .route(web::delete().to(delete_station)),
        )
}

// ===== New DTO-based endpoints =====

#[derive(serde::Deserialize)]
struct CityQuery {
    name: Option<String>,
    province: Option<String>,
}

async fn get_cities(state: web::Data<AppState>, q: web::Query<CityQuery>) -> HttpResponse {
    let result = if let Some(name) = &q.name {
        state.city_repo.find_by_name(name).await
    } else if let Some(province) = &q.province {
        state
            .city_repo
            .find_by_province(ProvinceName::from(province.clone()))
            .await
    } else {
        state.city_repo.load().await
    };

    match result {
        Ok(cities) => {
            let list: Vec<CityDTO> = cities
                .into_iter()
                .map(|c| CityDTO {
                    id: c.get_id().map(Into::into),
                    name: c.name().to_string(),
                    province: c.province().to_string(),
                })
                .collect();
            HttpResponse::Ok().json(list)
        }
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}

async fn get_city_by_id(state: web::Data<AppState>, path: web::Path<u64>) -> HttpResponse {
    let id = CityId::from(path.into_inner());
    match state.city_repo.find(id).await {
        Ok(Some(c)) => HttpResponse::Ok().json(CityDTO {
            id: c.get_id().map(Into::into),
            name: c.name().to_string(),
            province: c.province().to_string(),
        }),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}

async fn upsert_city(state: web::Data<AppState>, payload: web::Json<CityDTO>) -> HttpResponse {
    let dto = payload.into_inner();
    let mut city = City::new(
        dto.id.map(Into::into),
        CityName::from(dto.name),
        ProvinceName::from(dto.province),
    );
    match state.city_repo.save(&mut city).await {
        Ok(id) => HttpResponse::Ok().json(SaveResultDTO { id: id.into() }),
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}

async fn delete_city(state: web::Data<AppState>, path: web::Path<u64>) -> HttpResponse {
    let id = CityId::from(path.into_inner());
    match state.city_repo.find(id).await {
        Ok(Some(city)) => match state.city_repo.remove(city).await {
            Ok(()) => HttpResponse::Ok().finish(),
            Err(_e) => HttpResponse::InternalServerError().finish(),
        },
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(serde::Deserialize)]
struct StationQuery {
    #[serde(rename = "cityId")]
    city_id: Option<u64>,
    name: Option<String>,
}

async fn get_stations(state: web::Data<AppState>, q: web::Query<StationQuery>) -> HttpResponse {
    let result: Result<Vec<Station>, _> = if let Some(name) = &q.name {
        match state.station_repo.find_by_name(name).await {
            Ok(Some(s)) => Ok(vec![s]),
            Ok(None) => Ok(vec![]),
            Err(e) => Err(e),
        }
    } else if let Some(city_id) = q.city_id {
        state.station_repo.find_by_city(CityId::from(city_id)).await
    } else {
        state.station_repo.load().await
    };

    match result {
        Ok(stations) => {
            let list: Vec<StationDTO> = stations
                .into_iter()
                .map(|s| StationDTO {
                    id: s.get_id().map(Into::into),
                    name: s.name().to_string(),
                    city_id: s.city_id().into(),
                })
                .collect();
            HttpResponse::Ok().json(list)
        }
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}

async fn get_station_by_id(state: web::Data<AppState>, path: web::Path<u64>) -> HttpResponse {
    let id = StationId::from(path.into_inner());
    match state.station_repo.find(id).await {
        Ok(Some(s)) => HttpResponse::Ok().json(StationDTO {
            id: s.get_id().map(Into::into),
            name: s.name().to_string(),
            city_id: s.city_id().into(),
        }),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}

async fn upsert_station(
    state: web::Data<AppState>,
    payload: web::Json<StationDTO>,
) -> HttpResponse {
    let dto = payload.into_inner();
    let mut station = Station::new(dto.id.map(Into::into), dto.name, CityId::from(dto.city_id));
    match state.station_repo.save(&mut station).await {
        Ok(id) => HttpResponse::Ok().json(SaveResultDTO { id: id.into() }),
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}

async fn delete_station(state: web::Data<AppState>, path: web::Path<u64>) -> HttpResponse {
    let id = StationId::from(path.into_inner());
    match state.station_repo.find(id).await {
        Ok(Some(station)) => match state.station_repo.remove(station).await {
            Ok(()) => HttpResponse::Ok().finish(),
            Err(_e) => HttpResponse::InternalServerError().finish(),
        },
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_e) => HttpResponse::InternalServerError().finish(),
    }
}
