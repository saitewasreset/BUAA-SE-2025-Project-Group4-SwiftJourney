use crate::domain::model::user::{
    Age, Gender, IdentityCardId, PasswordAttempts, Phone, User, UserId, UserInfo,
};
use crate::domain::service::AggregateManagerImpl;
use crate::domain::{
    DbRepositorySupport, DiffType, Identifiable, MultiEntityDiff, RepositoryError,
};
use anyhow::{Context, anyhow};
use argon2::password_hash::PasswordHashString;
use email_address::EmailAddress;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

pub struct UserRepositoryImpl {
    db: DatabaseConnection,
    aggregate_manager: Arc<Mutex<AggregateManagerImpl<User, UserId>>>,
}

pub struct UserDataConverter;

impl UserDataConverter {
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

    pub fn transform_to_do(user: User) -> crate::models::user::ActiveModel {
        let mut model = crate::models::user::ActiveModel {
            id: ActiveValue::NotSet,
            username: ActiveValue::Set(user.username().to_owned()),
            hashed_password: ActiveValue::Set(user.hashed_password().as_bytes().to_vec()),
            hashed_payment_password: ActiveValue::Set(
                user.hashed_payment_password().as_bytes().to_vec(),
            ),
            salt: ActiveValue::Set(
                user.hashed_payment_password()
                    .password_hash()
                    .salt
                    .unwrap()
                    .as_str()
                    .as_bytes()
                    .to_vec(),
            ),
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

        model
    }

    pub fn make_from_do(user_do: crate::models::user::Model) -> anyhow::Result<User> {
        let user_id: UserId = (user_do.id as u64).into();
        let username: String = user_do.username;

        let hashed_password_string =
            Self::parse_bytes_to_password_hash_string(user_do.hashed_password)?;

        let hashed_payment_password_string =
            Self::parse_bytes_to_password_hash_string(user_do.hashed_payment_password)?;

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

        let user = User::new(
            Some(user_id),
            username,
            hashed_password_string,
            hashed_payment_password_string,
            wrong_payment_password_tried,
            user_info,
        );

        Ok(user)
    }
}

impl DbRepositorySupport<User, UserId> for UserRepositoryImpl {
    type Manager = AggregateManagerImpl<User, UserId>;

    fn get_aggregate_manager(&self) -> Arc<Mutex<Self::Manager>> {
        Arc::clone(&self.aggregate_manager)
    }

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

    async fn on_update(&self, diff: MultiEntityDiff) -> Result<(), RepositoryError> {
        for changes in diff.get_changes::<User, UserId>() {
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
