//! 个人信息仓储实现模块
//!
//! 本模块提供了个人信息实体的数据库仓储实现，包括：
//! - 个人信息数据的数据库操作（增删改查）
//! - 领域模型与数据库模型之间的转换
//! - 变更追踪与聚合管理

use std::sync::{Arc, Mutex};

use crate::domain::model::personal_info::{PersonalInfo, PersonalInfoId, PreferredSeatLocation};
use crate::domain::model::user::{IdentityCardId, RealName};
use crate::domain::repository::personal_info::PersonalInfoRepository;
use crate::domain::service::{AggregateManagerImpl, DiffInfo};
use crate::domain::{
    DbId, DbRepositorySupport, DiffType, Identifiable, MultiEntityDiff, RepositoryError, TypedDiff,
};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

impl_db_id_from_u64!(PersonalInfoId, i32, "personal info");

/// 个人信息仓储实现结构体
///
/// 负责个人信息实体的持久化操作，包含：
/// - 数据库连接
/// - 聚合管理器（用于跟踪实体变更）
pub struct PersonalInfoRepositoryImpl {
    db: DatabaseConnection,
    aggregate_manager: Arc<Mutex<AggregateManagerImpl<PersonalInfo>>>,
}

/// 个人信息数据转换器
///
/// 提供领域模型(`PersonalInfo`)与数据库模型之间的双向转换功能
pub struct PersonalInfoDataConverter;

impl PersonalInfoDataConverter {
    pub fn transform_to_do(personal_info: PersonalInfo) -> crate::models::person_info::ActiveModel {
        let mut model = crate::models::person_info::ActiveModel {
            id: ActiveValue::NotSet,
            uuid: ActiveValue::Set(personal_info.uuid().to_owned()),
            name: ActiveValue::Set(personal_info.name().to_string()),
            identity_card: ActiveValue::Set(personal_info.identity_card_id().to_string()),
            preferred_seat_location: ActiveValue::Set(
                personal_info.preferred_seat_location().to_string(),
            ),
            user_id: ActiveValue::Set(personal_info.user_id().to_db_value()),
            is_default: ActiveValue::Set(personal_info.is_default()),
        };

        if let Some(id) = personal_info.get_id() {
            model.id = ActiveValue::Set(u64::from(id) as i32);
        }

        model
    }

    pub fn make_from_do(
        user_do: crate::models::person_info::Model,
    ) -> anyhow::Result<PersonalInfo> {
        let id = PersonalInfoId::from(user_do.id as u64);
        let uuid = user_do.uuid;
        let name = RealName::try_from(user_do.name)?;
        let identity_card_id = IdentityCardId::try_from(user_do.identity_card)?;

        let preferred_seat_location = PreferredSeatLocation::try_from(
            user_do
                .preferred_seat_location
                .chars()
                .next()
                .ok_or_else(|| {
                    anyhow::anyhow!("Inconsistent: preferred seat location should not be null")
                })?,
        )
        .map_err(|e| anyhow!("Failed to parse preferred seat location: {}", e))?;
        let user_id = (user_do.user_id as u64).into();

        let mut personal_info = PersonalInfo::new(
            Some(id),
            uuid,
            name,
            identity_card_id,
            preferred_seat_location,
            user_id,
        );
        personal_info.set_default(user_do.is_default);

        Ok(personal_info)
    }
}

impl PersonalInfoRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        let detect_changes_fn = |diff: DiffInfo<PersonalInfo>| {
            let mut result = MultiEntityDiff::new();

            let diff_type = DiffType::from(&diff);

            let old = diff.old;
            let new = diff.new;

            result.add_change(TypedDiff::new(diff_type, old, new));

            result
        };

        PersonalInfoRepositoryImpl {
            db,
            aggregate_manager: Arc::new(Mutex::new(AggregateManagerImpl::new(Box::new(
                detect_changes_fn,
            )))),
        }
    }
}

#[async_trait]
impl DbRepositorySupport<PersonalInfo> for PersonalInfoRepositoryImpl {
    type Manager = AggregateManagerImpl<PersonalInfo>;

    fn get_aggregate_manager(&self) -> Arc<Mutex<Self::Manager>> {
        Arc::clone(&self.aggregate_manager)
    }

    /// 插入新个人信息到数据库
    ///
    /// # Arguments
    /// * `aggregate` - 要插入的个人信息领域模型
    ///
    /// # Returns
    /// 返回插入后的个人信息ID
    ///
    /// # Errors
    /// 当数据库操作失败时返回错误
    async fn on_insert(&self, aggregate: PersonalInfo) -> Result<PersonalInfoId, RepositoryError> {
        let id = aggregate.get_id();

        let model = PersonalInfoDataConverter::transform_to_do(aggregate);

        let result_model = model
            .insert(&self.db)
            .await
            .context(format!("Failed to insert personal info: {:?}", id))
            .map_err(RepositoryError::Db)?;

        Ok(PersonalInfoId::from(result_model.id as u64))
    }

