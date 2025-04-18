use crate::application::service::user_manager::{UserLoginDTO, UserRegisterDTO};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserRegisterCommand {
    pub phone: String,
    pub username: String,
    pub password: String,
    pub name: String,
    pub identity_card_id: String,
}

impl From<UserRegisterDTO> for UserRegisterCommand {
    fn from(dto: UserRegisterDTO) -> Self {
        UserRegisterCommand {
            phone: dto.phone,
            username: dto.username,
            password: dto.password,
            name: dto.name,
            identity_card_id: dto.identity_card_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserLoginCommand {
    pub phone: String,
    pub password: String,
}

impl From<UserLoginDTO> for UserLoginCommand {
    fn from(value: UserLoginDTO) -> Self {
        UserLoginCommand {
            phone: value.phone,
            password: value.password,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserLogoutCommand {
    pub session_id: String,
}
