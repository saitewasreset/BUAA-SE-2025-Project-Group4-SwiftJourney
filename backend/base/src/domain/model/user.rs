//! 用户领域模型模块
//!
//! 本模块实现了领域驱动设计(DDD)中的用户相关实体和值对象，
//! 包含用户核心实体、身份验证信息、个人资料等核心领域概念。
//!
//! # 主要组件
//!
//! - **实体(Entities)**:
//!   - [`User`][]: 系统用户的核心实体，包含认证信息和基本资料
//!   - [`UserInfo`][]: 用户的详细个人信息
//!
//! - **值对象(Value Objects)**:
//!   - [`UserId`][]: 用户的唯一标识符
//!   - [`Gender`][]: 性别枚举
//!   - [`Age`][]: 年龄值对象，带有验证逻辑
//!   - [`Phone`][]: 已验证的手机号码
//!   - [`IdentityCardId`][]: 符合中国标准的身份证号码
//!   - [`PasswordAttempts`][]: 密码错误尝试计数器
//!
//! - **错误类型**:
//!   - 为各值对象提供了详细的错误枚举类型
//!
//! # 设计原则
//!
//! 1. **强类型验证**：所有值对象都通过构造函数或TryFrom实现验证逻辑
//! 2. **不变性保证**：核心字段均为私有，通过方法提供访问
//! 3. **领域完整性**：封装了业务规则如密码尝试次数限制、身份证验证等
//!
//! # Examples
//!
//! 创建用户基本示例：
//!
//! ```
//! # use base::domain::model::user::{User, UserInfo, Phone, IdentityCardId};
//! # use std::convert::TryFrom;
//! # use argon2::password_hash::PasswordHashString;
//!
//! // 创建值对象
//! let phone = Phone::try_from("13812345678".to_string()).unwrap();
//! let id_card = IdentityCardId::try_from("11010519491231002X".to_string()).unwrap();
//!
//! // 构建用户信息
//! let info = UserInfo::new(
//!     "张三".to_string(),
//!     None,
//!     None,
//!     phone,
//!     None,
//!     id_card,
//! );
//!
//! // 创建用户实体
//! let user = User::new(
//!     None,
//!     "Scout".to_string(),
//!     PasswordHashString::new("$argon2id$v=19$m=65536,t=2,p=1$gZiV/M1gPc22ElAH/Jh1Hw$CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno").unwrap(),
//!     PasswordHashString::new("$argon2id$v=19$m=65536,t=2,p=1$gZiV/M1gPc22ElAH/Jh1Hw$CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno").unwrap(),
//!     0.try_into().unwrap(),
//!     info,
//! );
//! ```
//!
//! # 安全性考虑
//!
//! - 密码始终以哈希形式存储
//! - 敏感信息如身份证号、手机号有严格格式验证
//! - 密码尝试次数有上限控制
use crate::domain::{Aggregate, Entity, Identifiable, Identifier};
use argon2::password_hash::PasswordHashString;
use email_address::EmailAddress;
use shared::{PHONE_PREFIX_SET, PHONE_REGEX};
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(u64);

impl Identifier for UserId {}

impl From<u64> for UserId {
    fn from(value: u64) -> Self {
        UserId(value)
    }
}

impl From<UserId> for u64 {
    fn from(value: UserId) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gender {
    Male,
    Female,
}

#[derive(Error, Debug)]
pub enum GenderError {
    #[error("invalid gender: {0}")]
    InvalidGender(String),
}

impl Display for Gender {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Female => write!(f, "female"),
            Self::Male => write!(f, "male"),
        }
    }
}

