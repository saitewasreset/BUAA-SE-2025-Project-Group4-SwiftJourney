use crate::application::commands::user_manager::{
    UserLoginCommand, UserLogoutCommand, UserRegisterCommand,
};
use crate::application::service::user_manager::{UserManagerError, UserManagerService};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::Identifiable;
use crate::domain::model::session::SessionId;
use crate::domain::model::user::{IdentityCardId, Phone, RawPassword, RealName, Username};
use crate::domain::repository::user::UserRepository;
use crate::domain::service::session::SessionManagerService;
use crate::domain::service::user::UserService;
use async_trait::async_trait;
use std::sync::Arc;

pub struct UserManagerServiceImpl<U, R, S>
where
    U: UserService + 'static + Send + Sync,
    R: UserRepository + 'static + Send + Sync,
    S: SessionManagerService + 'static + Send + Sync,
{
    user_service: Arc<U>,
    user_repository: Arc<R>,
    session_manager: Arc<S>,
}

impl<U, R, S> UserManagerServiceImpl<U, R, S>
where
    U: UserService + 'static + Send + Sync,
    R: UserRepository + 'static + Send + Sync,
    S: SessionManagerService + 'static + Send + Sync,
{
    pub fn new(user_service: Arc<U>, user_repository: Arc<R>, session_manager: Arc<S>) -> Self {
        UserManagerServiceImpl {
            user_service,
            user_repository,
            session_manager,
        }
    }
}

#[async_trait]
impl<U, R, S> UserManagerService for UserManagerServiceImpl<U, R, S>
where
    U: UserService + 'static + Send + Sync,
    R: UserRepository + 'static + Send + Sync,
    S: SessionManagerService + 'static + Send + Sync,
{
    async fn register(
        &self,
        command: UserRegisterCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let phone =
            Phone::try_from(command.phone).map_err(|e| GeneralError::BadRequest(e.to_string()))?;
        let identity_card_id = IdentityCardId::try_from(command.identity_card_id)
            .map_err(|e| GeneralError::BadRequest(e.to_string()))?;

        let username = Username::try_from(command.username)
            .map_err(|_for_super_earth| UserManagerError::InvalidUsernameFormat)?;
        let raw_password = RawPassword::try_from(command.password)
            .map_err(|_for_super_earth| UserManagerError::InvalidPasswordFormat)?;

        let name = RealName::try_from(command.name)
            .map_err(|_for_super_earth| UserManagerError::InvalidNameFormat)?;

        self.user_service
            .register(username, raw_password, name, phone, identity_card_id)
            .await
            .map_err(|e| e.into())
    }

    async fn login(
        &self,
        command: UserLoginCommand,
    ) -> Result<SessionId, Box<dyn ApplicationError>> {
        let phone =
            Phone::try_from(command.phone).map_err(|e| GeneralError::BadRequest(e.to_string()))?;

        let user = self
            .user_repository
            .find_by_phone(phone)
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
            .ok_or(UserManagerError::InvalidPhoneNumberOrPassword)?;

        self.user_service
            .verify_password(&user, command.password)
            .await?;

        let session = self
            .session_manager
            .create_session(user.get_id().unwrap())
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

        Ok(session.session_id())
    }

    async fn logout(&self, command: UserLogoutCommand) -> Result<(), Box<dyn ApplicationError>> {
        let session_id = SessionId::try_from(command.session_id.as_str())
            .map_err(|_for_super_earth| GeneralError::InvalidSessionId)?;
        if let Some(session) = self
            .session_manager
            .get_session(session_id)
            .await
            .map_err(|_for_super_earth| GeneralError::InternalServerError)?
        {
            self.session_manager
                .delete_session(session)
                .await
                .map_err(|_for_super_earth| GeneralError::InternalServerError)?;

            Ok(())
        } else {
            Err(GeneralError::InvalidSessionId.into())
        }
    }
}
