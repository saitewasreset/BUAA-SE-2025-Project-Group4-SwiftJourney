use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionDTO {
    pub user_id: u64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonalInfoDTO {
    pub id: Option<u64>,
    pub uuid: Uuid,
    pub name: String,
    pub identity_card_id: String,
    pub preferred_seat_location: Option<char>,
    pub user_id: u64,
    pub is_default: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserInfoDTO {
    /// 用户真实姓名
    pub name: String,
    /// 用户性别(可选)
    pub gender: Option<String>,
    /// 用户年龄(可选)
    pub age: Option<u16>,
    /// 用户手机号码
    pub phone: String,
    /// 用户电子邮箱(可选)
    pub email: Option<String>,
    /// 用户身份证号
    pub identity_card_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserCombinedInfoDTO {
    pub user_id: u64,
    pub username: String,
    pub user_info: UserInfoDTO,
    pub personal_info_list: Vec<PersonalInfoDTO>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DbUserDTO {
    pub id: i32,
    pub username: String,
    pub hashed_password: Vec<u8>,
    pub hashed_payment_password: Option<Vec<u8>>,
    pub salt: Vec<u8>,
    pub wrong_payment_password_tried: i32,
    pub gender: Option<String>,
    pub age: Option<i32>,
    pub phone: String,
    pub email: Option<String>,
    pub name: String,
    pub identity_card_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DbPersonalInfo {
    pub id: i32,
    pub uuid: Uuid,
    pub name: String,
    pub identity_card: String,
    pub preferred_seat_location: Option<String>,
    pub user_id: i32,
    pub is_default: bool,
}
