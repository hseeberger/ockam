use ockam_core::compat::sync::Arc;
use ockam_core::Result;
use ockam_vault::KeyId;

use super::super::models::Identifier;
use super::super::{IdentitiesKeys, IdentitiesRepository, IdentitiesVault, Identity};

/// This struct supports functions for the creation and import of identities using an IdentityVault
pub struct IdentitiesCreation {
    repository: Arc<dyn IdentitiesRepository>,
    vault: Arc<dyn IdentitiesVault>,
}

impl IdentitiesCreation {
    /// Create a new identities import module
    pub fn new(
        repository: Arc<dyn IdentitiesRepository>,
        vault: Arc<dyn IdentitiesVault>,
    ) -> IdentitiesCreation {
        IdentitiesCreation { repository, vault }
    }

    /// Import and verify identity from its binary format
    pub async fn import(
        &self,
        expected_identifier: Option<&Identifier>,
        data: &[u8],
    ) -> Result<Identity> {
        Identity::import(expected_identifier, data, self.vault.clone()).await
    }

    /// Create an Identity
    pub async fn create_identity(&self) -> Result<Identity> {
        // TODO: Consider creating PurposeKeys by default
        self.make_and_persist_identity(None).await
    }
}

impl IdentitiesCreation {
    /// Make a new identity with its key and attributes
    /// and persist it
    async fn make_and_persist_identity(&self, key_id: Option<&KeyId>) -> Result<Identity> {
        let identity_keys = IdentitiesKeys::new(self.vault.clone());
        let identity = identity_keys.create_initial_key(key_id).await?;
        self.repository
            .update_identity(identity.identifier(), identity.change_history())
            .await?;
        Ok(identity)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::models::Identifier;
    use super::super::identities;
    use super::*;
    use core::str::FromStr;

    #[tokio::test]
    async fn test_identity_creation() -> Result<()> {
        let identities = identities();
        let creation = identities.identities_creation();
        let repository = identities.repository();
        let keys = identities.identities_keys();

        let identity = creation.create_identity().await?;
        let actual = repository.get_identity(identity.identifier()).await?;

        let actual = Identity::import_from_change_history(
            Some(identity.identifier()),
            actual,
            identities.vault(),
        )
        .await?;
        assert_eq!(
            actual, identity,
            "the identity can be retrieved from the repository"
        );

        let actual = repository.retrieve_identity(identity.identifier()).await?;
        assert!(actual.is_some());
        let actual = Identity::import_from_change_history(
            Some(identity.identifier()),
            actual.unwrap(),
            identities.vault(),
        )
        .await?;
        assert_eq!(
            actual, identity,
            "the identity can be retrieved from the repository as an Option"
        );

        let another_identifier = Identifier::from_str("Ie92f183eb4c324804ef4d62962dea94cf095a265")?;
        let missing = repository.retrieve_identity(&another_identifier).await?;
        assert_eq!(missing, None, "a missing identity returns None");

        let root_key = keys.get_secret_key(&identity).await;
        assert!(root_key.is_ok(), "there is a key for the created identity");

        Ok(())
    }
}
