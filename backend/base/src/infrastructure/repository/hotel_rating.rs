use crate::domain::model::hotel::{HotelId, HotelRating, HotelRatingId, Rating};
use crate::domain::model::user::UserId;
use crate::domain::repository::hotel_rating::HotelRatingRepository;
use crate::domain::{DbId, Identifiable, Repository, RepositoryError};
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use rust_decimal::Decimal;
use sea_orm::{ActiveValue, DatabaseConnection, EntityTrait, QueryFilter, Select};
use sea_orm::{ColumnTrait, DatabaseBackend, FromQueryResult, Statement};
use tracing::{error, instrument};

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

    #[instrument(skip(self))]
    async fn get_hotel_rating(&self, hotel_id: HotelId) -> Result<Option<Rating>, RepositoryError> {
        #[derive(Debug, FromQueryResult)]
        struct RatingQueryResult {
            rating: Option<Decimal>,
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
            .inspect_err(|e| error!("Failed to query hotel rating: {}", e))
            .context(format!(
                "Failed to calculate hotel rating for hotel id: {}",
                hotel_id
            ))?;

        rating_query_result
            .map(|r| Rating::try_from(r.rating.unwrap_or(Decimal::ZERO)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::hotel::{HotelRating, HotelRatingId, Rating};
    use crate::domain::model::user::UserId;
    use chrono::Local;
    use rust_decimal::prelude::FromPrimitive;
    use sea_orm::prelude::DateTimeWithTimeZone;
    use sea_orm::{ConnectionTrait, Database, Statement};

    async fn setup_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();

        // 建表
        db.execute(Statement::from_string(
            db.get_database_backend(),
            r#"
            CREATE TABLE hotel_rating (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id INTEGER NOT NULL,
                hotel_id INTEGER NOT NULL,
                time BIGINT NOT NULL,
                rating REAL NOT NULL,
                text TEXT NOT NULL
            );
            "#,
        ))
        .await
        .unwrap();

        db
    }

    fn sample_rating(user_id: i32, hotel_id: i32, rating: Decimal, text: &str) -> HotelRating {
        HotelRating::new(
            None,
            UserId::from_db_value(user_id).unwrap(),
            HotelId::from_db_value(hotel_id).unwrap(),
            DateTimeWithTimeZone::from(Local::now()),
            Rating::try_from(rating).unwrap(),
            text.to_string(),
        )
    }

    #[tokio::test]
    async fn test_save_and_find_success() {
        let db = setup_db().await;
        let repo = HotelRatingRepositoryImpl::new(db);

        let mut rating = sample_rating(1, 101, Decimal::from_f64(5.0).unwrap(), "Great stay!");
        let id = repo.save(&mut rating).await.unwrap();
        assert!(id.to_db_value() > 0);

        let found = repo.find(id).await.unwrap().unwrap();
        assert_eq!(found.text(), "Great stay!");
        assert_eq!(
            found.rating(),
            Rating::try_from(Decimal::from_f64(5.0).unwrap()).unwrap()
        );
    }

    #[tokio::test]
    async fn test_find_not_found() {
        let db = setup_db().await;
        let repo = HotelRatingRepositoryImpl::new(db);

        let not_found = repo
            .find(HotelRatingId::from_db_value(999).unwrap())
            .await
            .unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_update_success() {
        let db = setup_db().await;
        let repo = HotelRatingRepositoryImpl::new(db);

        // 1) 插入一条新的 rating（sample_rating 返回 None id -> INSERT）
        let mut rating = sample_rating(1, 101, Decimal::from_f64(4.0).unwrap(), "Nice hotel");
        let id = repo.save(&mut rating).await.unwrap();

        // 2) 从 DB 读取已保存的记录（保证我们拿到数据库真实的值）
        let loaded = repo.find(id).await.unwrap().unwrap();

        // 3) 用相同的 user_id/hotel_id/time/rating，但新的 text 构造一个新的 HotelRating（带上 id）
        let mut updated = HotelRating::new(
            Some(id), // 带上已存在的 id，save() 将走 update 分支
            loaded.user_id(),
            loaded.hotel_id(),
            loaded.time(),
            loaded.rating(),
            "Updated hotel".to_string(), // 新内容
        );

        // 4) 调用 save -> 应当成功更新现有行
        repo.save(&mut updated).await.unwrap();

        // 5) 验证
        let final_r = repo.find(id).await.unwrap().unwrap();
        assert_eq!(final_r.text(), "Updated hotel");
        assert_eq!(final_r.get_id(), Some(id));
    }

    #[tokio::test]
    async fn test_remove_success() {
        let db = setup_db().await;
        let repo = HotelRatingRepositoryImpl::new(db);

        let mut rating = sample_rating(1, 101, Decimal::from_f64(3.0).unwrap(), "Okay stay");
        let id = repo.save(&mut rating).await.unwrap();

        repo.remove(rating).await.unwrap();
        let after_remove = repo.find(id).await.unwrap();
        assert!(after_remove.is_none());
    }

    #[tokio::test]
    async fn test_get_comments_by_hotel_id_success() {
        let db = setup_db().await;
        let repo = HotelRatingRepositoryImpl::new(db);

        let mut r1 = sample_rating(1, 200, Decimal::from_f64(5.0).unwrap(), "Excellent");
        let mut r2 = sample_rating(2, 200, Decimal::from_f64(4.0).unwrap(), "Good");
        repo.save(&mut r1).await.unwrap();
        repo.save(&mut r2).await.unwrap();

        let comments = repo
            .get_comments_by_hotel_id(HotelId::from_db_value(200).unwrap())
            .await
            .unwrap();
        assert_eq!(comments.len(), 2);
    }

    #[tokio::test]
    async fn test_get_comments_by_hotel_id_not_found() {
        let db = setup_db().await;
        let repo = HotelRatingRepositoryImpl::new(db);

        let comments = repo
            .get_comments_by_hotel_id(HotelId::from_db_value(999).unwrap())
            .await
            .unwrap();
        assert!(comments.is_empty());
    }

    #[tokio::test]
    async fn test_get_comments_by_user_id_success() {
        let db = setup_db().await;
        let repo = HotelRatingRepositoryImpl::new(db);

        let mut r1 = sample_rating(10, 300, Decimal::from_f64(5.0).unwrap(), "Great");
        let mut r2 = sample_rating(10, 301, Decimal::from_f64(4.0).unwrap(), "Nice");
        repo.save(&mut r1).await.unwrap();
        repo.save(&mut r2).await.unwrap();

        let comments = repo
            .get_comments_by_user_id(UserId::from_db_value(10).unwrap())
            .await
            .unwrap();
        assert_eq!(comments.len(), 2);
    }

    #[tokio::test]
    async fn test_get_comments_by_user_id_not_found() {
        let db = setup_db().await;
        let repo = HotelRatingRepositoryImpl::new(db);

        let comments = repo
            .get_comments_by_user_id(UserId::from_db_value(123).unwrap())
            .await
            .unwrap();
        assert!(comments.is_empty());
    }

    #[tokio::test]
    async fn test_get_hotel_rating_success() {
        let db = setup_db().await;
        let repo = HotelRatingRepositoryImpl::new(db);

        let mut r1 = sample_rating(1, 400, Decimal::from_f64(5.0).unwrap(), "Awesome");
        let mut r2 = sample_rating(2, 400, Decimal::from_f64(3.0).unwrap(), "Okay");
        repo.save(&mut r1).await.unwrap();
        repo.save(&mut r2).await.unwrap();

        let avg = repo
            .get_hotel_rating(HotelId::from_db_value(400).unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            avg,
            Rating::try_from(Decimal::from_f64(4.0).unwrap()).unwrap()
        ); // 平均 (5+3)/2=4
    }

    #[tokio::test]
    async fn test_get_hotel_rating_not_found() {
        let db = setup_db().await;
        let repo = HotelRatingRepositoryImpl::new(db);

        let avg = repo
            .get_hotel_rating(HotelId::from_db_value(888).unwrap())
            .await
            .unwrap();
        // 没数据时 repository 逻辑会返回 0
        assert_eq!(avg.unwrap(), Rating::try_from(Decimal::ZERO).unwrap());
    }
}
