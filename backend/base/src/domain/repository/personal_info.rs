//! 个人信息仓储接口模块
//!
//! 该模块定义了个人信息实体的仓储接口，提供领域层对个人信息数据的访问抽象。
//! 接口设计遵循仓储模式，分离领域逻辑和数据访问细节。

use crate::domain::model::personal_info::PersonalInfo;
use crate::domain::model::user::UserId;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;

/// 个人信息仓储接口
///
/// 定义个人信息的持久化操作，包括基本的CRUD和领域特有的查询方法。
/// 实现该接口的结构体负责处理个人信息数据的实际存取。
///
/// # 方法
/// - `find_by_user_id`: 根据用户ID查询个人信息
#[async_trait]
pub trait PersonalInfoRepository: Repository<PersonalInfo> {
    /// 根据用户ID查询个人信息
    ///
    /// # Arguments
    /// * `user_id` - 用户ID
    ///
    /// # Returns
    /// * `Ok(Some(PersonalInfo))` - 查询成功且找到个人信息
    /// * `Ok(None)` - 查询成功但未找到个人信息
    /// * `Err(RepositoryError)` - 查询失败
    async fn find_by_user_id(
        &self,
        user_id: UserId,
    ) -> Result<Option<PersonalInfo>, RepositoryError>;
}