impl TryFrom<&str> for Gender {
    type Error = GenderError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "female" => Ok(Self::Female),
            "male" => Ok(Self::Male),
            _ => Err(GenderError::InvalidGender(value.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Age(u16);

impl Display for Age {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Error, Debug)]
pub enum AgeError {
    #[error("age cannot be negative")]
    NegativeValue,
    #[error("age exceeds maximum allowed value: ({0})")]
    ExceedsMaximum(u16),
}

impl TryFrom<i32> for Age {
    type Error = AgeError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(AgeError::NegativeValue);
        }

        if value > u16::MAX as i32 {
            return Err(AgeError::ExceedsMaximum(u16::MAX));
        }

        let value = value as u16;

        if value > 200 {
            return Err(AgeError::ExceedsMaximum(value));
        }

        Ok(Age(value))
    }
}

impl From<Age> for u16 {
    fn from(value: Age) -> Self {
        value.0
    }
}

impl From<Age> for i32 {
    fn from(value: Age) -> Self {
        value.0 as i32
    }
}

/// 表示一个有效的手机号码。
///
/// `Phone` 是一个包装字符串的类型，确保手机号码符合以下规则：
/// - 长度必须为 11 个字符。
/// - 必须符合正则表达式 `^1[3-9]\d{9}$` 定义的格式。
/// - 必须以有效的前缀开头（例如 "130", "131", "132"）。
///
/// # Examples
///
/// ```
/// use base::domain::model::user::Phone;
/// use std::convert::TryFrom;
///
/// let phone = Phone::try_from("13012345678".to_string()).unwrap();
/// assert_eq!(phone.to_string(), "13012345678");
/// ```

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Phone(String);

impl Display for Phone {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
// allow说明： enum_variant_names不建议Enum的所有变体含有相同的前缀/后缀（Invalid）
// 但，在用于表达校验错误时，错误类别以Invalid开头应当是合法的
/// 表示验证手机号码时可能发生的错误。
pub enum PhoneError {
    /// 手机号码长度无效。
    ///
    /// 期望的长度为 11 个字符。
    #[error("invalid phone number length, expected: 11, got: {0}")]
    InvalidLength(usize),
    /// 手机号码格式无效。
    ///
    /// 手机号码必须符合正则表达式 `^1[3-9]\d{9}$` 定义的格式。.
    #[error("invalid phone number format")]
    InvalidFormat,
    /// 手机号码前缀无效。
    ///
    /// 手机号码必须以有效的前缀开头（例如 "130", "131", "132"）。
    #[error("invalid phone number prefix: {0}")]
    InvalidPrefix(String),
}

/// 尝试将 `String` 转换为 `Phone`。
///
/// 输入的字符串必须满足以下条件：
/// - 长度必须为 11 个字符。
/// - 必须符合正则表达式 `^1[3-9]\d{9}$` 定义的格式。
/// - 必须以有效的前缀开头（例如 "130", "131", "132"）。
///
/// # Errors
///
/// 如果输入字符串不符合条件，则返回 `PhoneError`。
///
/// # Examples
///
/// ```
/// use base::domain::model::user::{Phone, PhoneError};
/// use std::convert::TryFrom;
///
/// let result = Phone::try_from("15412345678".to_string());
/// assert!(matches!(result, Err(PhoneError::InvalidPrefix(_))));
/// ```
impl TryFrom<String> for Phone {
    type Error = PhoneError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() != 11 {
            return Err(PhoneError::InvalidLength(value.len()));
        }

        if !PHONE_REGEX.is_match(&value) {
            return Err(PhoneError::InvalidFormat);
        }

        // 若手机号长度为11，则应当能取出前3位的前缀而不引发越界
        if !PHONE_PREFIX_SET.contains(&value[0..3]) {
            return Err(PhoneError::InvalidPrefix(value[0..3].to_string()));
        }

        Ok(Phone(value))
    }
}

/// 将 `Phone` 转换为 `String`。
///
/// # Examples
///
/// ```
/// use base::domain::model::user::Phone;
/// use std::convert::TryFrom;
///
/// let phone = Phone::try_from("13012345678".to_string()).unwrap();
/// let phone_str: String = phone.into();
/// assert_eq!(phone_str, "13012345678");
/// ```
impl From<Phone> for String {
    fn from(value: Phone) -> Self {
        value.0
    }
}

