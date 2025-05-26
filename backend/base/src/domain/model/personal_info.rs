use id_macro::define_id_type;
use uuid::Uuid;

use crate::domain::{Aggregate, Entity, Identifiable, Identifier};

use super::user::{IdentityCardId, RealName, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PreferredSeatLocation {
    A,
    B,
    C,
    D,
    F,
}

impl TryFrom<char> for PreferredSeatLocation {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'A' => Ok(PreferredSeatLocation::A),
            'B' => Ok(PreferredSeatLocation::B),
            'C' => Ok(PreferredSeatLocation::C),
            'D' => Ok(PreferredSeatLocation::D),
            'F' => Ok(PreferredSeatLocation::F),
            _ => Err("Invalid preferred seat location"),
        }
    }
}

impl From<PreferredSeatLocation> for char {
    fn from(value: PreferredSeatLocation) -> Self {
        match value {
            PreferredSeatLocation::A => 'A',
            PreferredSeatLocation::B => 'B',
            PreferredSeatLocation::C => 'C',
            PreferredSeatLocation::D => 'D',
            PreferredSeatLocation::F => 'F',
        }
    }
}

impl std::fmt::Display for PreferredSeatLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

define_id_type!(PersonalInfo);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonalInfo {
    id: Option<PersonalInfoId>,
    uuid: Uuid,
    name: RealName,
    identity_card_id: IdentityCardId,
    preferred_seat_location: Option<PreferredSeatLocation>,
    user_id: UserId,
    is_default: bool,
}

impl PersonalInfo {
    pub fn new(
        id: Option<PersonalInfoId>,
        uuid: Uuid,
        name: RealName,
        identity_card_id: IdentityCardId,
        preferred_seat_location: Option<PreferredSeatLocation>,
        user_id: UserId,
    ) -> Self {
        Self {
            id,
            uuid,
            name,
            identity_card_id,
            preferred_seat_location,
            user_id,
            is_default: false,
        }
    }

    pub fn id_mut(&mut self) -> &mut Option<PersonalInfoId> {
        &mut self.id
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn name(&self) -> &RealName {
        &self.name
    }

    pub fn identity_card_id(&self) -> &IdentityCardId {
        &self.identity_card_id
    }

    pub fn preferred_seat_location(&self) -> Option<PreferredSeatLocation> {
        self.preferred_seat_location
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn set_preferred_seat_location(&mut self, location: Option<PreferredSeatLocation>) {
        self.preferred_seat_location = location;
    }

    pub fn set_identity_card_id(&mut self, id: IdentityCardId) {
        self.identity_card_id = id;
    }

    pub fn set_name(&mut self, name: RealName) {
        self.name = name;
    }

    pub fn set_uuid(&mut self, uuid: Uuid) {
        self.uuid = uuid;
    }

    pub fn set_user_id(&mut self, user_id: UserId) {
        self.user_id = user_id;
    }

    pub fn is_default(&self) -> bool {
        self.is_default
    }

    pub fn set_default(&mut self, is_default: bool) {
        self.is_default = is_default;
    }
}

impl Identifiable for PersonalInfo {
    type ID = PersonalInfoId;

    fn get_id(&self) -> Option<Self::ID> {
        self.id
    }

    fn set_id(&mut self, id: Self::ID) {
        self.id = Some(id);
    }
}

impl Entity for PersonalInfo {}
impl Aggregate for PersonalInfo {}
