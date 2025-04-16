//! 用户仓储实现模块
//!
//! 本模块提供了用户实体的数据库仓储实现，包括：
//! - 用户数据的数据库操作（增删改查）
//! - 领域模型与数据库模型之间的转换
//! - 变更追踪与聚合管理

use crate::domain::model::password::HashedPassword;
use crate::domain::model::user::{
    Age, Gender, IdentityCardId, PasswordAttempts, Phone, User, UserId, UserInfo,
};
use crate::domain::repository::user::UserRepository;
use crate::domain::service::{AggregateManagerImpl, DiffInfo};
use crate::domain::{
    AggregateManager, DbRepositorySupport, DiffType, Identifiable, MultiEntityDiff, Repository,
    RepositoryError, TypedDiff,
};
use anyhow::{Context, anyhow};
use argon2::password_hash::PasswordHashString;
use email_address::EmailAddress;
use sea_orm::ColumnTrait;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, QueryFilter};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

/// 用户仓储实现结构体
///
/// 负责用户实体的持久化操作，包含：
/// - 数据库连接
/// - 聚合管理器（用于跟踪实体变更）
pub struct UserRepositoryImpl {
    db: DatabaseConnection,
    aggregate_manager: Arc<Mutex<AggregateManagerImpl<User>>>,
}

/// 用户数据转换器
///
/// 提供领域模型(`User`)与数据库模型之间的双向转换功能
pub struct UserDataConverter;

impl UserDataConverter {
    /// 将`Vec<u8>`解析为密码哈希字符串
    ///
    /// # Arguments
    /// * `bytes` - 存储的密码哈希字节
    ///
    /// # Errors
    /// 当字节不是有效的UTF-8字符串或不符合密码哈希格式时返回错误
    fn parse_bytes_to_password_hash_string(bytes: Vec<u8>) -> anyhow::Result<PasswordHashString> {
        let password_hash_string = String::from_utf8(bytes)?;
        PasswordHashString::new(&password_hash_string).map_err(|e| {
            anyhow!(
                "cannot parse password hash string: {} {}",
                password_hash_string,
                e
            )
        })
    }

    /// 将领域模型转换为数据库`Active Model`
    ///
    /// # Arguments
    /// * `user` - 用户领域模型
    ///
    /// # Returns
    /// 返回可用于数据库操作的SeaORM`Active Model`
    ///
    /// # Notes
    /// 用于password和payment_password的盐**必须相同**，具体地，数据库中只存储password的盐
    pub fn transform_to_do(user: User) -> crate::models::user::ActiveModel {
        let mut model = crate::models::user::ActiveModel {
            id: ActiveValue::NotSet,
            username: ActiveValue::Set(user.username().to_owned()),
            hashed_password: ActiveValue::Set(user.hashed_password().hashed_password.clone()),
            hashed_payment_password: ActiveValue::NotSet,
            salt: ActiveValue::Set(user.hashed_password().salt.clone().into()),
            wrong_payment_password_tried: ActiveValue::Set(u8::from(
                user.wrong_payment_password_tried(),
            ) as i32),
            gender: ActiveValue::Set(
                user.user_info()
                    .gender
                    .map(|gender| gender.to_owned().to_string()),
            ),
            age: ActiveValue::Set(user.user_info().age.map(|age| age.into())),
            phone: ActiveValue::Set(user.user_info().phone.to_string()),
            email: ActiveValue::Set(
                user.user_info()
                    .email
                    .as_ref()
                    .map(|email| email.to_owned().into()),
            ),
            name: ActiveValue::Set(user.user_info().name.to_string()),
            identity_card_id: ActiveValue::Set(user.user_info().identity_card_id.to_string()),
        };

        if let Some(id) = user.get_id() {
            model.id = ActiveValue::Set(u64::from(id) as i32);
        }

        if let Some(payment_password) = user.hashed_payment_password() {
            model.hashed_payment_password =
                ActiveValue::Set(Some(payment_password.hashed_password.clone()));
        }

        model
    }

