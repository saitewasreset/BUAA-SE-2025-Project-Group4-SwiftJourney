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

#[derive(Error, Debug)]
pub enum PasswordAttemptsError {
    #[error("input {0} exceed max attempts {1}")]
    ExceedMaxAttempts(u8, u8),
    #[error("password attempts cannot be negative: {0}")]
    NegativeValue(i32),
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PasswordAttempts(u8);

impl PasswordAttempts {
    pub const MAX: u8 = 5;

    pub fn new() -> Self {
        Self(0)
    }

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

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > Self::MAX {
            return Err(PasswordAttemptsError::ExceedMaxAttempts(value, Self::MAX));
        }

        Ok(PasswordAttempts(value))
    }
}

impl TryFrom<i32> for PasswordAttempts {
    type Error = PasswordAttemptsError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value < 0 {
            return Err(PasswordAttemptsError::NegativeValue(value));
        }

        if value > u8::MAX as i32 {
            return Err(PasswordAttemptsError::ExceedMaxAttempts(u8::MAX, Self::MAX));
        }

        PasswordAttempts::try_from(value as u8)
    }
}

impl From<PasswordAttempts> for u8 {
    fn from(value: PasswordAttempts) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    id: Option<UserId>,
    username: String,
    hashed_password: PasswordHashString,
    hashed_payment_password: PasswordHashString,
    wrong_payment_password_tried: PasswordAttempts,
    info: UserInfo,
}

impl User {
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

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn hashed_password(&self) -> &PasswordHashString {
        &self.hashed_password
    }

    pub fn hashed_payment_password(&self) -> &PasswordHashString {
        &self.hashed_payment_password
    }

    pub fn wrong_payment_password_tried(&self) -> PasswordAttempts {
        self.wrong_payment_password_tried
    }

    pub fn user_info(&self) -> &UserInfo {
        &self.info
    }
}

impl Identifiable<UserId> for User {
    fn get_id(&self) -> Option<UserId> {
        self.id
    }
}

impl Entity<UserId> for User {}

impl Aggregate<UserId> for User {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserInfo {
    pub name: String,
    pub gender: Option<Gender>,
    pub age: Option<Age>,
    pub phone: Phone,
    pub email: Option<EmailAddress>,
    pub identity_card_id: IdentityCardId,
}

impl UserInfo {
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
        use claims::{assert_err_eq, assert_ok_eq};

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
}
