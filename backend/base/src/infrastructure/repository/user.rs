//! 用户仓储实现模块
//!
//! 本模块提供了用户实体的数据库仓储实现，包括：
//! - 用户数据的数据库操作（增删改查）
//! - 领域模型与数据库模型之间的转换
//! - 变更追踪与聚合管理

use crate::domain::model::password::HashedPassword;
use crate::domain::model::user::{
    Age, Gender, IdentityCardId, PasswordAttempts, Phone, RealName, User, UserId, UserInfo,
    Username,
};
use crate::domain::repository::user::UserRepository;
use crate::domain::service::{AggregateManagerImpl, DiffInfo};
use crate::domain::DbId;
use crate::domain::{
    AggregateManager, DbRepositorySupport, DiffType, Identifiable, MultiEntityDiff, Repository,
    RepositoryError, TypedDiff,
};
use anyhow::Context;
use async_trait::async_trait;
use email_address::EmailAddress;
use sea_orm::ColumnTrait;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait, QueryFilter};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

impl_db_id_from_u64!(UserId, i32, "user id");

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
        let username = Username::try_from(user_do.username)?;

        let wrong_payment_password_tried: PasswordAttempts =
            user_do.wrong_payment_password_tried.try_into()?;

        let name = RealName::try_from(user_do.name)?;

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

            let diff_type = DiffType::from(&diff);

            let old = diff.old;
            let new = diff.new;

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