/// 将 `Phone` 的引用转换为字符串切片。
///
/// # Examples
///
/// ```
/// use base::domain::model::user::Phone;
/// use std::convert::TryFrom;
///
/// let phone = Phone::try_from("13012345678".to_string()).unwrap();
/// let phone_str: &str = (&phone).into();
/// assert_eq!(phone_str, "13012345678");
/// ```
impl<'a> From<&'a Phone> for &'a str {
    fn from(value: &'a Phone) -> Self {
        &value.0
    }
}

#[derive(Error, Debug)]
pub enum IdentityCardError {
    #[error("invalid identity card length, expected: 18, got: {0}")]
    InvalidLength(usize),
    #[error("invalid identity card format")]
    InvalidFormat,
    #[error("invalid identity card check code, expected: {0}, got: {1}")]
    InvalidCheckCode(char, char),
}

/// 中国居民身份证号码
///
/// 表示一个符合GB11643-1999标准的18位中国居民身份证号码。
/// 身份证号码包含地区码、出生日期、顺序码和校验码等信息。
///
/// # 格式说明
/// - 前6位：行政区划代码
/// - 中间8位：出生日期（YYYYMMDD格式）
/// - 后3位：顺序码（最后一位奇数表示男性，偶数表示女性）
/// - 最后1位：校验码（0-9或X）

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdentityCardId(String);

impl Display for IdentityCardId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl IdentityCardId {
    /// 验证给定的字符串是否为有效的中国身份证号码
    ///
    /// 检查内容包括：
    /// - 长度是否为18位
    /// - 前17位是否为数字
    /// - 最后一位是否为数字或X/x
    /// - 校验码是否正确
    ///
    /// # Arguments
    /// * `id` - 待验证的身份证号码字符串
    ///
    /// # Returns
    /// 如果验证通过返回`Ok(())`，否则返回相应的错误
    ///
    /// # Examples
    /// ```
    /// use base::domain::model::user::IdentityCardId;
    ///
    /// // 有效的身份证号码
    /// assert!(IdentityCardId::is_valid_china_id("11010519491231002X").is_ok());
    ///
    /// // 无效的长度
    /// assert!(IdentityCardId::is_valid_china_id("12345").is_err());
    ///
    /// // 无效的格式
    /// assert!(IdentityCardId::is_valid_china_id("1101051949123100XX").is_err());
    /// ```
    ///
    /// # Errors
    /// - `InvalidLength` - 当身份证号码长度不是18位时
    /// - `InvalidFormat` - 当身份证号码格式不正确时
    /// - `InvalidCheckCode` - 当校验码不匹配时
    pub fn is_valid_china_id(id: &str) -> Result<(), IdentityCardError> {
        // 检查长度是否为18位
        if id.len() != 18 {
            return Err(IdentityCardError::InvalidLength(id.len()));
        }

        // 前17位必须是数字
        if !id[..17].chars().all(|c| c.is_ascii_digit()) {
            return Err(IdentityCardError::InvalidFormat);
        }

        // 第18位可以是数字或X/x
        let last_char = id.chars().last().unwrap();
        if !(last_char.is_ascii_digit() || last_char == 'X' || last_char == 'x') {
            return Err(IdentityCardError::InvalidFormat);
        }

        // 计算校验码
        let weights = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
        let check_codes = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2'];

        let sum: u32 = id[..17]
            .chars()
            .map(|c| c.to_digit(10).unwrap())
            .zip(weights.iter())
            .map(|(digit, weight)| digit * weight)
            .sum();

        let computed_check_code = check_codes[(sum % 11) as usize];

        // 比较校验码（不区分大小写）
        if computed_check_code != last_char.to_ascii_uppercase() {
            Err(IdentityCardError::InvalidCheckCode(
                computed_check_code.to_ascii_uppercase(),
                last_char.to_ascii_uppercase(),
            ))
        } else {
            Ok(())
        }
    }
}

