//! Typed authored resources. Definition identity, actual ownership and lifetime
//! are separate axes; runtime adapters must validate an owner before binding it.

use std::{collections::BTreeSet, fmt};

use serde::{Deserialize, Serialize};

pub(crate) mod durable;
pub(crate) mod runtime;
pub(crate) mod store;
pub use durable::{RunResourceEntry, RunResourceState};
pub use store::{ResourceKey, ResourceMutation, ResourceOwner, ResourceValue, ResourceView};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceOperation {
    pub resource: ResourceId,
    pub owner: ResourceOwnerSelector,
    pub operation: ResourceMutation,
}

crate::authoring::clone_shape::struct_shape!(ResourceOperation {
    resource,
    owner,
    operation
});

pub const MAX_RESOURCE_ID_BYTES: usize = 128;
pub const MAX_RESOURCE_DEFINITIONS: usize = 4096;

/// A content key, never an employee, equipment copy or runtime instance ID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ResourceId(String);

impl ResourceId {
    pub fn new(value: String) -> Result<Self, ResourceDefinitionError> {
        if value.is_empty()
            || value.len() > MAX_RESOURCE_ID_BYTES
            || value.trim() != value
            || value.chars().any(char::is_control)
        {
            return Err(ResourceDefinitionError::InvalidId);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ResourceId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ResourceOwnerKind {
    Employee,
    EquipmentCopy,
    FragmentCopy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ResourceLifetime {
    Run,
    Attempt,
    Deployment,
    Activation,
    Delivery,
    Chain,
}

/// Explicit role selection. A provider is not implicitly the target/beneficiary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ResourceOwnerSelector {
    SourceEmployee,
    SourceEquipment,
    SourceFragment,
    TargetEmployee,
}

impl ResourceOwnerSelector {
    pub fn owner_kind(self) -> ResourceOwnerKind {
        match self {
            Self::SourceEmployee | Self::TargetEmployee => ResourceOwnerKind::Employee,
            Self::SourceEquipment => ResourceOwnerKind::EquipmentCopy,
            Self::SourceFragment => ResourceOwnerKind::FragmentCopy,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceDefinition {
    pub id: ResourceId,
    pub owner: ResourceOwnerKind,
    pub lifetime: ResourceLifetime,
    pub initial: u32,
    pub cap: u32,
}

impl ResourceDefinition {
    pub fn validate(&self) -> Result<(), ResourceDefinitionError> {
        if self.initial > self.cap {
            return Err(ResourceDefinitionError::InitialAboveCap(self.id.clone()));
        }
        Ok(())
    }

    pub fn validate_selector(
        &self,
        selector: ResourceOwnerSelector,
    ) -> Result<(), ResourceDefinitionError> {
        if selector.owner_kind() != self.owner {
            return Err(ResourceDefinitionError::OwnerKindMismatch(self.id.clone()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceCatalog {
    pub resources: Vec<ResourceDefinition>,
}

impl ResourceCatalog {
    pub fn validate(&self) -> Result<(), ResourceDefinitionError> {
        if self.resources.len() > MAX_RESOURCE_DEFINITIONS {
            return Err(ResourceDefinitionError::TooManyDefinitions);
        }
        crate::authoring::control::charge(
            self.resources.len() as u64,
            "resource_catalog_validation",
        )
        .map_err(|error| ResourceDefinitionError::Control(error.code()))?;
        let mut ids = BTreeSet::new();
        for definition in &self.resources {
            definition.validate()?;
            if !ids.insert(&definition.id) {
                return Err(ResourceDefinitionError::DuplicateId(definition.id.clone()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceDefinitionError {
    InvalidId,
    InitialAboveCap(ResourceId),
    DuplicateId(ResourceId),
    OwnerKindMismatch(ResourceId),
    TooManyDefinitions,
    Control(&'static str),
}

impl fmt::Display for ResourceDefinitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidId => write!(f, "resource id must be nonempty, trimmed, control-free and at most {MAX_RESOURCE_ID_BYTES} bytes"),
            Self::InitialAboveCap(id) => write!(f, "resource {}: initial exceeds cap", id.as_str()),
            Self::DuplicateId(id) => write!(f, "duplicate resource {}", id.as_str()),
            Self::OwnerKindMismatch(id) => write!(f, "resource {}: selector owner kind differs from declaration", id.as_str()),
            Self::TooManyDefinitions => write!(f, "resource catalog exceeds {MAX_RESOURCE_DEFINITIONS} definitions"),
            Self::Control(code) => write!(f, "resource catalog validation rejected: {code}"),
        }
    }
}

impl std::error::Error for ResourceDefinitionError {}

crate::authoring::clone_shape::newtype_shape!(ResourceId);
crate::authoring::clone_shape::copy_shape!(
    ResourceOwnerKind,
    ResourceLifetime,
    ResourceOwnerSelector
);
crate::authoring::clone_shape::struct_shape!(ResourceDefinition {
    id,
    owner,
    lifetime,
    initial,
    cap,
});
crate::authoring::clone_shape::struct_shape!(ResourceCatalog { resources });

#[cfg(test)]
mod tests {
    use super::*;

    fn declaration() -> &'static str {
        r#"(resources:[(id:"charge",owner:EquipmentCopy,lifetime:Run,initial:3,cap:5)])"#
    }

    #[test]
    fn strict_ron_requires_every_field_and_rejects_unknown_fields() {
        let catalog: ResourceCatalog = ron::de::from_str(declaration()).unwrap();
        catalog.validate().unwrap();
        for field in [
            "id:\"charge\",",
            "owner:EquipmentCopy,",
            "lifetime:Run,",
            "initial:3,",
            ",cap:5",
        ] {
            let input = declaration().replace(field, "");
            assert!(
                ron::de::from_str::<ResourceCatalog>(&input).is_err(),
                "{input}"
            );
        }
        for input in [
            "()".to_owned(),
            declaration().replace("cap:5", "cap:5,unknown:1"),
            declaration().replace("resources:", "unknown:0,resources:"),
        ] {
            assert!(
                ron::de::from_str::<ResourceCatalog>(&input).is_err(),
                "{input}"
            );
        }
    }

    #[test]
    fn owner_and_lifetime_are_orthogonal_and_selector_is_explicit() {
        for lifetime in [
            "Run",
            "Attempt",
            "Deployment",
            "Activation",
            "Delivery",
            "Chain",
        ] {
            for owner in ["Employee", "EquipmentCopy"] {
                let input = declaration()
                    .replace("Run", lifetime)
                    .replace("EquipmentCopy", owner);
                let catalog: ResourceCatalog = ron::de::from_str(&input).unwrap();
                catalog.validate().unwrap();
                let def = &catalog.resources[0];
                for selector in [
                    ResourceOwnerSelector::SourceEmployee,
                    ResourceOwnerSelector::SourceEquipment,
                    ResourceOwnerSelector::TargetEmployee,
                ] {
                    assert_eq!(
                        def.validate_selector(selector).is_ok(),
                        def.owner == selector.owner_kind()
                    );
                }
                let roundtrip: ResourceCatalog =
                    ron::de::from_str(&ron::ser::to_string(&catalog).unwrap()).unwrap();
                assert_eq!(catalog, roundtrip);
            }
        }
    }

    #[test]
    fn duplicate_ids_and_invalid_bounds_fail_without_normalization() {
        let mut catalog: ResourceCatalog = ron::de::from_str(declaration()).unwrap();
        catalog.resources.push(catalog.resources[0].clone());
        assert!(matches!(
            catalog.validate(),
            Err(ResourceDefinitionError::DuplicateId(_))
        ));
        catalog.resources.pop();
        catalog.resources[0].initial = 6;
        assert!(matches!(
            catalog.validate(),
            Err(ResourceDefinitionError::InitialAboveCap(_))
        ));
        catalog.resources[0].initial = 0;
        catalog.resources[0].cap = 0;
        catalog.validate().unwrap();
        catalog.resources[0].initial = u32::MAX;
        catalog.resources[0].cap = u32::MAX;
        catalog.validate().unwrap();
        for value in ["", " charge", "charge ", "a\nb"] {
            assert!(ResourceId::new(value.to_owned()).is_err());
        }
        assert!(ResourceId::new("x".repeat(MAX_RESOURCE_ID_BYTES + 1)).is_err());
        assert!(ResourceId::new("x".repeat(MAX_RESOURCE_ID_BYTES)).is_ok());
        for input in [
            declaration().replace("initial:3", "initial:-1"),
            declaration().replace("cap:5", "cap:4294967296"),
            declaration().replace("\"charge\"", "\" charge\""),
        ] {
            assert!(
                ron::de::from_str::<ResourceCatalog>(&input).is_err(),
                "{input}"
            );
        }
    }
}
