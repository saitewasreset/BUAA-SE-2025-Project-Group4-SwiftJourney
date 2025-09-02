use crate::api::{ApiEndpoint, InternalApiError, SuperClient, UserInternalServiceApi};
use crate::internal::user::command::{
    ClearWrongPaymentPasswordTriedCommand, SessionQuery, SetPaymentPasswordCommand, UserInfoQuery,
    VerifyPasswordCommand, VerifyPaymentPasswordCommand,
};
use crate::internal::user::dto::{DbPersonalInfo, DbUserDTO, SessionDTO, UserCombinedInfoDTO};
use crate::ports::user::UserPort;
use async_trait::async_trait;
use tracing::error;

pub struct HttpUserPortImpl {
    super_client: SuperClient,
}

impl HttpUserPortImpl {
    pub fn new(api_endpoint: ApiEndpoint) -> Self {
        let super_client = SuperClient::new(api_endpoint);

        Self { super_client }
    }
}

#[async_trait]
impl UserPort for HttpUserPortImpl {
    async fn verify_password(
        &self,
        command: VerifyPasswordCommand,
    ) -> Result<bool, InternalApiError> {
        self.super_client
            .post(UserInternalServiceApi::VerifyPassword, command)
            .await
            .inspect_err(|e| error!("Failed to verify password: {:?}", e))
    }

    async fn verify_payment_password(
        &self,
        command: VerifyPaymentPasswordCommand,
    ) -> Result<bool, InternalApiError> {
        self.super_client
            .post(UserInternalServiceApi::VerifyPaymentPassword, command)
            .await
            .inspect_err(|e| error!("Failed to verify payment password: {:?}", e))
    }

    async fn set_payment_password(
        &self,
        command: SetPaymentPasswordCommand,
    ) -> Result<(), InternalApiError> {
        self.super_client
            .post(UserInternalServiceApi::SetPaymentPassword, command)
            .await
            .inspect_err(|e| error!("Failed to set payment password: {:?}", e))
    }

    async fn clear_wrong_payment_password_tried(
        &self,
        command: ClearWrongPaymentPasswordTriedCommand,
    ) -> Result<(), InternalApiError> {
        self.super_client
            .post(
                UserInternalServiceApi::ClearWrongPaymentPasswordTried,
                command,
            )
            .await
            .inspect_err(|e| error!("Failed to clear wrong payment password: {:?}", e))
    }

    async fn get_session(
        &self,
        query: SessionQuery,
    ) -> Result<Option<SessionDTO>, InternalApiError> {
        self.super_client
            .post(UserInternalServiceApi::GetSession, query)
            .await
            .inspect_err(|e| error!("Failed to get session: {:?}", e))
    }

    async fn get_user_info(
        &self,
        query: UserInfoQuery,
    ) -> Result<Option<UserCombinedInfoDTO>, InternalApiError> {
        self.super_client
            .post(UserInternalServiceApi::GetUserInfo, query)
            .await
            .inspect_err(|e| error!("Failed to get user info: {:?}", e))
    }

    async fn db_get_user_info(&self) -> Result<Vec<DbUserDTO>, InternalApiError> {
        self.super_client
            .get(UserInternalServiceApi::DbGetUserInfo)
            .await
            .inspect_err(|e| error!("Failed to get db user info: {:?}", e))
    }

    async fn db_get_personal_info(&self) -> Result<Vec<DbPersonalInfo>, InternalApiError> {
        self.super_client
            .get(UserInternalServiceApi::DbGetPersonalInfo)
            .await
            .inspect_err(|e| error!("Failed to get db personal info: {:?}", e))
    }
}