impl TryFrom<String> for IdentityCardId {
    type Error = IdentityCardError;
    /// 尝试从字符串创建身份证号码
    ///
    /// 首先验证字符串是否符合身份证号码格式要求，验证通过后创建`IdentityCardId`
    ///
    /// # Examples
    /// ```
    /// use std::convert::TryFrom;
    /// use base::domain::model::user::IdentityCardId;
    ///
    /// let id = IdentityCardId::try_from("11010519491231002X".to_string());
    /// assert!(id.is_ok());
    ///
    /// let id = IdentityCardId::try_from("invalid".to_string());
    /// assert!(id.is_err());
    /// ```
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::is_valid_china_id(&value)?;

        Ok(IdentityCardId(value))
    }
}

impl From<IdentityCardId> for String {
    /// 将身份证号码转换回字符串
    ///
    /// # Examples
    /// ```
    /// use std::convert::TryFrom;
    /// use base::domain::model::user::IdentityCardId;
    ///
    /// let id = IdentityCardId::try_from("11010519491231002X".to_string()).unwrap();
    /// let s: String = id.into();
    /// assert_eq!(s, "11010519491231002X");
    /// ```
    fn from(value: IdentityCardId) -> Self {
        value.0
    }
}

impl<'a> From<&'a IdentityCardId> for &'a str {
    /// 获取身份证号码的字符串切片
    ///
    /// # Examples
    /// ```
    /// use std::convert::TryFrom;
    /// use base::domain::model::user::IdentityCardId;
    ///
    /// let id = IdentityCardId::try_from("11010519491231002X".to_string()).unwrap();
    /// let s: &str = (&id).into();
    /// assert_eq!(s, "11010519491231002X");
    /// ```
    fn from(value: &'a IdentityCardId) -> Self {
        &value.0
    }
}

#[derive(Error, Debug)]
pub enum PasswordError {
    #[error("invalid password")]
    InvalidPassword,
    #[error("max attempts of {0} exceed")]
    MaxAttemptsExceeded(u8),
}

/// 密码尝试次数相关的错误类型
#[derive(Error, Debug)]
pub enum PasswordAttemptsError {
    /// 输入值超过最大尝试次数限制
    #[error("input {0} exceed max attempts {1}")]
    ExceedMaxAttempts(u8, u8),
    /// 输入值过大（超过u8范围）
    #[error("value: {0} is too large")]
    ValueTooLarge(i32),
    /// 输入值为负数
    #[error("password attempts cannot be negative: {0}")]
    NegativeValue(i32),
}

/// 密码错误尝试次数
///
/// 该类型用于安全地管理密码验证时的错误尝试次数，
/// 确保次数不会超过最大限制且不会出现非法值。
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PasswordAttempts(u8);

impl PasswordAttempts {
    /// 允许的最大密码尝试次数
    pub const MAX: u8 = 5;

    /// 创建一个新的密码尝试计数器，初始值为0
    ///
    /// # Examples
    /// ```
    /// # use base::domain::model::user::PasswordAttempts;
    /// let attempts = PasswordAttempts::new();
    /// assert_eq!(u8::from(attempts), 0);
    /// ```
    pub fn new() -> Self {
        Self(0)
    }

    /// 增加尝试次数
    ///
    /// 如果当前次数已经达到最大值，则返回错误
    ///
    /// # Errors
    /// 当尝试次数超过最大值时返回`PasswordError::MaxAttemptsExceeded`
    ///
    /// # Examples
    /// ```
    /// # use base::domain::model::user::PasswordAttempts;
    /// let mut attempts = PasswordAttempts::new();
    /// assert!(attempts.increment().is_ok());
    /// ```
    pub fn increment(&mut self) -> Result<(), PasswordError> {
        if self.0 >= Self::MAX {
            return Err(PasswordError::MaxAttemptsExceeded(Self::MAX));
        }
        self.0 += 1;
        Ok(())
    }
}

