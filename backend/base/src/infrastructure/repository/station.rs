use crate::domain::DbId;
use crate::domain::model::city::CityId;
use crate::domain::model::station::{Station, StationId};
use crate::domain::repository::station::StationRepository;
use crate::domain::{Identifiable, Repository, RepositoryError};
use crate::infrastructure::repository::transform_list;
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, QueryFilter, TransactionTrait};
use sea_orm::{ColumnTrait, Select};
use shared::data::StationData;
use std::collections::HashMap;

pub struct StationRepositoryImpl {
    db: DatabaseConnection,
}

impl_db_id_from_u64!(StationId, i32, "station");

pub struct StationDataConverter;

impl StationDataConverter {
    pub fn make_from_do(station_do: crate::models::station::Model) -> anyhow::Result<Station> {
        let station_id = StationId::from_db_value(station_do.id)?;
        let name = station_do.name;
        let city_id = CityId::from_db_value(station_do.city_id)?;

        Ok(Station::new(Some(station_id), name, city_id))
    }

    pub fn transform_to_do(station: &Station) -> crate::models::station::ActiveModel {
        let mut model = crate::models::station::ActiveModel {
            id: ActiveValue::NotSet,
            name: ActiveValue::Set(station.name().to_string()),
            city_id: ActiveValue::Set(station.city_id().to_db_value()),
        };

        if let Some(id) = station.get_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        model
    }
}

#[async_trait]
impl Repository<Station> for StationRepositoryImpl {
    async fn find(&self, id: StationId) -> Result<Option<Station>, RepositoryError> {
        let result = crate::models::station::Entity::find_by_id(u64::from(id) as i32)
            .one(&self.db)
            .await
            .context(format!("Failed to find station with id: {}", u64::from(id)))?;

        result
            .map(StationDataConverter::make_from_do)
            .transpose()
            .context(format!(
                "Failed to validate station with id: {}",
                u64::from(id)
            ))
            .map_err(RepositoryError::ValidationError)
    }

    async fn remove(&self, aggregate: Station) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            let id = u64::from(id) as i32;

            crate::models::station::Entity::delete_by_id(id)
                .exec(&self.db)
                .await
                .context(format!("Failed to remove station with id: {}", id))?;
        }

        Ok(())
    }

    async fn save(&self, aggregate: &mut Station) -> Result<StationId, RepositoryError> {
        let station_do = StationDataConverter::transform_to_do(&aggregate);
        let result = crate::models::station::Entity::insert(station_do)
            .on_conflict(
                OnConflict::column(crate::models::station::Column::Id)
                    .update_columns([
                        crate::models::station::Column::Name,
                        crate::models::station::Column::CityId,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .context(format!(
                "Failed to save station with id: {:?}",
                aggregate.get_id()
            ))?;

        let id = result.last_insert_id as u64;
        aggregate.set_id(id.into());

        Ok(id.into())
    }
}

#[async_trait]
impl StationRepository for StationRepositoryImpl {
    async fn load(&self) -> Result<Vec<Station>, RepositoryError> {
        self.query_stations(|q| q).await
    }

    async fn find_by_city(&self, city_id: CityId) -> Result<Vec<Station>, RepositoryError> {
        self.query_stations(|q| {
            q.filter(crate::models::station::Column::CityId.eq(u64::from(city_id) as i32))
        })
        .await
    }

    async fn find_by_name(&self, station_name: &str) -> Result<Option<Station>, RepositoryError> {
        let model = crate::models::station::Entity::find()
            .filter(crate::models::station::Column::Name.eq(station_name))
            .one(&self.db)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;

        let id = model.as_ref().map(|x| x.id);

        model
            .map(StationDataConverter::make_from_do)
            .transpose()
            .context(format!("Failed to validate station with id: {:?}", id))
            .map_err(RepositoryError::ValidationError)
    }

    async fn save_raw(&self, station_data: StationData) -> Result<(), RepositoryError> {
        let txn = self
            .db
            .begin()
            .await
            .context("failed to start transaction")?;

        let cities = crate::models::city::Entity::find()
            .all(&txn)
            .await
            .context("failed to load cities")?;

        let city_name_to_id = cities
            .into_iter()
            .map(|city| (city.name, city.id))
            .collect::<HashMap<_, _>>();

        for station in &station_data {
            if !city_name_to_id.contains_key(&station.name) {
                return Err(RepositoryError::InconsistentState(anyhow!(
                    "City not found: {}",
                    station.name
                )));
            }
        }

        let model_list = station_data
            .into_iter()
            .map(|station| {
                let city_id = city_name_to_id[&station.name];
                crate::models::station::ActiveModel {
                    id: ActiveValue::NotSet,
                    name: ActiveValue::Set(station.name),
                    city_id: ActiveValue::Set(city_id),
                }
            })
            .collect::<Vec<_>>();

        crate::models::station::Entity::insert_many(model_list)
            .on_conflict(
                OnConflict::column(crate::models::station::Column::Name)
                    .update_column(crate::models::station::Column::CityId)
                    .to_owned(),
            )
            .exec(&txn)
            .await
            .context("failed to save raw station data")?;

        txn.commit().await.context("failed to commit transaction")?;

        Ok(())
    }
}

impl StationRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        StationRepositoryImpl { db }
    }

    pub async fn query_stations(
        &self,
        builder: impl FnOnce(
            Select<crate::models::station::Entity>,
        ) -> Select<crate::models::station::Entity>,
    ) -> Result<Vec<Station>, RepositoryError> {
        let query = builder(crate::models::station::Entity::find());
        let stations = query
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;
        transform_list(stations, StationDataConverter::make_from_do, |x| x.id)
            .context("Failed to transform station list")
            .map_err(RepositoryError::ValidationError)
    }
}
