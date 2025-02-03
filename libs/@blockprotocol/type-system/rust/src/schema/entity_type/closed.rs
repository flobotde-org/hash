use alloc::sync::Arc;
use core::cmp;
use std::collections::{HashMap, HashSet, hash_map::Entry};

use error_stack::{Report, bail, ensure};
use itertools::Itertools as _;
use thiserror::Error;

use crate::{
    schema::{
        EntityType, EntityTypeReference, EntityTypeToPropertyTypeEdge, EntityTypeUuid,
        InheritanceDepth, InverseEntityTypeMetadata, PropertyTypeUuid,
        entity_type::{EntityConstraints, EntityTypeDisplayMetadata, extend_links},
    },
    url::{BaseUrl, VersionedUrl},
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClosedEntityTypeMetadata {
    #[serde(rename = "$id")]
    pub id: VersionedUrl,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_plural: Option<String>,
    pub description: String,
    #[cfg_attr(
        target_arch = "wasm32",
        tsify(type = "[EntityTypeDisplayMetadata, ...EntityTypeDisplayMetadata[]]")
    )]
    pub all_of: Vec<EntityTypeDisplayMetadata>,
    #[serde(default, skip_serializing_if = "InverseEntityTypeMetadata::is_empty")]
    pub inverse: InverseEntityTypeMetadata,
}

impl ClosedEntityTypeMetadata {
    #[must_use]
    pub fn is_link(&self) -> bool {
        self.all_of.iter().any(|entity_type| {
            entity_type.id.base_url.as_str()
                == "https://blockprotocol.org/@blockprotocol/types/entity-type/link/"
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClosedEntityType {
    #[serde(rename = "$id")]
    pub id: VersionedUrl,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_plural: Option<String>,
    pub description: String,
    #[cfg_attr(
        target_arch = "wasm32",
        tsify(type = "[EntityTypeDisplayMetadata, ...EntityTypeDisplayMetadata[]]")
    )]
    pub all_of: Vec<EntityTypeDisplayMetadata>,
    #[serde(default, skip_serializing_if = "InverseEntityTypeMetadata::is_empty")]
    pub inverse: InverseEntityTypeMetadata,
    #[serde(flatten)]
    pub constraints: EntityConstraints,
}

#[derive(Debug, Error)]
pub enum ResolveClosedEntityTypeError {
    #[error(
        "Resolving the entity type encountered unknown schemas in `allOf`: {}.",
        serde_json::json!(.0.iter().map(|reference| &reference.url).collect::<Vec<_>>()),
    )]
    UnknownSchemas(HashSet<EntityTypeReference>),
    #[error("Resolving the entity type encountered incompatible property: {0}.")]
    IncompatibleProperty(BaseUrl),
    #[error("The entity type has an empty schema")]
    EmptySchema,
    #[error("The entity type has an inheritance depth overflow")]
    InheritanceDepthOverflow,
}

impl ClosedEntityType {
    /// Create a closed entity type from an entity type and its resolve data.
    ///
    /// # Errors
    ///
    /// Returns an error if the entity type references unknown schema in `allOf`.
    pub fn from_resolve_data(
        mut schema: EntityType,
        resolve_data: &EntityTypeResolveData,
    ) -> Result<Self, Report<ResolveClosedEntityTypeError>> {
        let mut closed_schema = Self {
            id: schema.id.clone(),
            constraints: schema.constraints,
            title: schema.title,
            title_plural: schema.title_plural,
            description: schema.description,
            inverse: schema.inverse,
            all_of: vec![EntityTypeDisplayMetadata {
                id: schema.id,
                depth: 0,
                icon: schema.icon,
                label_property: schema.label_property,
            }],
        };

        for (depth, entity_type) in resolve_data.ordered_schemas() {
            schema.all_of.remove((&entity_type.id).into());
            closed_schema
                .constraints
                .properties
                .extend(entity_type.constraints.properties.clone());
            closed_schema
                .constraints
                .required
                .extend(entity_type.constraints.required.clone());
            extend_links(
                &mut closed_schema.constraints.links,
                entity_type.constraints.links.clone(),
            );

            closed_schema.all_of.push(EntityTypeDisplayMetadata {
                id: entity_type.id.clone(),
                depth: depth
                    .inner()
                    .checked_add(1)
                    .ok_or(ResolveClosedEntityTypeError::InheritanceDepthOverflow)?,
                label_property: entity_type.label_property.clone(),
                icon: entity_type.icon.clone(),
            });
        }

        ensure!(
            schema.all_of.is_empty(),
            ResolveClosedEntityTypeError::UnknownSchemas(schema.all_of)
        );

        Ok(closed_schema)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(target_arch = "wasm32", derive(tsify::Tsify))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClosedMultiEntityType {
    #[serde(flatten)]
    pub constraints: EntityConstraints,
    #[cfg_attr(
        target_arch = "wasm32",
        tsify(type = "[ClosedEntityTypeMetadata, ...ClosedEntityTypeMetadata[]]")
    )]
    pub all_of: Vec<ClosedEntityTypeMetadata>,
}