    /// 根据ID查询个人信息
    ///
    /// # Arguments
    /// * `id` - 个人信息ID
    ///
    /// # Errors
    /// 当数据库操作或数据验证失败时返回错误
    ///
    /// # Returns
    /// 返回查询到的个人信息领域模型（如果存在）
    async fn on_select(&self, id: PersonalInfoId) -> Result<Option<PersonalInfo>, RepositoryError> {
        let id = id.to_db_value();

        let personal_info_do = crate::models::person_info::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .context(format!("Failed to find personal info with id: {}", id))
            .map_err(RepositoryError::Db)?;

        personal_info_do
            .map(PersonalInfoDataConverter::make_from_do)
            .transpose()
            .context(format!("Failed to validate personal info with id: {}", id))
            .map_err(RepositoryError::ValidationError)
    }

    /// 更新个人信息变更到数据库
    ///
    /// # Arguments
    /// * `diff` - 包含变更信息的差异对象
    ///
    /// # Errors
    /// 当数据库操作失败时返回错误
    async fn on_update(&self, diff: MultiEntityDiff) -> Result<(), RepositoryError> {
        for changes in diff.get_changes::<PersonalInfo>() {
            match changes.diff_type {
                DiffType::Unchanged => {}
                DiffType::Added => {
                    let new_value = changes.new_value.unwrap();
                    let id = new_value.get_id();
                    PersonalInfoDataConverter::transform_to_do(new_value)
                        .insert(&self.db)
                        .await
                        .context(format!("Failed to add personal info with id: {:?}", id))
                        .map_err(RepositoryError::Db)?;
                }
                DiffType::Modified => {
                    let new_value = changes.new_value.unwrap();
                    let id = new_value.get_id();

                    PersonalInfoDataConverter::transform_to_do(new_value)
                        .update(&self.db)
                        .await
                        .context(format!("Failed to update personal info with id: {:?}", id))
                        .map_err(RepositoryError::Db)?;
                }
                DiffType::Removed => {
                    if let Some(id) = changes.old_value.unwrap().get_id() {
                        let id = id.to_db_value();
                        crate::models::person_info::Entity::delete_by_id(id)
                            .exec(&self.db)
                            .await
                            .context(format!("Failed to delete personal info with id: {:?}", id))
                            .map_err(RepositoryError::Db)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 从数据库删除个人信息
    ///
    /// # Arguments
    /// * `aggregate` - 要删除的个人信息领域模型
    ///
    /// # Errors
    /// 当数据库操作失败时返回错误
    async fn on_delete(&self, aggregate: PersonalInfo) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            let id = id.to_db_value();

            crate::models::person_info::Entity::delete_by_id(id)
                .exec(&self.db)
                .await
                .context(format!("Failed to delete personal info with id: {}", id))
                .map_err(RepositoryError::Db)?;
        }

        Ok(())
    }
}

#[async_trait]
impl PersonalInfoRepository for PersonalInfoRepositoryImpl {
    /// 根据用户ID查询多个个人信息
    ///
    /// # Arguments
    /// * `user_id` - 用户ID
    ///
    /// # Returns
    /// * `Ok(Vec<PersonalInfo>)` - 查询成功返回个人信息列表（可能为空）
    /// * `Err(RepositoryError)` - 查询失败
    async fn find_by_user_id(
        &self,
        user_id: crate::domain::model::user::UserId,
    ) -> Result<Vec<PersonalInfo>, RepositoryError> {
        let user_id_value = user_id.to_db_value();

        let personal_info_items = crate::models::person_info::Entity::find()
            .filter(crate::models::person_info::Column::UserId.eq(user_id_value))
            .all(&self.db)
            .await
            .context(format!(
                "Failed to find personal info with user id: {}",
                user_id_value
            ))
            .map_err(RepositoryError::Db)?;

        let mut result = Vec::new();
        for item in personal_info_items {
            match PersonalInfoDataConverter::make_from_do(item) {
                Ok(info) => result.push(info),
                Err(err) => {
                    // 记录错误但继续处理其他记录
                    tracing::warn!("Failed to convert personal info: {}", err);
                }
            }
        }

        Ok(result)
    }

    /// 根据用户ID和身份证号查询个人信息
    ///
    /// # Arguments
    /// * `user_id` - 用户ID
    /// * `identity_card_id` - 身份证号
    ///
    /// # Returns
    /// * `Ok(Some(PersonalInfo))` - 查询成功且找到个人信息
    /// * `Ok(None)` - 查询成功但未找到个人信息
    /// * `Err(RepositoryError)` - 查询失败
    async fn find_by_user_id_and_identity_card(
        &self,
        user_id: crate::domain::model::user::UserId,
        identity_card_id: crate::domain::model::user::IdentityCardId,
    ) -> Result<Option<PersonalInfo>, RepositoryError> {
        let user_id_value = user_id.to_db_value();
        let identity_card_value = identity_card_id.to_string();

        let personal_info_do = crate::models::person_info::Entity::find()
            .filter(crate::models::person_info::Column::UserId.eq(user_id_value))
            .filter(crate::models::person_info::Column::IdentityCard.eq(&identity_card_value))
            .one(&self.db)
            .await
            .context(format!(
                "Failed to find personal info with user id: {} and identity card: {}",
                user_id_value, identity_card_value
            ))
            .map_err(RepositoryError::Db)?;

        personal_info_do
            .map(PersonalInfoDataConverter::make_from_do)
            .transpose()
            .context(format!(
                "Failed to validate personal info with user id: {} and identity card: {}",
                user_id_value, identity_card_value
            ))
            .map_err(RepositoryError::ValidationError)
    }
}
