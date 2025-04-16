//! 密码领域模型和核心抽象
//!
//! 本模块定义了密码处理相关的领域原语和核心特征，包括：
//! - 密码哈希与验证抽象
//! - 盐值生成抽象
//! - 密码盐和哈希密码的Domain Primitive

/// 密码盐值生成器特征
///
/// 定义生成密码盐值的标准接口，所有具体实现应：
/// - 生成密码学安全的随机盐值
/// - 满足最小长度要求(推荐32字节以上)
pub trait PasswordSaltGenerator {
    /// 生成新的密码盐值
    fn generate() -> PasswordSalt;
}

/// 密码哈希器特征
///
/// 定义密码哈希和验证的标准接口，实现应当：
/// - 使用抗ASIC/GPU的哈希算法(如Argon2)
/// - 内置合理的默认工作因子
pub trait PasswordHasher {
    /// 对原始密码进行哈希处理
    ///
    /// # Arguments
    /// - `raw_password`: 用户提供的原始密码字节
    /// - `salt`: 预先生成的密码盐值
    ///
    /// # Errors
    /// - 当哈希操作失败时返回错误(如内存不足)
    fn hash(raw_password: &[u8], salt: PasswordSalt) -> anyhow::Result<HashedPassword>;

    /// 验证密码是否匹配
    ///
    /// # Arguments
    /// - `raw_password`: 用户提供的原始密码字节
    /// - `hashed_password`: 加盐存储的密码以及盐值
    ///
    /// # Errors
    /// - 当验证过程出现错误时返回(如哈希值格式无效)
    fn verify(raw_password: &[u8], hashed_password: HashedPassword) -> anyhow::Result<bool>;
}

/// 密码盐值Domain Primitive
///
/// # Examples
/// ```
/// use base::domain::model::password::PasswordSalt;
/// let salt = PasswordSalt::from(vec![0u8; 32]);
/// assert_eq!(Vec::from(salt).len(), 32);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PasswordSalt(Vec<u8>);

impl From<Vec<u8>> for PasswordSalt {
    fn from(value: Vec<u8>) -> Self {
        PasswordSalt(value)
    }
}

impl From<PasswordSalt> for Vec<u8> {
    fn from(value: PasswordSalt) -> Self {
        value.0
    }
}

impl<'a> From<&'a PasswordSalt> for &'a [u8] {
    fn from(value: &'a PasswordSalt) -> Self {
        &value.0
    }
}

/// 哈希密码值对象
///
/// 包含：
/// - 计算后的密码哈希值
/// - 使用的盐值

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashedPassword {
    /// 密码哈希值(二进制格式)
    pub hashed_password: Vec<u8>,
    /// 使用的盐值
    pub salt: PasswordSalt,
}
