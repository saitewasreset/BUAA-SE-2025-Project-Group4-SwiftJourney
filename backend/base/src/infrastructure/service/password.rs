//! 密码服务基础设施实现
//!
//! 基于 Argon2 算法提供密码处理的实现：
//! - 密码盐值生成
//! - 密码哈希计算
//! - 密码验证
//!
//! # 实现说明
//! - 使用 `argon2` 作为默认密码哈希算法
//! - 集成 OS 随机数生成器作为熵源
//! - 自动配置安全的工作参数
//!
//! # Security
//! - 抵抗 GPU/ASIC 破解

use crate::domain::model::password::{
    HashedPassword, PasswordHasher, PasswordSalt, PasswordSaltGenerator,
};
use crate::domain::service::password::PasswordService;
use anyhow::Context;

use argon2::PasswordVerifier;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;

/// Argon2 盐值生成器实现
///
/// 使用操作系统提供的密码学安全随机数生成器(CSPRNG)生成盐值
///
/// # Implementation Details
/// - 内部使用 `SaltString::generate`
pub struct Argon2PasswordSaltGenerator;
/// Argon2 密码哈希器实现
///
/// 提供符合当前安全最佳实践的密码哈希功能：
/// - 自动内存硬化
/// - 抵抗旁路攻击
/// - 可配置工作因子
pub struct Argon2PasswordHasher;

impl PasswordSaltGenerator for Argon2PasswordSaltGenerator {
    /// 生成新的密码盐值
    ///
    /// # Panics
    /// - 当系统随机数源不可用时可能 panic
    fn generate() -> PasswordSalt {
        SaltString::generate(&mut OsRng)
            .to_string()
            .as_bytes()
            .to_owned()
            .into()
    }
}

impl PasswordHasher for Argon2PasswordHasher {
    /// 执行密码哈希计算
    ///
    /// # Arguments
    /// - `raw_password`: 原始密码字节
    /// - `salt`: 密码盐值
    ///
    /// # Errors
    /// - 当输入盐值格式无效时返回错误
    /// - 当哈希计算失败时返回错误
    fn hash(raw_password: &[u8], salt: PasswordSalt) -> anyhow::Result<HashedPassword> {
        use argon2::PasswordHasher;

        let _salt = salt.clone();

        let salt = String::from_utf8(Vec::from(salt))
            .context("Failed to convert PasswordSalt to String")?;

        let salt: argon2::password_hash::Salt = salt.as_str().try_into().map_err(|e| {
            anyhow::anyhow!(
                "Failed to convert PasswordSalt to argon2::password_hash::Salt: {}",
                e
            )
        })?;

        let hashed_password = argon2::Argon2::default()
            .hash_password(raw_password, salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
            .to_string()
            .as_bytes()
            .to_owned();

        Ok(HashedPassword {
            hashed_password,
            salt: _salt,
        })
    }

    /// 验证密码匹配性
    ///
    /// # Errors
    /// - 当哈希值格式无效时返回错误
    fn verify(raw_password: &[u8], hashed_password: HashedPassword) -> anyhow::Result<bool> {
        let password_hash: String = String::from_utf8(hashed_password.hashed_password)
            .context("Failed to convert HashedPassword to String")?;

        let password_hash: argon2::PasswordHash = argon2::PasswordHash::new(password_hash.as_str())
            .map_err(|e| {
                anyhow::anyhow!(
                    "Failed to convert HashedPassword to argon2::PasswordHash: {}",
                    e
                )
            })?;

        Ok(argon2::Argon2::default()
            .verify_password(raw_password, &password_hash)
            .is_ok())
    }
}

/// 密码服务具体实现
///
/// 组合盐值生成器和哈希器的门面服务，提供：
/// - 统一的密码操作接口
/// - 类型安全的配置组合
///
/// # Type Parameters
/// - `G`: 盐值生成器实现
/// - `H`: 密码哈希器实现
pub struct PasswordServiceImpl<G, H>
where
    G: PasswordSaltGenerator,
    H: PasswordHasher,
{
    salt_generator: G,
    hasher: H,
}

impl<G, H> PasswordService for PasswordServiceImpl<G, H>
where
    G: PasswordSaltGenerator,
    H: PasswordHasher,
{
    fn generate_salt() -> PasswordSalt {
        G::generate()
    }

    fn hash_password(raw_password: &[u8], salt: PasswordSalt) -> anyhow::Result<HashedPassword> {
        H::hash(raw_password, salt)
    }

    fn verify(raw_password: &[u8], hashed_password: HashedPassword) -> anyhow::Result<bool> {
        H::verify(raw_password, hashed_password)
    }
}

/// 预配置的 Argon2 密码服务类型别名
///
/// 组合了：
/// - `Argon2PasswordSaltGenerator`
/// - `Argon2PasswordHasher`
pub type Argon2PasswordServiceImpl =
    PasswordServiceImpl<Argon2PasswordSaltGenerator, Argon2PasswordHasher>;