impl TryFrom<u8> for PasswordAttempts {
    type Error = PasswordAttemptsError;

    /// 从u8类型创建PasswordAttempts
    ///
    /// # Errors
    /// 如果输入值超过最大尝试次数限制，返回`PasswordAttemptsError::ExceedMaxAttempts`
    ///
    /// # Examples
    /// ```
    /// # use base::domain::model::user::PasswordAttempts;
    /// let attempts = PasswordAttempts::try_from(3);
    /// assert!(attempts.is_ok());
    /// ```
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Self::MAX {
            return Err(PasswordAttemptsError::ExceedMaxAttempts(value, Self::MAX));
        }

        Ok(PasswordAttempts(value))
    }
}

impl TryFrom<i32> for PasswordAttempts {
    type Error = PasswordAttemptsError;

    /// 从i32类型创建PasswordAttempts
    ///
    /// # Errors
    /// 如果输入值为负数，返回`PasswordAttemptsError::NegativeValue`
    /// 如果输入值超过u8范围，返回`PasswordAttemptsError::ValueTooLarge`
    /// 如果输入值超过最大尝试次数限制，返回`PasswordAttemptsError::ExceedMaxAttempts`
    ///
    /// # Examples
    /// ```
    /// # use base::domain::model::user::PasswordAttempts;
    /// let attempts = PasswordAttempts::try_from(3i32);
    /// assert!(attempts.is_ok());
    /// ```
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(PasswordAttemptsError::NegativeValue(value));
        }

        if value > u8::MAX as i32 {
            return Err(PasswordAttemptsError::ValueTooLarge(value));
        }

        PasswordAttempts::try_from(value as u8)
    }
}

impl From<PasswordAttempts> for u8 {
    /// 将PasswordAttempts转换回u8类型
    ///
    /// # Examples
    /// ```
    /// # use base::domain::model::user::PasswordAttempts;
    ///
    /// let attempts = PasswordAttempts::new();
    /// let count: u8 = attempts.into();
    /// assert_eq!(count, 0);
    /// ```
    fn from(value: PasswordAttempts) -> Self {
        value.0
    }
}

/// 表示系统用户的实体
///
/// 包含用户的基本信息、认证信息和支付密码相关信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    /// 用户ID，新用户创建时为None
    id: Option<UserId>,
    /// 用户名，用于显示
    username: String,
    /// 经过哈希处理的登录密码
    hashed_password: PasswordHashString,
    /// 经过哈希处理的支付密码
    hashed_payment_password: PasswordHashString,
    /// 支付密码错误尝试次数
    wrong_payment_password_tried: PasswordAttempts,
    /// 用户详细信息
    info: UserInfo,
}

impl User {
    /// 创建一个新的User实例
    ///
    /// # Arguments
    /// * `id` - 用户ID，新用户时为None
    /// * `username` - 用户名
    /// * `hashed_password` - 已哈希的登录密码
    /// * `hashed_payment_password` - 已哈希的支付密码
    /// * `wrong_payment_password_tried` - 支付密码错误尝试次数
    /// * `info` - 用户详细信息
    ///
    /// # Returns
    /// 返回构建好的User实例
    pub fn new(
        id: Option<UserId>,
        username: String,
        hashed_password: PasswordHashString,
        hashed_payment_password: PasswordHashString,
        wrong_payment_password_tried: PasswordAttempts,
        info: UserInfo,
    ) -> Self {
        User {
            id,
            username,
            hashed_password,
            hashed_payment_password,
            wrong_payment_password_tried,
            info,
        }
    }

    /// 获取用户名
    pub fn username(&self) -> &str {
        &self.username
    }

    /// 获取哈希后的登录密码
    pub fn hashed_password(&self) -> &PasswordHashString {
        &self.hashed_password
    }

