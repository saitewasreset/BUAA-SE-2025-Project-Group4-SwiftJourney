use crate::domain::model::city::{City, CityId, CityName, ProvinceName};
use crate::domain::repository::city::CityRepository;
use crate::domain::{DbId, Identifiable, Repository, RepositoryError};
use crate::infrastructure::repository::transform_list;
use anyhow::Context;
use async_trait::async_trait;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, QueryFilter};
use sea_orm::{ColumnTrait, Select};

pub struct CityRepositoryImpl {
    db: DatabaseConnection,
}

pub struct CityDataConverter;

impl_db_id_from_u64!(CityId, i32, "city");

impl CityDataConverter {
    pub fn make_from_do(city_do: crate::models::city::Model) -> anyhow::Result<City> {
        let city_id = CityId::from_db_value(city_do.id)?;
        let name = city_do.name.into();
        let province = city_do.province.into();

        Ok(City::new(Some(city_id), name, province))
    }

    pub fn transform_to_do(city: City) -> crate::models::city::ActiveModel {
        let mut model = crate::models::city::ActiveModel {
            id: ActiveValue::NotSet,
            name: ActiveValue::Set(city.name().to_string()),
            province: ActiveValue::Set(city.province().to_string()),
        };

        if let Some(id) = city.get_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        model
    }
}

#[async_trait]
impl Repository<City> for CityRepositoryImpl {
    async fn find(&self, id: CityId) -> Result<Option<City>, RepositoryError> {
        let result = crate::models::city::Entity::find_by_id(id.to_db_value())
            .one(&self.db)
            .await
            .context(format!("Failed to find city with id: {}", id.to_db_value()))?;

        result
            .map(CityDataConverter::make_from_do)
            .transpose()
            .context(format!(
                "Failed to validate city with id: {}",
                id.to_db_value()
            ))
            .map_err(RepositoryError::ValidationError)
    }

    async fn remove(&self, aggregate: City) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            let id = id.to_db_value();

            crate::models::city::Entity::delete_by_id(id)
                .exec(&self.db)
                .await
                .context(format!("Failed to remove city with id: {}", id))?;
        }

        Ok(())
    }

    async fn save(&self, aggregate: &mut City) -> Result<CityId, RepositoryError> {
        let city_do = CityDataConverter::transform_to_do(aggregate.clone());
        let result = crate::models::city::Entity::insert(city_do)
            .on_conflict(
                OnConflict::column(crate::models::city::Column::Id)
                    .update_columns([
                        crate::models::city::Column::Name,
                        crate::models::city::Column::Province,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .context(format!(
                "Failed to save city with id: {:?}",
                aggregate.get_id()
            ))?;

        let id = result.last_insert_id as u64;
        aggregate.set_id(id.into());

        Ok(id.into())
    }
}

#[async_trait]
impl CityRepository for CityRepositoryImpl {
    async fn load(&self) -> Result<Vec<City>, RepositoryError> {
        self.query_cities(|f| f).await
    }

    async fn find_by_name(&self, city_name: CityName) -> Result<Vec<City>, RepositoryError> {
        self.query_cities(|f| f.filter(crate::models::city::Column::Name.eq(city_name.to_string())))
            .await
    }

    async fn find_by_province(
        &self,
        province_name: ProvinceName,
    ) -> Result<Vec<City>, RepositoryError> {
        self.query_cities(|f| {
            f.filter(crate::models::city::Column::Province.eq(province_name.to_string()))
        })
        .await
    }
}

impl CityRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        CityRepositoryImpl { db }
    }

    pub async fn query_cities(
        &self,
        builder: impl FnOnce(Select<crate::models::city::Entity>) -> Select<crate::models::city::Entity>,
    ) -> Result<Vec<City>, RepositoryError> {
        let query = builder(crate::models::city::Entity::find());
        let stations = query
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;
        transform_list(stations, CityDataConverter::make_from_do, |x| x.id)
            .context("Failed to transform city list")
            .map_err(RepositoryError::ValidationError)
    }
}