    /// 从数据库模型创建领域模型
    ///
    /// # Arguments
    /// * `user_do` - 数据库中的用户`Data Object`
    ///
    /// # Errors
    /// 当数据转换或验证失败时返回错误
    ///
    /// # Returns
    /// 返回构建成功的用户领域模型
    pub fn make_from_do(user_do: crate::models::user::Model) -> anyhow::Result<User> {
        let user_id: UserId = (user_do.id as u64).into();
        let username: String = user_do.username;

        let wrong_payment_password_tried: PasswordAttempts =
            user_do.wrong_payment_password_tried.try_into()?;

        let name: String = user_do.name;

        let gender: Option<Gender> = user_do
            .gender
            .map(|gender| gender.as_str().try_into())
            .transpose()?;

        let age: Option<Age> = user_do.age.map(|age| age.try_into()).transpose()?;

        let phone: Phone = user_do.phone.try_into()?;

        let email: Option<EmailAddress> = user_do
            .email
            .map(|email| EmailAddress::from_str(email.as_str()))
            .transpose()?;

        let identity_card_id: IdentityCardId = user_do.identity_card_id.try_into()?;

        let user_info = UserInfo::new(name, gender, age, phone, email, identity_card_id);

        let salt = user_do.salt;

        let hashed_password = HashedPassword {
            hashed_password: user_do.hashed_password,
            salt: salt.clone().into(),
        };

        let hashed_payment_password = user_do.hashed_payment_password.map(|p| HashedPassword {
            hashed_password: p,
            salt: salt.clone().into(),
        });

        let user = User::new(
            Some(user_id),
            username,
            hashed_password,
            hashed_payment_password,
            wrong_payment_password_tried,
            user_info,
        );

        Ok(user)
    }
}

impl UserRepositoryImpl {
    /// 创建新的用户仓储实例
    ///
    /// # Arguments
    /// * `db` - 数据库连接
    ///
    /// # Returns
    /// 返回初始化好的用户仓储实例
    pub fn new(db: DatabaseConnection) -> Self {
        let detect_changes_fn = |diff: DiffInfo<User>| {
            let mut result = MultiEntityDiff::new();

            let old = diff.old;
            let new = diff.new;

            let diff_type = match (&old, &new) {
                (None, None) => DiffType::Unchanged,
                (None, Some(_)) => DiffType::Added,
                (Some(_), None) => DiffType::Removed,
                (Some(old_value), Some(new_value)) => {
                    if old_value == new_value {
                        DiffType::Unchanged
                    } else {
                        DiffType::Modified
                    }
                }
            };

            result.add_change(TypedDiff::new(diff_type, old, new));

            result
        };

        UserRepositoryImpl {
            db,
            aggregate_manager: Arc::new(Mutex::new(AggregateManagerImpl::new(Box::new(
                detect_changes_fn,
            )))),
        }
    }
}

impl DbRepositorySupport<User> for UserRepositoryImpl {
    type Manager = AggregateManagerImpl<User>;

    fn get_aggregate_manager(&self) -> Arc<Mutex<Self::Manager>> {
        Arc::clone(&self.aggregate_manager)
    }

    /// 插入新用户到数据库
    ///
    /// # Arguments
    /// * `aggregate` - 要插入的用户领域模型
    ///
    /// # Errors
    /// 当数据库操作失败时返回错误
    async fn on_insert(&self, aggregate: User) -> Result<(), RepositoryError> {
        let id = aggregate.get_id();

        let model = UserDataConverter::transform_to_do(aggregate);

        model
            .insert(&self.db)
            .await
            .context(format!("failed to insert user with id: {:?}", id))
            .map_err(RepositoryError::Db)?;

        Ok(())
    }

    /// 根据ID查询用户
    ///
    /// # Arguments
    /// * `id` - 用户ID
    ///
    /// # Errors
    /// 当数据库操作或数据验证失败时返回错误
    ///
    /// # Returns
    /// 返回查询到的用户领域模型（如果存在）
    async fn on_select(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        let id: i32 = u64::from(id) as i32;

        let user_do = crate::models::user::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .context(format!("failed to find user with id: {}", id))?;

        user_do
            .map(UserDataConverter::make_from_do)
            .transpose()
            .context(format!("failed to validation user with id: {}", id))
            .map_err(RepositoryError::ValidationError)
    }

