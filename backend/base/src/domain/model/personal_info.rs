use id_macro::define_id_type;

use crate::domain::Identifier;

define_id_type!(PersonalInfo);
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PersonalInfo {}
