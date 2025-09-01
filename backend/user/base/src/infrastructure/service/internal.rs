use crate::application::service::internal::{UserInternalService, UserInternalServiceError};
use crate::domain::model::session::SessionId;
use crate::domain::model::user::{PaymentPassword, User, UserId};
use crate::domain::repository::personal_info::PersonalInfoRepository;
use crate::domain::repository::user::UserRepository;
use crate::domain::service::session::SessionManagerService;
use crate::domain::service::user::UserService;
use async_trait::async_trait;
use shared::domain::Identifiable;
use shared::internal::user::command::{
    ClearWrongPaymentPasswordTriedCommand, SessionQuery, SetPaymentPasswordCommand, UserInfoQuery,
    VerifyPasswordCommand, VerifyPaymentPasswordCommand,
};
use shared::internal::user::dto::PersonalInfoDTO;
use shared::internal::user::dto::{
    DbPersonalInfo, DbUserDTO, SessionDTO, UserCombinedInfoDTO, UserInfoDTO,
};
use std::sync::Arc;
use tracing::{error, instrument};
use uuid::Uuid;

pub struct UserInternalServiceImpl<US, SMS, UR, PIR>
where
    US: UserService,
    SMS: SessionManagerService,
    UR: UserRepository,
    PIR: PersonalInfoRepository,
{
    user_service: Arc<US>,
    session_manager_service: Arc<SMS>,
    user_repository: Arc<UR>,
    personal_info_repository: Arc<PIR>,
}

impl<US, SMS, UR, PIR> UserInternalServiceImpl<US, SMS, UR, PIR>
where
    US: UserService,
    SMS: SessionManagerService,
    UR: UserRepository,
    PIR: PersonalInfoRepository,
{
    pub fn new(
        user_service: Arc<US>,
        session_manager_service: Arc<SMS>,
        user_repository: Arc<UR>,
        personal_info_repository: Arc<PIR>,
    ) -> Self {
        Self {
            user_service,
            session_manager_service,
            user_repository,
            personal_info_repository,
        }
    }

    async fn load_user_from_query(
        &self,
        user_id: u64,
    ) -> Result<Option<User>, UserInternalServiceError> {
        let user_id = UserId::from(user_id);

        let user = self
            .user_repository
            .find(user_id)
            .await
            .inspect_err(|e| error!("error loading user from db: {:?}", e))
            .map_err(|e| UserInternalServiceError::RelatedServiceError(e.into()))?;

        Ok(user)
    }
}

#[async_trait]
impl<US, SMS, UR, PIR> UserInternalService for UserInternalServiceImpl<US, SMS, UR, PIR>
where
    US: UserService,
    SMS: SessionManagerService,
    UR: UserRepository,
    PIR: PersonalInfoRepository,
{
    #[instrument(skip(self))]
    async fn verify_password(
        &self,
        command: VerifyPasswordCommand,
    ) -> Result<bool, UserInternalServiceError> {
        let user = self.load_user_from_query(command.user_id).await?;

        if let Some(user) = user {
            Ok(self
                .user_service
                .verify_password(&user, command.raw_password)
                .await
                .is_ok())
        } else {
            Err(UserInternalServiceError::NoSuchUser(command.user_id))
        }
    }

    async fn verify_payment_password(
        &self,
        command: VerifyPaymentPasswordCommand,
    ) -> Result<bool, UserInternalServiceError> {
        let user = self
            .load_user_from_query(command.user_id)
            .await?
            .ok_or(UserInternalServiceError::NoSuchUser(command.user_id))?;

        Ok(self
            .user_service
            .verify_payment_password(&user, command.raw_payment_password)
            .await
            .is_ok())
    }

    async fn set_payment_password(
        &self,
        command: SetPaymentPasswordCommand,
    ) -> Result<(), UserInternalServiceError> {
        let user_id = UserId::from(command.user_id);

        let payment_password_opt = if let Some(raw_payment_password) = command.raw_payment_password
        {
            Some(
                PaymentPassword::try_from(raw_payment_password.as_str()).map_err(
                    |_for_super_earth| {
                        UserInternalServiceError::InvalidPaymentPassword(raw_payment_password)
                    },
                )?,
            )
        } else {
            None
        };

        self.user_service
            .set_payment_password(user_id, payment_password_opt)
            .await
            .inspect_err(|e| error!("error set payment password: {:?}", e))
            .map_err(|e| UserInternalServiceError::RelatedServiceError(e.into()))
    }

    async fn clear_wrong_payment_password_tried(
        &self,
        command: ClearWrongPaymentPasswordTriedCommand,
    ) -> Result<(), UserInternalServiceError> {
        let user_id = UserId::from(command.user_id);

        self.user_service
            .clear_wrong_payment_password_tried(user_id)
            .await
            .inspect_err(|e| error!("error clear wrong payment password: {:?}", e))
            .map_err(|e| UserInternalServiceError::RelatedServiceError(e.into()))
    }

    async fn get_session(
        &self,
        query: SessionQuery,
    ) -> Result<Option<SessionDTO>, UserInternalServiceError> {
        let session_id = SessionId::from(Uuid::try_from(query.session_id.as_str()).map_err(
            |_for_super_earth| UserInternalServiceError::InvalidSessionId(query.session_id),
        )?);

        Ok(self
            .session_manager_service
            .get_session(session_id)
            .await
            .map_err(|e| UserInternalServiceError::RelatedServiceError(e.into()))?
            .map(|s| s.into()))
    }

    async fn get_user_info(
        &self,
        query: UserInfoQuery,
    ) -> Result<Option<UserCombinedInfoDTO>, UserInternalServiceError> {
        let user = self.load_user_from_query(query.user_id).await?;

        if let Some(user) = user {
            let user_info_dto: UserInfoDTO = user.user_info().clone().into();

            let personal_info_list = self
                .personal_info_repository
                .find_by_user_id(user.get_id().unwrap())
                .await
                .map_err(|e| UserInternalServiceError::RelatedServiceError(e.into()))?;

            let personal_info_dto_list: Vec<PersonalInfoDTO> = personal_info_list
                .into_iter()
                .map(|personal_info| personal_info.into())
                .collect();

            Ok(Some(UserCombinedInfoDTO {
                user_id: user.get_id().unwrap().into(),
                username: user.username().to_string(),
                user_info: user_info_dto,
                personal_info_list: personal_info_dto_list,
            }))
        } else {
            Ok(None)
        }
    }

    async fn db_get_user_info(&self) -> Result<Vec<DbUserDTO>, UserInternalServiceError> {
        let user_entity_list = self
            .user_repository
            .load_all_raw()
            .await
            .map_err(|e| UserInternalServiceError::RelatedServiceError(e.into()))?;

        Ok(user_entity_list.into_iter().map(|u| u.into()).collect())
    }

    async fn db_get_personal_info(&self) -> Result<Vec<DbPersonalInfo>, UserInternalServiceError> {
        let personal_info_entity_list = self
            .personal_info_repository
            .load_all_raw()
            .await
            .map_err(|e| UserInternalServiceError::RelatedServiceError(e.into()))?;

        Ok(personal_info_entity_list
            .into_iter()
            .map(|p| p.into())
            .collect())
    }
}