    /// 更新用户变更到数据库
    ///
    /// # Arguments
    /// * `diff` - 包含变更信息的差异对象
    ///
    /// # Errors
    /// 当数据库操作失败时返回错误
    async fn on_update(&self, diff: MultiEntityDiff) -> Result<(), RepositoryError> {
        for changes in diff.get_changes::<User>() {
            match changes.diff_type {
                DiffType::Unchanged => {}
                DiffType::Added => {
                    let new_value = changes.new_value.unwrap();
                    let id = new_value.get_id();
                    UserDataConverter::transform_to_do(new_value)
                        .insert(&self.db)
                        .await
                        .context(format!("failed to update user with id: {:?}", id))
                        .map_err(RepositoryError::Db)?;
                }
                DiffType::Modified => {
                    let new_value = changes.new_value.unwrap();
                    let id = new_value.get_id();

                    UserDataConverter::transform_to_do(new_value)
                        .update(&self.db)
                        .await
                        .context(format!("failed to update user with id: {:?}", id))
                        .map_err(RepositoryError::Db)?;
                }
                DiffType::Removed => {
                    if let Some(id) = changes.old_value.unwrap().get_id() {
                        let id = u64::from(id) as i32;
                        crate::models::user::Entity::delete_by_id(id)
                            .exec(&self.db)
                            .await
                            .context(format!("failed to delete user with id: {:?}", id))
                            .map_err(RepositoryError::Db)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// 从数据库删除用户
    ///
    /// # Arguments
    /// * `aggregate` - 要删除的用户领域模型
    ///
    /// # Errors
    /// 当数据库操作失败时返回错误
    async fn on_delete(&self, aggregate: User) -> Result<(), RepositoryError> {
        if let Some(id) = aggregate.get_id() {
            let id = u64::from(id) as i32;

            crate::models::user::Entity::delete_by_id(id)
                .exec(&self.db)
                .await
                .context(format!("failed to delete user with id: {}", id))
                .map_err(RepositoryError::Db)?;
        }

        Ok(())
    }
}

impl UserRepository for UserRepositoryImpl {
    async fn find_by_phone(&self, phone: Phone) -> Result<Option<User>, RepositoryError> {
        let phone: String = phone.into();

        let user_do = crate::models::user::Entity::find()
            .filter(crate::models::user::Column::Phone.eq(phone.clone()))
            .one(&self.db)
            .await
            .context(format!("failed to find user with phone: {}", phone))
            .map_err(RepositoryError::Db)?;

        user_do
            .map(|user_do| UserDataConverter::make_from_do(user_do))
            .transpose()
            .map(|user| {
                if let Some(user) = user.clone() {
                    self.aggregate_manager.lock().unwrap().attach(user);
                }

                user
            })
            .context(format!("failed to validation user with phone: {}", phone))
            .map_err(RepositoryError::ValidationError)
    }

    async fn find_by_identity_card_id(
        &self,
        identity_card_id: IdentityCardId,
    ) -> Result<Option<User>, RepositoryError> {
        let identity_card_id: String = identity_card_id.into();

        let user_do = crate::models::user::Entity::find()
            .filter(crate::models::user::Column::IdentityCardId.eq(identity_card_id.clone()))
            .one(&self.db)
            .await
            .context(format!(
                "failed to find user with identity card id: {}",
                identity_card_id
            ))
            .map_err(RepositoryError::Db)?;

        user_do
            .map(|user_do| UserDataConverter::make_from_do(user_do))
            .transpose()
            .map(|user| {
                if let Some(user) = user.clone() {
                    self.aggregate_manager.lock().unwrap().attach(user);
                }

                user
            })
            .context(format!(
                "failed to validation user with identity card id: {}",
                identity_card_id
            ))
            .map_err(RepositoryError::ValidationError)
    }

    async fn remove_by_phone(&self, phone: Phone) -> Result<(), RepositoryError> {
        let user = self.find_by_phone(phone).await?;

        if let Some(user) = user {
            self.aggregate_manager.lock().unwrap().detach(&user);
            self.remove(user).await?;
        }

        Ok(())
    }
}
