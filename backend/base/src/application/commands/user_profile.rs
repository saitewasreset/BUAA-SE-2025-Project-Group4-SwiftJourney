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
