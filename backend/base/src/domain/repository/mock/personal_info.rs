#![cfg(test)]

use crate::domain::model::personal_info::{PersonalInfo, PersonalInfoId};
use crate::domain::model::user::{UserId, IdentityCardId};
use crate::domain::{Repository, RepositoryError};
use async_trait::async_trait;
use mockall::mock;
use crate::domain::repository::personal_info::PersonalInfoRepository;

mock! {
    pub PersonalInfoRepository {}

    #[async_trait]
    impl Repository<PersonalInfo> for PersonalInfoRepository {
        async fn find(&self, id: PersonalInfoId) -> Result<Option<PersonalInfo>, RepositoryError>;
        async fn remove(&self, aggregate: PersonalInfo) -> Result<(), RepositoryError>;
        async fn save(&self, aggregate: &mut PersonalInfo) -> Result<PersonalInfoId, RepositoryError>;
    }

    #[async_trait]
    impl PersonalInfoRepository for PersonalInfoRepository {
        async fn find_by_user_id(&self, user_id: UserId) -> Result<Vec<PersonalInfo>, RepositoryError>;

        async fn find_by_user_id_and_identity_card(
            &self,
            user_id: UserId,
            identity_card_id: IdentityCardId,
        ) -> Result<Option<PersonalInfo>, RepositoryError>;
    }
}

pub fn mock_personal_info_repository() -> MockPersonalInfoRepository {
    MockPersonalInfoRepository::new()
}