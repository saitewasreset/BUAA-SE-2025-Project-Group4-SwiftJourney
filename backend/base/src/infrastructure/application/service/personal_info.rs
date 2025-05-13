//! 个人信息服务实现
//!
//! 本模块提供了`PersonalInfoService` trait的具体实现，负责处理个人信息的查询和更新操作。
//! 通过与会话管理和个人信息仓储的交互，完成业务逻辑的执行。
//!
//! # 主要组件
//! - `PersonalInfoServiceImpl`: 个人信息服务的主要实现结构体
//!   - 依赖`SessionManagerService`进行会话验证
//!   - 依赖`PersonalInfoRepository`进行个人信息数据存取
//!
//! # 实现细节
//! 服务实现包含以下核心功能:
//! 1. 通过会话ID验证用户身份
//! 2. 查询个人信息
//! 3. 更新个人信息
//!
//! # 注意事项
//! - 所有操作都需要有效的会话ID
//! - 更新操作会验证输入数据的有效性
//! - 错误处理遵循应用层定义的错误规范

use crate::application::commands::personal_info::{PersonalInfoQuery, SetPersonalInfoCommand};
use crate::application::service::personal_info::{
    PersonalInfoDTO, PersonalInfoError, PersonalInfoService,
};
use crate::application::{ApplicationError, GeneralError};
use crate::domain::model::personal_info::{PersonalInfo, PreferredSeatLocation};
use crate::domain::model::session::SessionId;
use crate::domain::model::user::{IdentityCardId, RealName, UserId};
use crate::domain::repository::personal_info::PersonalInfoRepository;
use crate::domain::service::session::SessionManagerService;
use async_trait::async_trait;
use std::sync::Arc;

/// 个人信息服务实现
///
/// 依赖:
/// - 会话管理服务 - 用于验证会话和获取用户ID
/// - 个人信息仓储 - 用于存取个人信息数据
pub struct PersonalInfoServiceImpl<S, P>
where
    S: SessionManagerService + 'static + Send + Sync,
    P: PersonalInfoRepository + 'static + Send + Sync,
{
    session_manager: Arc<S>,
    personal_info_repository: Arc<P>,
}

impl<S, P> PersonalInfoServiceImpl<S, P>
where
    S: SessionManagerService + 'static + Send + Sync,
    P: PersonalInfoRepository + 'static + Send + Sync,
{
    /// 创建个人信息服务实例
    ///
    /// # Arguments
    /// * `session_manager` - 会话管理服务
    /// * `personal_info_repository` - 个人信息仓储
    ///
    /// # Returns
    /// 返回初始化好的`PersonalInfoServiceImpl`实例
    pub fn new(session_manager: Arc<S>, personal_info_repository: Arc<P>) -> Self {
        PersonalInfoServiceImpl {
            session_manager,
            personal_info_repository,
        }
    }

    /// 根据会话ID获取用户ID和对应的个人信息
    ///
    /// # Arguments
    /// * `session_id` - 会话ID字符串
    ///
    /// # Returns
    /// - `Ok((UserId, Option<PersonalInfo>))`: 用户ID和可能存在的个人信息
    /// - `Err(Box<dyn ApplicationError>)`: 错误信息
    ///
    /// # Errors
    /// - `GeneralError::InvalidSessionId`: 无效的会话ID
    /// - `GeneralError::InternalServerError`: 内部服务错误
    async fn get_user_id_and_personal_info_by_session(
        &self,
        session_id: &str,
    ) -> Result<(UserId, Option<PersonalInfo>), Box<dyn ApplicationError>> {
        let session_id =
            SessionId::try_from(session_id).map_err(|_| GeneralError::InvalidSessionId)?;

        let user_id = self
            .session_manager
            .get_user_id_by_session(session_id)
            .await
            .map_err(|_| GeneralError::InternalServerError)?
            .ok_or(GeneralError::InvalidSessionId)?;

        let personal_info = self
            .personal_info_repository
            .find_by_user_id(user_id)
            .await
            .map_err(|_| GeneralError::InternalServerError)?;

        Ok((user_id, personal_info))
    }
}