impl ClosedMultiEntityType {
    #[must_use]
    pub fn from_closed_schema(closed_schemas: ClosedEntityType) -> Self {
        Self {
            all_of: vec![ClosedEntityTypeMetadata {
                id: closed_schemas.id.clone(),
                title: closed_schemas.title.clone(),
                title_plural: closed_schemas.title_plural.clone(),
                description: closed_schemas.description.clone(),
                inverse: closed_schemas.inverse.clone(),
                all_of: closed_schemas.all_of,
            }],
            constraints: closed_schemas.constraints,
        }
    }

    pub fn is_link(&self) -> bool {
        self.all_of.iter().any(ClosedEntityTypeMetadata::is_link)
    }

    /// Creates a closed entity type from multiple closed entity types.
    ///
    /// This results in a closed entity type which is used for entities with multiple types.
    ///
    /// # Errors
    ///
    /// Returns an error if the entity types have incompatible properties.
    pub fn from_multi_type_closed_schema(
        closed_schemas: impl IntoIterator<Item = ClosedEntityType>,
    ) -> Result<Self, Report<ResolveClosedEntityTypeError>> {
        let mut closed_schemas = closed_schemas.into_iter();
        let Some(mut closed_schema) = closed_schemas.next().map(Self::from_closed_schema) else {
            bail!(ResolveClosedEntityTypeError::EmptySchema);
        };

        for schema in closed_schemas {
            closed_schema.add_closed_entity_type(schema)?;
        }

        Ok(closed_schema)
    }

    /// Adds a new closed entity type to the multi entity type.
    ///
    /// # Errors
    ///
    /// Returns an error if the entity types have incompatible properties.
    pub fn add_closed_entity_type(
        &mut self,
        schema: ClosedEntityType,
    ) -> Result<(), Report<ResolveClosedEntityTypeError>> {
        for (base_url, property) in schema.constraints.properties {
            match self.constraints.properties.entry(base_url) {
                Entry::Occupied(entry) => {
                    ensure!(
                        property == *entry.get(),
                        ResolveClosedEntityTypeError::IncompatibleProperty(entry.key().clone())
                    );
                }
                Entry::Vacant(entry) => {
                    entry.insert(property);
                }
            }
        }
        self.constraints
            .required
            .extend(schema.constraints.required);
        extend_links(&mut self.constraints.links, schema.constraints.links);
        self.all_of.push(ClosedEntityTypeMetadata {
            id: schema.id,
            title: schema.title,
            title_plural: schema.title_plural,
            description: schema.description,
            inverse: schema.inverse,
            all_of: schema.all_of,
        });

        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct EntityTypeResolveData {
    inheritance_depths: HashMap<EntityTypeUuid, (InheritanceDepth, Arc<EntityType>)>,
    // We currently don't need to store the actual target entity types and property types, except
    // for inheritance, so we only store the depth.
    links: HashMap<EntityTypeUuid, InheritanceDepth>,
    link_destinations: HashMap<EntityTypeUuid, InheritanceDepth>,
    properties: HashMap<PropertyTypeUuid, InheritanceDepth>,
}

impl EntityTypeResolveData {
    pub fn add_entity_type_inheritance_edge(
        &mut self,
        target: Arc<EntityType>,
        target_id: EntityTypeUuid,
        depth: u16,
    ) {
        let depth = InheritanceDepth::new(depth);
        match self.inheritance_depths.entry(target_id) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().0 = cmp::min(depth, entry.get().0);
            }
            Entry::Vacant(entry) => {
                entry.insert((depth, target));
            }
        }
    }

    pub fn add_entity_type_link_edge(&mut self, target_id: EntityTypeUuid, depth: u16) {
        let depth = InheritanceDepth::new(depth);
        match self.links.entry(target_id) {
            Entry::Occupied(mut entry) => {
                *entry.get_mut() = cmp::min(depth, *entry.get());
            }
            Entry::Vacant(entry) => {
                entry.insert(depth);
            }
        }
    }

    pub fn add_entity_type_link_destination_edge(&mut self, target_id: EntityTypeUuid, depth: u16) {
        let depth = InheritanceDepth::new(depth);
        match self.link_destinations.entry(target_id) {
            Entry::Occupied(mut entry) => {
                *entry.get_mut() = cmp::min(depth, *entry.get());
            }
            Entry::Vacant(entry) => {
                entry.insert(depth);
            }
        }
    }

    pub fn add_property_type_edge(
        &mut self,
        edge: EntityTypeToPropertyTypeEdge,
        target_id: PropertyTypeUuid,
        depth: u16,
    ) {
        let depth = InheritanceDepth::new(depth);
        match edge {
            EntityTypeToPropertyTypeEdge::Property => match self.properties.entry(target_id) {
                Entry::Occupied(mut entry) => {
                    *entry.get_mut() = cmp::min(depth, *entry.get());
                }
                Entry::Vacant(entry) => {
                    entry.insert(depth);
                }
            },
        }
    }

