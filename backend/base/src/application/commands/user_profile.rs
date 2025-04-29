use crate::application::service::user_profile::SetUserProfileDTO;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserProfileQuery {
    pub session_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetUserProfileCommand {
    pub session_id: String,
    pub username: String,
    pub gender: Option<String>,
    pub age: Option<u16>,
    pub email: String,
}

impl SetUserProfileCommand {
    pub fn from_session_id_and_dto(session_id: String, dto: SetUserProfileDTO) -> Self {
        SetUserProfileCommand {
            session_id,
            username: dto.username,
            gender: dto.gender,
            age: dto.age,
            email: dto.email,
        }
    }
}
