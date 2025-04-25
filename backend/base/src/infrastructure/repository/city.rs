use crate::domain::model::city::{City, CityId, CityName, ProvinceName};
use crate::domain::repository::city::CityRepository;
use crate::domain::{Identifiable, Repository, RepositoryError};
use crate::infrastructure::repository::transform_list;
use anyhow::Context;
use async_trait::async_trait;
use sea_orm::ColumnTrait;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, QueryFilter};

pub struct CityRepositoryImpl {
    db: DatabaseConnection,
}

pub struct CityDataConverter;

impl CityDataConverter {
    pub fn make_from_do(city_do: crate::models::city::Model) -> anyhow::Result<City> {
        let city_id = city_do.id.try_into()?;
        let name = city_do.name.into();
        let province = city_do.province.into();

        Ok(City::new(Some(city_id), name, province))
    }

    pub fn transform_to_do(city: City) -> crate::models::city::ActiveModel {
        crate::models::city::ActiveModel {
            id: ActiveValue::NotSet,
            name: ActiveValue::Set(city.name().to_string()),
            province: ActiveValue::Set(city.province().to_string()),
        }
    }
}

#[async_trait]
impl Repository<City> for CityRepositoryImpl {
    async fn find(&self, id: CityId) -> Result<Option<City>, RepositoryError> {
        let result = crate::models::city::Entity::find_by_id(u64::from(id) as i32)
            .one(&self.db)
            .await
            .context(format!("Failed to find city with id: {}", u64::from(id)))?;

        result
            .map(CityDataConverter::make_from_do)
            .transpose()
            .context(format!(
                "Failed to validate city with id: {}",
                u64::from(id)
            ))
            .map_err(RepositoryError::ValidationError)
    }

    async fn remove(&self, aggregate: City) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            let id = u64::from(id) as i32;

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
        let do_list = crate::models::city::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;

        transform_list(do_list, CityDataConverter::make_from_do, |x| x.id)
            .context("Failed to transform city list")
            .map_err(RepositoryError::ValidationError)
    }

    async fn find_by_name(&self, city_name: CityName) -> Result<Vec<City>, RepositoryError> {
        let do_list = crate::models::city::Entity::find()
            .filter(crate::models::city::Column::Name.eq(city_name.to_string()))
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;

        transform_list(do_list, CityDataConverter::make_from_do, |x| x.id)
            .context("Failed to transform city list")
            .map_err(RepositoryError::ValidationError)
    }

    async fn find_by_province(
        &self,
        province_name: ProvinceName,
    ) -> Result<Vec<City>, RepositoryError> {
        let do_list = crate::models::city::Entity::find()
            .filter(crate::models::city::Column::Province.eq(province_name.to_string()))
            .all(&self.db)
            .await
            .map_err(|e| RepositoryError::Db(e.into()))?;

        transform_list(do_list, CityDataConverter::make_from_do, |x| x.id)
            .context("Failed to transform city list")
            .map_err(RepositoryError::ValidationError)
    }
}