#[async_trait]
impl<S, P> PersonalInfoService for PersonalInfoServiceImpl<S, P>
where
    S: SessionManagerService + 'static + Send + Sync,
    P: PersonalInfoRepository + 'static + Send + Sync,
{
    /// 获取个人信息
    ///
    /// # Arguments
    /// * `query` - 包含会话ID的查询条件
    ///
    /// # Returns
    /// - `Ok(PersonalInfoDTO)`: 成功获取的个人信息DTO
    /// - `Err(Box<dyn ApplicationError>)`: 错误信息
    ///
    /// # Errors
    /// - `GeneralError::InvalidSessionId`: 无效的会话ID
    /// - `GeneralError::InternalServerError`: 内部服务错误
    async fn get_personal_info(
        &self,
        query: PersonalInfoQuery,
    ) -> Result<PersonalInfoDTO, Box<dyn ApplicationError>> {
        let (_, personal_info) = self
            .get_user_id_and_personal_info_by_session(&query.session_id)
            .await?;

        let personal_info: PersonalInfo = personal_info.ok_or_else(|| {
            Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
        })?;

        Ok(personal_info.into())
    }

    /// 设置个人信息
    ///
    /// # Arguments
    /// * `command` - 包含更新数据和会话ID的命令对象
    ///
    /// # Returns
    /// - `Ok(())`: 更新成功
    /// - `Err(Box<dyn ApplicationError>)`: 错误信息
    ///
    /// # Errors
    /// - `GeneralError::InvalidSessionId`: 无效的会话ID
    /// - `GeneralError::BadRequest`: 无效的输入数据
    /// - `GeneralError::InternalServerError`: 内部服务错误
    async fn set_personal_info(
        &self,
        command: SetPersonalInfoCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let (user_id, personal_info_opt) = self
            .get_user_id_and_personal_info_by_session(&command.session_id)
            .await?;

        // 解析输入数据
        let name = RealName::try_from(command.name)
            .map_err(|e| GeneralError::BadRequest(e.to_string()))?;

        let identity_card_id = IdentityCardId::try_from(command.identity_card_id)
            .map_err(|_| PersonalInfoError::InvalidIdentityCardId)?;

        let preferred_seat_location = PreferredSeatLocation::try_from(
            command
                .preferred_seat_location
                .chars()
                .next()
                .ok_or_else(|| {
                    GeneralError::BadRequest("Preferred seat location cannot be empty".into())
                })?,
        )
        .map_err(|_| PersonalInfoError::InvalidPreferredSeatLocation)?;

        // 更新或创建个人信息
        if let Some(mut personal_info) = personal_info_opt {
            // 更新现有个人信息
            personal_info.set_name(name);
            personal_info.set_identity_card_id(identity_card_id);
            personal_info.set_preferred_seat_location(preferred_seat_location);

            self.personal_info_repository
                .save(&mut personal_info)
                .await
                .map_err(|_| GeneralError::InternalServerError)?;
        } else {
            // 创建新的个人信息
            let mut new_personal_info = PersonalInfo::new(
                None,
                uuid::Uuid::new_v4(),
                name,
                identity_card_id,
                preferred_seat_location,
                user_id,
            );

            self.personal_info_repository
                .save(&mut new_personal_info)
                .await
                .map_err(|_| GeneralError::InternalServerError)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Repository;
    use crate::domain::RepositoryError;
    use crate::domain::model::personal_info::{PersonalInfo, PersonalInfoId};
    use crate::domain::model::session::Session;
    use async_trait::async_trait;
    use mockall::{mock, predicate::*};

    // 模拟会话管理服务
    mock! {
        SessionManagerService {}
        #[async_trait]
        impl SessionManagerService for SessionManagerService {
            async fn create_session(&self, user_id: UserId) -> Result<Session, RepositoryError>;
            async fn delete_session(&self, session: Session) -> Result<(), RepositoryError>;
            async fn get_session(&self, session_id: SessionId) -> Result<Option<Session>, RepositoryError>;
            async fn get_user_id_by_session(&self, session_id: SessionId) -> Result<Option<UserId>, RepositoryError>;
        }
    }

    // 模拟个人信息仓储
    mock! {
        PersonalInfoRepository {}
        #[async_trait]
        impl Repository<PersonalInfo> for PersonalInfoRepository {
            async fn find(&self, id: PersonalInfoId) -> Result<Option<PersonalInfo>, RepositoryError>;
            async fn save(&self, entity: &mut PersonalInfo) -> Result<PersonalInfoId, RepositoryError>;
            async fn remove(&self, aggregate: PersonalInfo) -> Result<(), RepositoryError>;
        }
        #[async_trait]
        impl PersonalInfoRepository for PersonalInfoRepository {
            async fn find_by_user_id(&self, user_id: UserId) -> Result<Option<PersonalInfo>, RepositoryError>;
        }
    }

    // 创建测试用的个人信息
    fn create_test_personal_info(id: Option<PersonalInfoId>) -> PersonalInfo {
        PersonalInfo::new(
            id,
            uuid::Uuid::new_v4(),
            RealName::try_from("张三".to_string()).unwrap(),
            IdentityCardId::try_from("110101199001011234".to_string()).unwrap(),
            PreferredSeatLocation::try_from('W').unwrap(),
            UserId::from(1),
        )
    }

    #[tokio::test]
    async fn test_get_personal_info_success() {
        let mut mock_session = MockSessionManagerService::new();
        let mut mock_personal_info_repo = MockPersonalInfoRepository::new();
        let personal_info = create_test_personal_info(Some(PersonalInfoId::from(1)));
        let user_id = UserId::from(1);
        let session_id = SessionId::random();

        // 设置会话管理器的期望
        mock_session
            .expect_get_user_id_by_session()
            .with(eq(session_id))
            .return_once(move |_| Ok(Some(user_id)));

        // 设置个人信息仓库的期望
        mock_personal_info_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .return_once(move |_| Ok(Some(personal_info.clone())));

        let service =
            PersonalInfoServiceImpl::new(Arc::new(mock_session), Arc::new(mock_personal_info_repo));

        let result = service
            .get_personal_info(PersonalInfoQuery {
                session_id: session_id.to_string(),
            })
            .await;

        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.name, "张三");
        assert_eq!(dto.identity_card_id, "110101199001011234");
        assert_eq!(dto.preferred_seat_location, "W");
    }

    #[tokio::test]
    async fn test_set_personal_info_update_success() {
        let mut mock_session = MockSessionManagerService::new();
        let mut mock_personal_info_repo = MockPersonalInfoRepository::new();
        let personal_info = create_test_personal_info(Some(PersonalInfoId::from(1)));
        let user_id = UserId::from(1);
        let session_id = SessionId::random();

        // 设置会话管理器的期望
        mock_session
            .expect_get_user_id_by_session()
            .with(eq(session_id))
            .return_once(move |_| Ok(Some(user_id)));

        // 设置个人信息查询和更新的期望
        mock_personal_info_repo
            .expect_find_by_user_id()
            .with(eq(user_id))
            .return_once(move |_| Ok(Some(personal_info)));

        mock_personal_info_repo
            .expect_save()
            .withf(|p| {
                p.name().to_string() == "李四"
                    && p.identity_card_id().to_string() == "110101199001011235"
                    && p.preferred_seat_location().to_string() == "A"
            })
            .return_once(|_| Ok(PersonalInfoId::from(1)));

        let service =
            PersonalInfoServiceImpl::new(Arc::new(mock_session), Arc::new(mock_personal_info_repo));

        let result = service
            .set_personal_info(SetPersonalInfoCommand {
                session_id: session_id.to_string(),
                name: "李四".to_string(),
                identity_card_id: "110101199001011235".to_string(),
                preferred_seat_location: "A".to_string(),
            })
            .await;

        assert!(result.is_ok());
    }
}
