//! # 交易仓储模块
//!
//! 该模块定义了火车票订购系统中的交易仓储接口。主要包含以下内容：
//!
//! - `TransactionRepository`: 异步 trait，定义了交易仓储的操作。
use crate::domain::model::transaction::Transaction;
use crate::domain::model::user::UserId;
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use rust_decimal::Decimal;
use uuid::Uuid;

/// 异步 trait，定义了交易仓储的操作。
///
/// 包含以下方法：
/// - `find_by_uuid`: 根据 UUID 查找交易。
/// - `find_by_user_id`: 根据用户 ID 查找所有交易。
/// - `get_user_balance`: 获取用户的余额。
#[async_trait]
pub trait TransactionRepository: Repository<Transaction> {
    /// 根据 UUID 查找交易。
    ///
    /// Arguments:
    /// - `uuid`: 交易的 UUID。
    ///
    /// Returns:
    /// - 成功时返回 `Option<Transaction>`，如果未找到则返回 `None`。
    /// - 失败时返回 `RepositoryError`。
    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<Transaction>, RepositoryError>;

    /// 根据用户 ID 查找所有交易。
    ///
    /// Arguments:
    /// - `user_id`: 用户的唯一标识符。
    ///
    /// Returns:
    /// - 成功时返回交易列表。
    /// - 失败时返回 `RepositoryError`。
    async fn find_by_user_id(&self, user_id: UserId) -> Result<Vec<Transaction>, RepositoryError>;

    /// 获取用户的余额。
    ///
    /// Arguments:
    /// - `user_id`: 用户的唯一标识符。
    ///
    /// Returns:
    /// - 成功时返回用户的余额，可能为空。
    /// - 失败时返回 `RepositoryError`。
    async fn get_user_balance(&self, user_id: UserId) -> Result<Option<Decimal>, RepositoryError>;
}