#[async_trait]
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
    /// /// # Returns
    /// 返回插入后生成的用户ID
    /// # Errors
    /// 当数据库操作失败时返回错误
    async fn on_insert(&self, aggregate: User) -> Result<UserId, RepositoryError> {
        let id = aggregate.get_id();

        let model = UserDataConverter::transform_to_do(aggregate);

        let result_model = model
            .insert(&self.db)
            .await
            .context(format!("failed to insert user with id: {:?}", id))
            .map_err(RepositoryError::Db)?;

        Ok((result_model.id as u64).into())
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

#[async_trait]
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
            .map(UserDataConverter::make_from_do)
            .transpose()
            .inspect(|user| {
                if let Some(user) = user.clone() {
                    self.aggregate_manager.lock().unwrap().attach(user);
                }
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
            .map(UserDataConverter::make_from_do)
            .transpose()
            .inspect(|user| {
                if let Some(user) = user.clone() {
                    self.aggregate_manager.lock().unwrap().attach(user);
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::password::PasswordSalt;
    use crate::domain::model::user::{Gender, Phone, RealName, UserId, UserInfo, Username};
    use email_address::EmailAddress;
    use sea_orm::{ConnectionTrait, Database, DatabaseConnection, Statement};

    /// 初始化内存数据库
    async fn setup_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();

        db.execute(Statement::from_string(
            db.get_database_backend(),
            r#"
            CREATE TABLE user (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL,
                hashed_password TEXT NOT NULL,
                hashed_payment_password TEXT,
                salt TEXT NOT NULL,
                wrong_payment_password_tried INTEGER NOT NULL,
                gender TEXT,
                age INTEGER,
                phone TEXT NOT NULL,
                email TEXT,
                name TEXT NOT NULL,
                identity_card_id TEXT NOT NULL
            );
            "#
            .to_string(),
        ))
        .await
        .unwrap();

        db
    }

    /// 构造一个简单 User
    fn make_test_user(id: Option<UserId>) -> User {
        let username = Username::try_from("testuser".to_string()).unwrap();
        let name = RealName::try_from("Test Name".to_string()).unwrap();
        let phone = Phone::try_from("15999999999".to_string()).unwrap();
        let email = Some(EmailAddress::from_str("test@example.com").unwrap());
        let identity_card_id = IdentityCardId::try_from("11010519491231002X".to_string()).unwrap();
        let user_info = UserInfo::new(
            name,
            Some(Gender::Male),
            Some(Age::try_from(20).unwrap()),
            phone,
            email,
            identity_card_id,
        );

        User::new(
            id,
            username,
            HashedPassword {
                hashed_password: vec![],
                salt: PasswordSalt::from(vec![]),
            },
            None,
            PasswordAttempts::try_from(0).unwrap(),
            user_info,
        )
    }

    #[tokio::test]
    async fn test_on_insert_and_on_select() {
        let db = setup_db().await;
        let repo = UserRepositoryImpl::new(db);

        let user = make_test_user(None);

        // 正向测试：插入并查询
        let user_id = repo.on_insert(user.clone()).await.unwrap();
        let fetched = repo.on_select(user_id).await.unwrap().unwrap();
        assert_eq!(fetched.username(), user.username());

        // 反向测试：查询不存在ID
        let not_found = repo.on_select(999.into()).await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_on_update() {
        let db = setup_db().await;
        let repo = UserRepositoryImpl::new(db.clone());

        let mut user = make_test_user(None);
        let user_id = repo.on_insert(user.clone()).await.unwrap();
        user.set_id(user_id);

        // 修改字段
        let mut modified_user = user.clone();
        modified_user.set_username(Username::try_from("newname".to_string()).unwrap());

        let mut diff = MultiEntityDiff::new();
        diff.add_change(TypedDiff::new(
            DiffType::Modified,
            Some(user),
            Some(modified_user.clone()),
        ));

        repo.on_update(diff).await.unwrap();

        let fetched = repo.on_select(user_id).await.unwrap().unwrap();
        assert_eq!(fetched.username(), modified_user.username());
    }

    #[tokio::test]
    async fn test_on_delete() {
        let db = setup_db().await;
        let repo = UserRepositoryImpl::new(db.clone());

        let mut user = make_test_user(None);
        let user_id = repo.on_insert(user.clone()).await.unwrap();
        user.set_id(user_id);

        // 正向删除
        repo.on_delete(user.clone()).await.unwrap();
        let fetched = repo.on_select(user_id).await.unwrap();
        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_find_by_phone() {
        let db = setup_db().await;
        let repo = UserRepositoryImpl::new(db.clone());

        let mut user = make_test_user(None);
        let phone = user.user_info().phone.clone();
        let user_id = repo.on_insert(user.clone()).await.unwrap();
        user.set_id(user_id);

        // 正向查询
        let fetched = repo.find_by_phone(phone.clone()).await.unwrap().unwrap();
        assert_eq!(fetched.get_id(), Some(user_id));

        // 反向查询
        let not_found = repo
            .find_by_phone(Phone::try_from("15899999999".to_string()).unwrap())
            .await
            .unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_find_by_identity_card_id() {
        let db = setup_db().await;
        let repo = UserRepositoryImpl::new(db.clone());

        let mut user = make_test_user(None);
        let id_card = user.user_info().identity_card_id.clone();
        let user_id = repo.on_insert(user.clone()).await.unwrap();
        user.set_id(user_id);

        // 正向查询
        let fetched = repo
            .find_by_identity_card_id(id_card.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.get_id(), Some(user_id));

        // 反向查询
        let not_found = repo
            .find_by_identity_card_id(
                IdentityCardId::try_from("150524200701216857".to_string()).unwrap(),
            )
            .await
            .unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_remove_by_phone() {
        let db = setup_db().await;
        let repo = UserRepositoryImpl::new(db.clone());

        let mut user = make_test_user(None);
        let phone = user.user_info().phone.clone();
        let user_id = repo.on_insert(user.clone()).await.unwrap();
        user.set_id(user_id);

        // 正向删除
        repo.remove_by_phone(phone.clone()).await.unwrap();
        let fetched = repo.find_by_phone(phone.clone()).await.unwrap();
        assert!(fetched.is_none());

        // 反向删除（不存在用户）
        repo.remove_by_phone(Phone::try_from("15999999999".to_string()).unwrap())
            .await
            .unwrap();
    }
}