    pub fn extend_edges(&mut self, depth_offset: u16, other: &Self) {
        for (target_id, (relative_depth, schema)) in &other.inheritance_depths {
            let absolut_depth = InheritanceDepth::new(relative_depth.inner() + depth_offset);
            match self.inheritance_depths.entry(*target_id) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().0 = cmp::min(absolut_depth, entry.get().0);
                }
                Entry::Vacant(entry) => {
                    entry.insert((absolut_depth, Arc::clone(schema)));
                }
            }
        }
        for (target_id, relative_depth) in &other.links {
            let absolut_depth = InheritanceDepth::new(relative_depth.inner() + depth_offset);
            match self.links.entry(*target_id) {
                Entry::Occupied(mut entry) => {
                    *entry.get_mut() = cmp::min(absolut_depth, *entry.get());
                }
                Entry::Vacant(entry) => {
                    entry.insert(absolut_depth);
                }
            }
        }
        for (target_id, relative_depth) in &other.link_destinations {
            let absolut_depth = InheritanceDepth::new(relative_depth.inner() + depth_offset);
            match self.link_destinations.entry(*target_id) {
                Entry::Occupied(mut entry) => {
                    *entry.get_mut() = cmp::min(absolut_depth, *entry.get());
                }
                Entry::Vacant(entry) => {
                    entry.insert(absolut_depth);
                }
            }
        }
        for (target_id, relative_depth) in &other.properties {
            let absolut_depth = InheritanceDepth::new(relative_depth.inner() + depth_offset);
            match self.properties.entry(*target_id) {
                Entry::Occupied(mut entry) => {
                    *entry.get_mut() = cmp::min(absolut_depth, *entry.get());
                }
                Entry::Vacant(entry) => {
                    entry.insert(absolut_depth);
                }
            }
        }
    }

    pub fn inheritance_depths(&self) -> impl Iterator<Item = (EntityTypeUuid, InheritanceDepth)> {
        self.inheritance_depths
            .iter()
            .map(|(id, (depth, _))| (*id, *depth))
    }

    pub fn links(&self) -> impl Iterator<Item = (EntityTypeUuid, InheritanceDepth)> {
        self.links.iter().map(|(id, depth)| (*id, *depth))
    }

    pub fn link_destinations(&self) -> impl Iterator<Item = (EntityTypeUuid, InheritanceDepth)> {
        self.link_destinations
            .iter()
            .map(|(id, depth)| (*id, *depth))
    }

    pub fn properties(&self) -> impl Iterator<Item = (PropertyTypeUuid, InheritanceDepth)> {
        self.properties.iter().map(|(id, depth)| (*id, *depth))
    }

    /// Returns an iterator over the schemas ordered by inheritance depth and entity type id.
    fn ordered_schemas(&self) -> impl Iterator<Item = (InheritanceDepth, &EntityType)> {
        // TODO: Construct the sorted list on the fly when constructing this struct
        self.inheritance_depths
            .iter()
            .sorted_by_key(|(data_type_id, (depth, _))| (*depth, data_type_id.into_uuid()))
            .map(|(_, (depth, schema))| (*depth, &**schema))
    }
}

#[cfg(test)]
mod tests {
    use alloc::sync::Arc;

    use crate::{
        schema::{EntityType, EntityTypeUuid, OntologyTypeResolver, entity_type::ClosedEntityType},
        url::BaseUrl,
        utils::tests::{JsonEqualityCheck, ensure_serialization_from_str},
    };

    #[test]
    fn merge_entity_type() {
        let mut ontology_type_resolver = OntologyTypeResolver::default();

        let building = ensure_serialization_from_str::<EntityType>(
            hash_graph_test_data::entity_type::BUILDING_V1,
            JsonEqualityCheck::Yes,
        );
        ontology_type_resolver
            .add_unresolved_entity_type(EntityTypeUuid::from_url(&building.id), Arc::new(building));

        let church: EntityType = ensure_serialization_from_str::<EntityType>(
            hash_graph_test_data::entity_type::CHURCH_V1,
            JsonEqualityCheck::Yes,
        );
        let church_id = EntityTypeUuid::from_url(&church.id);
        ontology_type_resolver.add_unresolved_entity_type(church_id, Arc::new(church.clone()));

        let resolved_church = ontology_type_resolver
            .resolve_entity_type_metadata(church_id)
            .expect("church not resolved");
        let closed_church = ClosedEntityType::from_resolve_data(church, &resolved_church)
            .expect("Could not close church");

        assert!(
            closed_church.constraints.properties.contains_key(
                &BaseUrl::new(
                    "https://blockprotocol.org/@alice/types/property-type/built-at/".to_owned()
                )
                .expect("invalid url")
            )
        );
        assert!(
            closed_church.constraints.properties.contains_key(
                &BaseUrl::new(
                    "https://blockprotocol.org/@alice/types/property-type/number-bells/".to_owned()
                )
                .expect("invalid url")
            )
        );
    }
}
