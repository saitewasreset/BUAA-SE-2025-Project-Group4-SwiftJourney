use crate::domain::model::personal_info::PersonalInfo;
use crate::domain::model::session::Session;
use crate::domain::model::user::UserInfo;
use async_trait::async_trait;
use shared::domain::Identifiable;
use shared::internal::user::command::{
    ClearWrongPaymentPasswordTriedCommand, SessionQuery, SetPaymentPasswordCommand, UserInfoQuery,
    VerifyPasswordCommand, VerifyPaymentPasswordCommand,
};
use shared::internal::user::dto::{
    DbPersonalInfo, DbUserDTO, PersonalInfoDTO, SessionDTO, UserCombinedInfoDTO, UserInfoDTO,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserInternalServiceError {
    #[error("no such user: id = {0}")]
    NoSuchUser(u64),
    #[error("invalid payment password: {0}")]
    InvalidPaymentPassword(String),
    #[error("invalid session ID: {0}")]
    InvalidSessionId(String),
    #[error(transparent)]
    RelatedServiceError(#[from] anyhow::Error),
}

impl From<Session> for SessionDTO {
    fn from(value: Session) -> Self {
        SessionDTO {
            user_id: value.user_id().into(),
            created_at: value.created_at(),
            expires_at: value.expires_at(),
        }
    }
}

impl From<PersonalInfo> for PersonalInfoDTO {
    fn from(value: PersonalInfo) -> Self {
        PersonalInfoDTO {
            id: Some(
                value
                    .get_id()
                    .expect("saved personal info should have id")
                    .into(),
            ),
            uuid: value.uuid(),
            name: value.name().to_string(),
            identity_card_id: value.identity_card_id().to_string(),
            preferred_seat_location: value.preferred_seat_location().map(|x| x.into()),
            user_id: (*value.user_id()).into(),
            is_default: value.is_default(),
        }
    }
}

impl From<UserInfo> for UserInfoDTO {
    fn from(value: UserInfo) -> Self {
        UserInfoDTO {
            name: value.name.to_string(),
            gender: value.gender.map(|x| x.to_string()),
            age: value.age.map(|x| x.into()),
            phone: value.phone.to_string(),
            email: value.email.map(|x| x.to_string()),
            identity_card_id: value.identity_card_id.to_string(),
        }
    }
}

impl From<crate::models::user::Model> for DbUserDTO {
    fn from(value: crate::models::user::Model) -> Self {
        DbUserDTO {
            id: value.id,
            username: value.username,
            hashed_password: value.hashed_password,
            hashed_payment_password: value.hashed_payment_password,
            salt: value.salt,
            wrong_payment_password_tried: value.wrong_payment_password_tried,
            gender: value.gender,
            age: value.age,
            phone: value.phone,
            email: value.email,
            name: value.name,
            identity_card_id: value.identity_card_id,
        }
    }
}

impl From<crate::models::person_info::Model> for DbPersonalInfo {
    fn from(value: crate::models::person_info::Model) -> Self {
        DbPersonalInfo {
            id: value.id,
            uuid: value.uuid,
            name: value.name,
            identity_card: value.identity_card,
            preferred_seat_location: value.preferred_seat_location,
            user_id: value.user_id,
            is_default: value.is_default,
        }
    }
}

#[async_trait]
pub trait UserInternalService: 'static + Send + Sync {
    async fn verify_password(
        &self,
        command: VerifyPasswordCommand,
    ) -> Result<bool, UserInternalServiceError>;

    async fn verify_payment_password(
        &self,
        command: VerifyPaymentPasswordCommand,
    ) -> Result<bool, UserInternalServiceError>;

    async fn set_payment_password(
        &self,
        command: SetPaymentPasswordCommand,
    ) -> Result<(), UserInternalServiceError>;

    async fn clear_wrong_payment_password_tried(
        &self,
        command: ClearWrongPaymentPasswordTriedCommand,
    ) -> Result<(), UserInternalServiceError>;

    async fn get_session(
        &self,
        query: SessionQuery,
    ) -> Result<Option<SessionDTO>, UserInternalServiceError>;

    async fn get_user_info(
        &self,
        query: UserInfoQuery,
    ) -> Result<Option<UserCombinedInfoDTO>, UserInternalServiceError>;

    async fn db_get_user_info(&self) -> Result<Vec<DbUserDTO>, UserInternalServiceError>;

    async fn db_get_personal_info(&self) -> Result<Vec<DbPersonalInfo>, UserInternalServiceError>;
}
