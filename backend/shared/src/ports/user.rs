use crate::api::InternalApiError;
use crate::internal::user::command::{
    ClearWrongPaymentPasswordTriedCommand, SessionQuery, SetPaymentPasswordCommand, UserInfoQuery,
    VerifyPasswordCommand, VerifyPaymentPasswordCommand,
};
use crate::internal::user::dto::{DbPersonalInfo, DbUserDTO, SessionDTO, UserCombinedInfoDTO};
use async_trait::async_trait;

#[async_trait]
pub trait UserPort: 'static + Send + Sync {
    async fn verify_password(
        &self,
        command: VerifyPasswordCommand,
    ) -> Result<bool, InternalApiError>;

    async fn verify_payment_password(
        &self,
        command: VerifyPaymentPasswordCommand,
    ) -> Result<bool, InternalApiError>;

    async fn set_payment_password(
        &self,
        command: SetPaymentPasswordCommand,
    ) -> Result<(), InternalApiError>;

    async fn clear_wrong_payment_password_tried(
        &self,
        command: ClearWrongPaymentPasswordTriedCommand,
    ) -> Result<(), InternalApiError>;

    async fn get_session(
        &self,
        query: SessionQuery,
    ) -> Result<Option<SessionDTO>, InternalApiError>;

    async fn get_user_info(
        &self,
        query: UserInfoQuery,
    ) -> Result<Option<UserCombinedInfoDTO>, InternalApiError>;

    async fn db_get_user_info(&self) -> Result<Vec<DbUserDTO>, InternalApiError>;

    async fn db_get_personal_info(&self) -> Result<Vec<DbPersonalInfo>, InternalApiError>;
}
