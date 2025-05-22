use crate::domain::model::hotel::{HotelId, HotelRating, HotelRatingId, Rating};
use crate::domain::model::user::UserId;
use crate::domain::repository::hotel_rating::HotelRatingRepository;
use crate::domain::{DbId, Identifiable, Repository, RepositoryError};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use rust_decimal::Decimal;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, QueryFilter, Select};
use sea_orm::{ColumnTrait, DatabaseBackend, FromQueryResult, Statement};

impl_db_id_from_u64!(HotelRatingId, i32, "hotel rating");

pub struct HotelRatingDataConverter;

impl HotelRatingDataConverter {
    pub fn make_from_do(
        hotel_rating_do: crate::models::hotel_rating::Model,
    ) -> Result<HotelRating, anyhow::Error> {
        Ok(HotelRating::new(
            Some(HotelRatingId::from_db_value(hotel_rating_do.id)?),
            UserId::from_db_value(hotel_rating_do.user_id)?,
            HotelId::from_db_value(hotel_rating_do.hotel_id)?,
            hotel_rating_do.time,
            Rating::try_from(hotel_rating_do.rating)
                .map_err(|e| anyhow!("Invalid rating {}: {}", hotel_rating_do.rating, e))?,
            hotel_rating_do.text,
        ))
    }

    pub fn transform_to_do(hotel_rating: &HotelRating) -> crate::models::hotel_rating::ActiveModel {
        let mut model = crate::models::hotel_rating::ActiveModel {
            id: ActiveValue::NotSet,
            user_id: ActiveValue::Set(hotel_rating.user_id().to_db_value()),
            hotel_id: ActiveValue::Set(hotel_rating.hotel_id().to_db_value()),
            time: ActiveValue::Set(hotel_rating.time()),
            rating: ActiveValue::Set(hotel_rating.rating().into()),
            text: ActiveValue::Set(hotel_rating.text().to_string()),
        };

        if let Some(id) = hotel_rating.get_id() {
            model.id = ActiveValue::Set(id.to_db_value());
        }

        model
    }
}

pub struct HotelRatingRepositoryImpl {
    db: DatabaseConnection,
}

impl HotelRatingRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn query_hotel_comments(
        &self,
        builder: impl FnOnce(
            Select<crate::models::hotel_rating::Entity>,
        ) -> Select<crate::models::hotel_rating::Entity>,
    ) -> Result<Vec<HotelRating>, RepositoryError> {
        let model_list = builder(crate::models::hotel_rating::Entity::find())
            .all(&self.db)
            .await
            .context("Failed to query hotel comments")?;

        let mut result = Vec::with_capacity(model_list.len());

        for model in model_list {
            result.push(HotelRatingDataConverter::make_from_do(model)?);
        }

        Ok(result)
    }
}

#[async_trait]
impl Repository<HotelRating> for HotelRatingRepositoryImpl {
    async fn find(&self, id: HotelRatingId) -> Result<Option<HotelRating>, RepositoryError> {
        let hotel_rating_do = crate::models::hotel_rating::Entity::find_by_id(id.to_db_value())
            .one(&self.db)
            .await
            .context(format!(
                "Failed to find hotel rating for hotel rating id: {}",
                id
            ))?;

        hotel_rating_do
            .map(HotelRatingDataConverter::make_from_do)
            .transpose()
            .map_err(RepositoryError::ValidationError)
    }

    async fn remove(&self, aggregate: HotelRating) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            crate::models::hotel_rating::Entity::delete_by_id(id.to_db_value())
                .exec(&self.db)
                .await
                .context(format!(
                    "Failed to delete hotel rating for hotel rating id: {}",
                    id
                ))?;
        }

        Ok(())
    }

    async fn save(&self, aggregate: &mut HotelRating) -> Result<HotelRatingId, RepositoryError> {
        let model = HotelRatingDataConverter::transform_to_do(aggregate);

        if let Some(id) = aggregate.get_id() {
            crate::models::hotel_rating::Entity::update(model)
                .exec(&self.db)
                .await
                .context(format!("Failed to update hotel rating with id: {}", id))?;

            Ok(id)
        } else {
            let result = crate::models::hotel_rating::Entity::insert(model)
                .exec(&self.db)
                .await
                .context("Failed to insert hotel rating")?;

            let id = HotelRatingId::from_db_value(result.last_insert_id)?;

            aggregate.set_id(id);

            Ok(id)
        }
    }
}

#[async_trait]
impl HotelRatingRepository for HotelRatingRepositoryImpl {
    async fn get_comments_by_hotel_id(
        &self,
        hotel_id: HotelId,
    ) -> Result<Vec<HotelRating>, RepositoryError> {
        self.query_hotel_comments(|q| {
            q.filter(crate::models::hotel_rating::Column::HotelId.eq(hotel_id.to_db_value()))
        })
        .await
    }

    async fn get_comments_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Vec<HotelRating>, RepositoryError> {
        self.query_hotel_comments(|q| {
            q.filter(crate::models::hotel_rating::Column::UserId.eq(user_id.to_db_value()))
        })
        .await
    }

    async fn get_hotel_rating(&self, hotel_id: HotelId) -> Result<Option<Rating>, RepositoryError> {
        #[derive(Debug, FromQueryResult)]
        struct RatingQueryResult {
            rating: Decimal,
        }

        let rating_query_result =
            RatingQueryResult::find_by_statement(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                r#"SELECT
    AVG("hotel_rating"."rating") AS "rating"
FROM "hotel_rating"
WHERE "hotel_rating"."hotel_id" = $1"#,
                [hotel_id.to_db_value().into()],
            ))
            .one(&self.db)
            .await
            .context(format!(
                "Failed to calculate hotel rating for hotel id: {}",
                hotel_id
            ))?;

        rating_query_result
            .map(|r| Rating::try_from(r.rating))
            .transpose()
            .map_err(|e| {
                RepositoryError::ValidationError(anyhow!(
                    "Invalid rating for hotel id {}: {}",
                    hotel_id,
                    e
                ))
            })
    }
}