    /// 获取哈希后的支付密码
    pub fn hashed_payment_password(&self) -> &PasswordHashString {
        &self.hashed_payment_password
    }

    /// 获取支付密码错误尝试次数
    pub fn wrong_payment_password_tried(&self) -> PasswordAttempts {
        self.wrong_payment_password_tried
    }

    /// 获取用户详细信息
    pub fn user_info(&self) -> &UserInfo {
        &self.info
    }
}

impl Identifiable for User {
    type ID = UserId;
    fn get_id(&self) -> Option<UserId> {
        self.id
    }
}

/// 支付密码
///
/// 表示一个符合业务规则的支付密码，必须是6位ASCII数字组成。
/// 该类型保证一旦创建就一定是有效格式，所有使用该类型的地方都可以信任其内容。
///
/// # Examples
///
/// ```
/// # use std::convert::TryFrom;
/// # use base::domain::model::user::PaymentPassword;
/// let password = PaymentPassword::try_from("123456").unwrap();
/// assert_eq!(String::from(password), "123456");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaymentPassword([char; 6]);

/// 支付密码相关错误类型
#[derive(Error, Debug)]
pub enum PaymentPasswordError {
    /// 密码长度不符合6位要求
    #[error("payment password must be 6 digits, got {0} characters")]
    InvalidLength(usize),
    /// 密码包含非数字字符
    #[error("payment password must contain only digits, found non-digit character")]
    NonDigitCharacter,
}

impl TryFrom<&str> for PaymentPassword {
    type Error = PaymentPasswordError;

    /// 从字符串创建支付密码
    ///
    /// 执行严格验证：
    /// 1. 必须正好6个字符长度
    /// 2. 必须全部为ASCII数字(0-9)
    ///
    /// # 参数
    /// - `value`: 待验证的密码字符串
    ///
    /// # 返回
    /// - `Ok(PaymentPassword)`: 验证通过的有效密码
    /// - `Err(PaymentPasswordError)`: 包含具体验证失败原因
    ///
    /// # Errors
    ///
    /// 返回以下错误之一：
    /// - `PaymentPasswordError::InvalidLength`: 当输入长度不为6时
    /// - `PaymentPasswordError::NonDigitCharacter`: 当包含非数字字符时
    ///
    /// # Examples
    ///
    /// ```
    /// // 有效密码
    /// # use base::domain::model::user::PaymentPassword;
    /// let password = PaymentPassword::try_from("123456").unwrap();
    ///
    /// // 无效密码
    /// assert!(PaymentPassword::try_from("12345").is_err());  // 长度不足
    /// assert!(PaymentPassword::try_from("12345a").is_err()); // 包含字母
    /// ```
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 6 {
            return Err(PaymentPasswordError::InvalidLength(value.len()));
        }

        let mut result_array: [char; 6] = ['0'; 6];

        for (idx, ch) in value.chars().enumerate() {
            if !ch.is_ascii_digit() {
                return Err(PaymentPasswordError::NonDigitCharacter);
            }

            result_array[idx] = ch;
        }

        Ok(PaymentPassword(result_array))
    }
}

impl From<PaymentPassword> for String {
    /// 将支付密码转换回字符串
    fn from(value: PaymentPassword) -> Self {
        String::from_iter(value.0)
    }
}

impl Entity for User {}

impl Aggregate for User {}
/// 用户的详细信息
///
/// 包含用户的个人身份信息和联系方式
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserInfo {
    /// 用户真实姓名
    pub name: String,
    /// 用户性别(可选)
    pub gender: Option<Gender>,
    /// 用户年龄(可选)
    pub age: Option<Age>,
    /// 用户手机号码
    pub phone: Phone,
    /// 用户电子邮箱(可选)
    pub email: Option<EmailAddress>,
    /// 用户身份证号
    pub identity_card_id: IdentityCardId,
}

