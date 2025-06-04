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
use crate::domain::model::user::{IdentityCardId, RealName};
use crate::domain::repository::personal_info::PersonalInfoRepository;
use crate::domain::service::session::SessionManagerService;
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{error, instrument};

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
}

#[async_trait]
impl<S, P> PersonalInfoService for PersonalInfoServiceImpl<S, P>
where
    S: SessionManagerService + 'static + Send + Sync,
    P: PersonalInfoRepository + 'static + Send + Sync,
{
    /// 获取所有个人信息
    ///
    /// # Arguments
    /// * `query` - 包含会话ID的查询条件
    ///
    /// # Returns
    /// - `Ok(Vec<PersonalInfoDTO>)`: 成功获取的个人信息DTO列表
    /// - `Err(Box<dyn ApplicationError>)`: 错误信息
    ///
    /// # Errors
    /// - `GeneralError::InvalidSessionId`: 无效的会话ID
    /// - `GeneralError::InternalServerError`: 内部服务错误
    #[instrument(skip(self))]
    async fn get_personal_info(
        &self,
        query: PersonalInfoQuery,
    ) -> Result<Vec<PersonalInfoDTO>, Box<dyn ApplicationError>> {
        let session_id = SessionId::try_from(query.session_id.as_str())
            .map_err(|_| Box::new(GeneralError::InvalidSessionId) as Box<dyn ApplicationError>)?;

        let user_id = self
            .session_manager
            .get_user_id_by_session(session_id)
            .await
            .inspect_err(|e| {
                error!("Failed to get user ID by session: {:?} {}", session_id, e);
            })
            .map_err(|_| Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>)?
            .ok_or(Box::new(GeneralError::InvalidSessionId) as Box<dyn ApplicationError>)?;

        let personal_infos = self
            .personal_info_repository
            .find_by_user_id(user_id)
            .await
            .inspect_err(|e| {
                error!("Failed to find personal info for user {}: {:?}", user_id, e);
            })
            .map_err(|_| {
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        Ok(personal_infos
            .into_iter()
            .map(PersonalInfoDTO::from)
            .collect())
    }

    /// 设置个人信息（创建、更新或删除）
    ///
    /// # Arguments
    /// * `command` - 包含操作数据和会话ID的命令对象
    ///
    /// # Returns
    /// - `Ok(())`: 操作成功
    /// - `Err(Box<dyn ApplicationError>)`: 错误信息
    ///
    /// # Errors
    /// - `GeneralError::InvalidSessionId`: 无效的会话ID
    /// - `PersonalInfoError::InvalidIdentityCardIdFormat`: 身份证号格式错误
    /// - `PersonalInfoError::InvalidIdentityCardId`: 身份证号对应的个人信息不存在
    /// - `PersonalInfoError::InvalidPreferredSeatLocation`: 无效的座位偏好
    /// - `GeneralError::InternalServerError`: 内部服务错误
    #[instrument(skip(self))]
    async fn set_personal_info(
        &self,
        command: SetPersonalInfoCommand,
    ) -> Result<(), Box<dyn ApplicationError>> {
        let session_id = SessionId::try_from(command.session_id.as_str())
            .map_err(|_| Box::new(GeneralError::InvalidSessionId) as Box<dyn ApplicationError>)?;

        let user_id = self
            .session_manager
            .get_user_id_by_session(session_id)
            .await
            .inspect_err(|e| {
                error!("Failed to get user ID by session: {:?} {}", session_id, e);
            })
            .map_err(|_| Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>)?
            .ok_or(Box::new(GeneralError::InvalidSessionId) as Box<dyn ApplicationError>)?;

        let identity_card_id = match IdentityCardId::try_from(command.identity_card_id.clone()) {
            Ok(id) => id,
            Err(_) => {
                return Err(Box::new(PersonalInfoError::InvalidIdentityCardIdFormat)
                    as Box<dyn ApplicationError>);
            }
        };

        let existing_info = self
            .personal_info_repository
            .find_by_user_id_and_identity_card(user_id, identity_card_id.clone())
            .await
            .inspect_err(|e| {
                error!(
                    "Failed to find personal info for user {} with ID {}: {:?}",
                    user_id, identity_card_id, e
                );
            })
            .map_err(|_| {
                Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
            })?;

        if command.is_delete_operation() {
            // 删除操作
            if let Some(info) = existing_info {
                self.personal_info_repository
                    .remove(info)
                    .await
                    .map_err(|_| {
                        Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                    })?;
            } else {
                return Err(
                    Box::new(PersonalInfoError::InvalidIdentityCardId) as Box<dyn ApplicationError>
                );
            }
            return Ok(());
        }

        // 创建或更新操作
        if command.is_update_operation() {
            // SAFETY：`is_update_operation`确保了`name.is_some()`以及`default.is_some()`为真
            let name = command.name.unwrap();
            let is_default = command.default.unwrap();

            let real_name = match RealName::try_from(name) {
                Ok(name) => name,
                Err(_) => {
                    return Err(Box::new(GeneralError::BadRequest("Invalid name".into())));
                }
            };

            let preferred_seat_location = match command.preferred_seat_location {
                Some(ref location) if !location.is_empty() => Some(
                    PreferredSeatLocation::try_from(location.chars().next().unwrap()).map_err(
                        |e| {
                            error!("Invalid preferred seat location: {:?}", e);
                            Box::new(PersonalInfoError::InvalidPreferredSeatLocation)
                                as Box<dyn ApplicationError>
                        },
                    )?,
                ),
                _ => None,
            };

            if let Some(mut info) = existing_info {
                // 更新现有个人信息
                info.set_name(real_name);
                info.set_preferred_seat_location(preferred_seat_location);
                info.set_default(is_default);

                self.personal_info_repository
                    .save(&mut info)
                    .await
                    .inspect_err(|e| {
                        error!("Failed to update personal info: {:?}", e);
                    })
                    .map_err(|_| {
                        Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                    })?;
            } else {
                // 创建新的个人信息
                let mut new_info = PersonalInfo::new(
                    None,
                    uuid::Uuid::new_v4(),
                    real_name,
                    identity_card_id,
                    preferred_seat_location,
                    user_id,
                );
                new_info.set_default(is_default);

                self.personal_info_repository
                    .save(&mut new_info)
                    .await
                    .inspect_err(|e| {
                        error!("Failed to save new personal info: {:?}", e);
                    })
                    .map_err(|_| {
                        Box::new(GeneralError::InternalServerError) as Box<dyn ApplicationError>
                    })?;
            }
            return Ok(());
        }

        // 如果没有匹配的操作类型，返回错误
        return Err(Box::new(GeneralError::BadRequest(
            "Invalid operation parameters".into(),
        )));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Repository;
    use crate::domain::RepositoryError;
    use crate::domain::model::personal_info::{PersonalInfo, PersonalInfoId};
    use crate::domain::model::session::Session;
    use crate::domain::model::user::UserId;
    use async_trait::async_trait;
    use mockall::predicate;
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
            async fn verify_session_id(&self, session_id_str: &str) -> Result<bool, RepositoryError>;
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
            async fn find_by_user_id(&self, user_id: UserId) -> Result<Vec<PersonalInfo>, RepositoryError>;
            async fn find_by_user_id_and_identity_card(&self, user_id: UserId, identity_card_id: IdentityCardId) -> Result<Option<PersonalInfo>, RepositoryError>;
        }
    }

    // 创建测试用的个人信息
    fn create_test_personal_info(id: Option<PersonalInfoId>) -> PersonalInfo {
        PersonalInfo::new(
            id,
            uuid::Uuid::new_v4(),
            RealName::try_from("高松灯".to_string()).unwrap(),
            IdentityCardId::try_from("110101200903149273".to_string()).unwrap(),
            Some(PreferredSeatLocation::try_from('A').unwrap()),
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
            .return_once(move |_| Ok(vec![personal_info.clone()]));

        let service =
            PersonalInfoServiceImpl::new(Arc::new(mock_session), Arc::new(mock_personal_info_repo));

        let result = service
            .get_personal_info(PersonalInfoQuery {
                session_id: session_id.to_string(),
            })
            .await;

        assert!(result.is_ok());
        let dtos = result.unwrap();
        assert_eq!(dtos.len(), 1);
        let dto = &dtos[0];
        assert_eq!(dto.name, "高松灯");
        assert_eq!(dto.identity_card_id, "110101200903149273");
        assert_eq!(dto.preferred_seat_location.as_ref().unwrap(), "A");
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
            .expect_find_by_user_id_and_identity_card()
            .with(eq(user_id), predicate::always())
            .return_once(move |_, _| Ok(Some(personal_info)));

        mock_personal_info_repo
            .expect_save()
            .withf(|p| {
                p.name().to_string() == "要乐奈"
                    && p.identity_card_id().to_string() == "110101200903149273"
                    && p.preferred_seat_location() == Some(PreferredSeatLocation::A)
            })
            .return_once(|_| Ok(PersonalInfoId::from(1)));

        let service =
            PersonalInfoServiceImpl::new(Arc::new(mock_session), Arc::new(mock_personal_info_repo));

        let result = service
            .set_personal_info(SetPersonalInfoCommand {
                session_id: session_id.to_string(),
                name: Some("要乐奈".to_string()),
                identity_card_id: "110101200903149273".to_string(),
                preferred_seat_location: Some("A".to_string()),
                default: Some(true),
            })
            .await;

        assert!(result.is_ok());
    }
}
