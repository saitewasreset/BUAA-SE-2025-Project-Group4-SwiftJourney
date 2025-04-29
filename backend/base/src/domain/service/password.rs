//! 密码领域服务层抽象
//!
//! 提供密码处理的领域服务接口，作为协调以下操作的统一门面：
//! - 盐值生成
//! - 密码哈希
//! - 密码验证
//!
//! 本模块不包含具体实现，具体算法由基础设施层提供。
//!
//! # 架构角色
//! - 领域服务层：封装密码处理的领域逻辑
//! - 防腐层：隔离领域模型与具体技术实现
//!
//! # Examples
//! ```
//! # use base::domain::model::password::{HashedPassword, PasswordSalt};
//! # use base::domain::service::password::PasswordService;
//! struct MockService;
//! impl PasswordService for MockService {
//!     /* 实现略 */
//!     fn generate_salt() -> PasswordSalt {
//!         todo!()
//!     }
//!
//!     fn hash_password(raw_password: &[u8], salt: PasswordSalt) -> anyhow::Result<HashedPassword> {
//!         todo!()
//!     }
//!
//!     fn verify(raw_password: &[u8], hashed_password: HashedPassword) -> anyhow::Result<bool> {
//!         todo!()
//!     }
//!     
//! }
//! ```

use crate::domain::model::password::{HashedPassword, PasswordSalt};

/// 密码领域服务接口
///
/// 定义密码处理的核心领域操作，作为：
/// - 应用层调用的统一接口
/// - 领域模型的技术无关抽象
///
/// # 实现要求
/// - 所有方法必须线程安全
/// - 应委托给 `PasswordSaltGenerator` 和 `PasswordHasher` 执行具体操作
pub trait PasswordService {
    /// 生成新的密码盐值
    ///
    /// # Notes
    /// 实际委托给 `PasswordSaltGenerator` 实现
    fn generate_salt() -> PasswordSalt;

    /// 计算密码哈希值
    ///
    /// # Arguments
    /// - `raw_password`: 原始密码字节
    /// - `salt`: 使用的盐值
    ///
    /// # Errors
    /// - 当哈希计算失败时返回错误
    ///
    /// # Performance
    /// 此操作设计为CPU密集型，不建议在异步上下文中使用
    fn hash_password(raw_password: &[u8], salt: PasswordSalt) -> anyhow::Result<HashedPassword>;

    /// 验证密码是否正确
    ///
    /// # Errors
    /// - 当验证过程出现异常时返回错误
    fn verify(raw_password: &[u8], hashed_password: HashedPassword) -> anyhow::Result<bool>;
}