impl UserInfo {
    /// 创建新的UserInfo实例
    ///
    /// # Arguments
    /// * `name` - 真实姓名
    /// * `gender` - 性别(可选)
    /// * `age` - 年龄(可选)
    /// * `phone` - 手机号码
    /// * `email` - 电子邮箱(可选)
    /// * `identity_card_id` - 身份证号
    ///
    /// # Returns
    /// 返回构建好的UserInfo实例
    pub fn new(
        name: String,
        gender: Option<Gender>,
        age: Option<Age>,
        phone: Phone,
        email: Option<EmailAddress>,
        identity_card_id: IdentityCardId,
    ) -> Self {
        UserInfo {
            name,
            gender,
            age,
            phone,
            email,
            identity_card_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};
    use std::convert::TryFrom;

    #[test]
    fn user_id_conversions() {
        let id = 42u64;
        let user_id = UserId::from(id);
        assert_eq!(u64::from(user_id), id);
    }

    mod gender {
        use super::*;
        use claims::assert_ok_eq;

        #[test]
        fn display() {
            assert_eq!(Gender::Male.to_string(), "male");
            assert_eq!(Gender::Female.to_string(), "female");
        }

        #[test]
        fn try_from_valid_str() {
            assert_ok_eq!(Gender::try_from("male"), Gender::Male);
            assert_ok_eq!(Gender::try_from("female"), Gender::Female);
        }

        #[test]
        fn try_from_invalid_str() {
            let result = Gender::try_from("For Super Earth!");
            assert!(matches!(result, Err(GenderError::InvalidGender(_))));
        }
    }

    mod age {
        use super::*;

        #[test]
        fn try_from_valid() {
            assert_ok!(Age::try_from(0));
            assert_ok!(Age::try_from(200));
            assert_ok!(Age::try_from(25));
        }

        #[test]
        fn try_from_negative() {
            let result = Age::try_from(-1);
            assert!(matches!(result, Err(AgeError::NegativeValue)));
        }

        #[test]
        fn try_from_exceed() {
            let result = Age::try_from(300);
            assert!(matches!(result, Err(AgeError::ExceedsMaximum(300))));
        }

        #[test]
        fn try_from_overflow() {
            let result = Age::try_from(u8::MAX as i32 + 233);
            assert!(matches!(result, Err(AgeError::ExceedsMaximum(_))));
        }

        #[test]
        fn display() {
            let age = Age::try_from(30).unwrap();
            assert_eq!(age.to_string(), "30");
        }
    }

    mod phone {
        use super::*;

        #[test]
        fn valid_phone() {
            let phone = Phone::try_from("13012345678".to_string());
            assert_ok!(phone);
        }

        #[test]
        fn invalid_length() {
            let cases = vec!["1", "123456789", "123456789012"];
            for case in cases {
                let result = Phone::try_from(case.to_string());
                assert!(matches!(result, Err(PhoneError::InvalidLength(_))));
            }
        }

        #[test]
        fn invalid_format() {
            let cases = vec![
                "12012345678", // 第二位不是3-9
                "10012345678", // 第二位是0
                "1a012345678", // 包含非数字
            ];
            for case in cases {
                let result = Phone::try_from(case.to_string());
                assert!(matches!(result, Err(PhoneError::InvalidFormat)));
            }
        }

        #[test]
        fn invalid_prefix() {
            let result = Phone::try_from("15412345678".to_string());
            assert!(matches!(result, Err(PhoneError::InvalidPrefix(_))));
        }
    }

    mod identity_card {
        use super::*;

        #[test]
        fn valid_id() {
            let valid_ids = vec![
                "110108197502157336", // 正确校验码
                "110108200811088252", // 正确校验码
            ];
            for id in valid_ids {
                assert_ok!(IdentityCardId::try_from(id.to_string()));
            }
        }

        #[test]
        fn invalid_length() {
            let cases = vec!["12345", "1234567890123456789"];
            for case in cases {
                let result = IdentityCardId::try_from(case.to_string());
                assert!(matches!(result, Err(IdentityCardError::InvalidLength(_))));
            }
        }

        #[test]
        fn invalid_format() {
            let cases = vec![
                "1101051949a231002X", // 前17位包含字母
                "11010519491231002#", // 非法结尾字符
            ];
            for case in cases {
                let result = IdentityCardId::try_from(case.to_string());

                assert!(matches!(result, Err(IdentityCardError::InvalidFormat)))
            }
        }

        #[test]
        fn invalid_check_code() {
            let case = "110105194912310020"; // 错误校验码
            let result = IdentityCardId::try_from(case.to_string());
            assert!(matches!(
                result,
                Err(IdentityCardError::InvalidCheckCode(_, _))
            ))
        }
    }

    mod password_attempts {
        use super::*;

        #[test]
        fn new() {
            let attempts = PasswordAttempts::new();
            assert_eq!(u8::from(attempts), 0);
        }

        #[test]
        fn increment() {
            let mut attempts = PasswordAttempts::new();
            for _ in 0..5 {
                assert_ok!(attempts.increment());
            }
            assert_err!(attempts.increment());
            assert!(matches!(
                attempts.increment(),
                Err(PasswordError::MaxAttemptsExceeded(5))
            ))
        }

        #[test]
        fn try_from_valid_u8() {
            assert_ok!(PasswordAttempts::try_from(0));
            assert_ok!(PasswordAttempts::try_from(5));
        }

        #[test]
        fn try_from_invalid_u8() {
            let result = PasswordAttempts::try_from(6);
            assert!(matches!(
                result,
                Err(PasswordAttemptsError::ExceedMaxAttempts(6, 5))
            ))
        }

        #[test]
        fn try_from_negative_i32() {
            let result = PasswordAttempts::try_from(-1);
            assert!(matches!(
                result,
                Err(PasswordAttemptsError::NegativeValue(-1))
            ))
        }
    }

    mod payment_password {
        use super::*;
        use claims::{assert_err, assert_ok}; // 可以使用 claims crate 更清晰的断言

        /// 测试有效支付密码的创建
        #[test]
        fn valid_payment_password() {
            let cases = ["123456", "000000", "999999", "010203"];

            for &input in &cases {
                let password = PaymentPassword::try_from(input);
                assert_ok!(&password);

                // 验证内容是否正确存储
                let password = password.unwrap();
                assert_eq!(String::from(password), input);
            }
        }

        /// 测试长度不符合要求的输入
        #[test]
        fn invalid_length() {
            let cases = [
                ("", 0),        // 空字符串
                ("1", 1),       // 过短
                ("12345", 5),   // 少一位
                ("1234567", 7), // 多一位
            ];

            for &(input, expected_len) in &cases {
                let err = PaymentPassword::try_from(input).unwrap_err();
                match err {
                    PaymentPasswordError::InvalidLength(len) => assert_eq!(len, expected_len),
                    _ => panic!("Expected InvalidLength error"),
                }
            }
        }

        /// 测试非数字字符输入
        #[test]
        fn non_digit_characters() {
            let cases = [
                "a23456",       // 字母开头
                "1b3456",       // 字母中间
                "12345c",       // 字母结尾
                "123-56",       // 特殊字符
                "１２３４５６", // 全角数字(应该失败)
            ];

            for &input in &cases {
                let err = PaymentPassword::try_from(input).unwrap_err();
                assert!(
                    matches!(err, PaymentPasswordError::NonDigitCharacter),
                    "Input: {} should trigger NonDigitCharacter error",
                    input
                );
            }
        }

        /// 测试类型转换
        #[test]
        fn string_conversion() {
            let input = "654321";
            let password = PaymentPassword::try_from(input).unwrap();
            assert_eq!(String::from(password), input);
        }
    }
}
