use alloc::sync::Arc;

use cedar_policy_core::ast;
use error_stack::{Report, ResultExt as _, bail};
use type_system::{
    provenance::{ActorId, ActorType, AiId, MachineId, UserId},
    web::OwnedById,
};
use uuid::Uuid;

pub use self::actor::Actor;
use self::{
    role::{RoleId, SubteamRoleId, WebRoleId},
    team::{SubteamId, TeamId},
};
use super::cedar::CedarEntityId as _;

pub mod actor;
pub mod role;
pub mod team;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, derive_more::Display)]
pub enum PrincipalId {
    Actor(ActorId),
    Team(TeamId),
    Role(RoleId),
}

impl PrincipalId {
    #[must_use]
    pub const fn as_uuid(&self) -> &Uuid {
        match self {
            Self::Actor(actor_id) => actor_id.as_uuid(),
            Self::Team(team_id) => team_id.as_uuid(),
            Self::Role(role_id) => role_id.as_uuid(),
        }
    }

    #[must_use]
    pub const fn into_uuid(self) -> Uuid {
        match self {
            Self::Actor(actor_id) => actor_id.into_uuid(),
            Self::Team(team_id) => team_id.into_uuid(),
            Self::Role(role_id) => role_id.into_uuid(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum PrincipalConstraint {
    Actor {
        #[serde(flatten)]
        actor: ActorId,
    },
    ActorType {
        actor_type: ActorType,
    },
    Team {
        #[serde(flatten)]
        team: TeamId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actor_type: Option<ActorType>,
    },
    Role {
        #[serde(flatten)]
        role: RoleId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        actor_type: Option<ActorType>,
    },
}

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub(crate) enum InvalidPrincipalConstraint {
    #[display("Cannot convert constraints containing slots")]
    AmbiguousSlot,
    #[error(ignore)]
    #[display("Unexpected entity type: {_0}")]
    UnexpectedEntityType(ast::EntityType),
    #[display("Invalid principal ID")]
    InvalidPrincipalId,
}

fn actor_type_to_cedar(actor_type: ActorType) -> &'static Arc<ast::EntityType> {
    match actor_type {
        ActorType::User => UserId::entity_type(),
        ActorType::Machine => MachineId::entity_type(),
        ActorType::Ai => AiId::entity_type(),
    }
}

fn actor_type_from_cedar(
    actor_type: &ast::EntityType,
) -> Result<ActorType, Report<InvalidPrincipalConstraint>> {
    if *actor_type == **UserId::entity_type() {
        Ok(ActorType::User)
    } else if *actor_type == **MachineId::entity_type() {
        Ok(ActorType::Machine)
    } else if *actor_type == **AiId::entity_type() {
        Ok(ActorType::Ai)
    } else {
        Err(Report::new(
            InvalidPrincipalConstraint::UnexpectedEntityType(ast::EntityType::clone(actor_type)),
        ))
    }
}

impl PrincipalConstraint {
    pub(crate) fn try_from_cedar(
        constraint: &ast::PrincipalConstraint,
    ) -> Result<Option<Self>, Report<InvalidPrincipalConstraint>> {
        Ok(match constraint.as_inner() {
            ast::PrincipalOrResourceConstraint::Any => None,
            ast::PrincipalOrResourceConstraint::Is(principal_type) => Some(Self::ActorType {
                actor_type: actor_type_from_cedar(principal_type)?,
            }),
            ast::PrincipalOrResourceConstraint::Eq(ast::EntityReference::EUID(entity_ref)) => {
                Some(Self::try_from_cedar_eq(entity_ref)?)
            }
            ast::PrincipalOrResourceConstraint::IsIn(
                principal_type,
                ast::EntityReference::EUID(entity_ref),
            ) => Some(Self::try_from_cedar_is_in(
                Some(principal_type),
                entity_ref,
            )?),
            ast::PrincipalOrResourceConstraint::In(ast::EntityReference::EUID(principal)) => {
                Some(Self::try_from_cedar_is_in(None, principal)?)
            }
            ast::PrincipalOrResourceConstraint::Eq(ast::EntityReference::Slot(_))
            | ast::PrincipalOrResourceConstraint::IsIn(_, ast::EntityReference::Slot(_))
            | ast::PrincipalOrResourceConstraint::In(ast::EntityReference::Slot(_)) => {
                bail!(InvalidPrincipalConstraint::AmbiguousSlot);
            }
        })
    }

    fn try_from_cedar_eq(
        principal: &ast::EntityUID,
    ) -> Result<Self, Report<InvalidPrincipalConstraint>> {
        if *principal.entity_type() == **UserId::entity_type() {
            Ok(Self::Actor {
                actor: ActorId::User(
                    UserId::from_eid(principal.eid())
                        .change_context(InvalidPrincipalConstraint::InvalidPrincipalId)?,
                ),
            })
        } else if *principal.entity_type() == **MachineId::entity_type() {
            Ok(Self::Actor {
                actor: ActorId::Machine(
                    MachineId::from_eid(principal.eid())
                        .change_context(InvalidPrincipalConstraint::InvalidPrincipalId)?,
                ),
            })
        } else if *principal.entity_type() == **AiId::entity_type() {
            Ok(Self::Actor {
                actor: ActorId::Ai(
                    AiId::from_eid(principal.eid())
                        .change_context(InvalidPrincipalConstraint::InvalidPrincipalId)?,
                ),
            })
        } else {
            Err(Report::new(
                InvalidPrincipalConstraint::UnexpectedEntityType(ast::EntityType::clone(
                    principal.entity_type(),
                )),
            ))
        }
    }

    fn try_from_cedar_is_in(
        principal_type: Option<&ast::EntityType>,
        in_principal: &ast::EntityUID,
    ) -> Result<Self, Report<InvalidPrincipalConstraint>> {
        let actor_type = principal_type.map(actor_type_from_cedar).transpose()?;

        if *in_principal.entity_type() == **OwnedById::entity_type() {
            Ok(Self::Team {
                actor_type,
                team: TeamId::Web(
                    OwnedById::from_eid(in_principal.eid())
                        .change_context(InvalidPrincipalConstraint::InvalidPrincipalId)?,
                ),
            })
        } else if *in_principal.entity_type() == **WebRoleId::entity_type() {
            Ok(Self::Role {
                actor_type,
                role: RoleId::Web(
                    WebRoleId::from_eid(in_principal.eid())
                        .change_context(InvalidPrincipalConstraint::InvalidPrincipalId)?,
                ),
            })
        } else if *in_principal.entity_type() == **SubteamId::entity_type() {
            Ok(Self::Team {
                actor_type,
                team: TeamId::Subteam(
                    SubteamId::from_eid(in_principal.eid())
                        .change_context(InvalidPrincipalConstraint::InvalidPrincipalId)?,
                ),
            })
        } else if *in_principal.entity_type() == **SubteamRoleId::entity_type() {
            Ok(Self::Role {
                actor_type,
                role: RoleId::Subteam(
                    SubteamRoleId::from_eid(in_principal.eid())
                        .change_context(InvalidPrincipalConstraint::InvalidPrincipalId)?,
                ),
            })
        } else {
            Err(Report::new(
                InvalidPrincipalConstraint::UnexpectedEntityType(ast::EntityType::clone(
                    in_principal.entity_type(),
                )),
            ))
        }
    }

    #[must_use]
    pub(crate) fn to_cedar(&self) -> ast::PrincipalConstraint {
        match self {
            Self::ActorType { actor_type } => ast::PrincipalConstraint::is_entity_type(Arc::clone(
                actor_type_to_cedar(*actor_type),
            )),
            Self::Actor { actor } => ast::PrincipalConstraint::is_eq(Arc::new(match actor {
                ActorId::User(user) => user.to_euid(),
                ActorId::Machine(machine) => machine.to_euid(),
                ActorId::Ai(ai) => ai.to_euid(),
            })),
            Self::Team { team, actor_type } => {
                let euid = Arc::new(team.to_euid());
                if let Some(actor_type) = actor_type {
                    ast::PrincipalConstraint::is_entity_type_in(
                        Arc::clone(actor_type_to_cedar(*actor_type)),
                        euid,
                    )
                } else {
                    ast::PrincipalConstraint::is_in(euid)
                }
            }
            Self::Role { role, actor_type } => {
                let euid = Arc::new(role.to_euid());
                if let Some(actor_type) = actor_type {
                    ast::PrincipalConstraint::is_entity_type_in(
                        Arc::clone(actor_type_to_cedar(*actor_type)),
                        euid,
                    )
                } else {
                    ast::PrincipalConstraint::is_in(euid)
                }
            }
        }
    }
}

#[cfg(test)]
#[expect(clippy::panic_in_result_fn, reason = "Assertions in test are expected")]
mod tests {
    use core::error::Error;

    use indoc::formatdoc;
    use pretty_assertions::assert_eq;
    use serde_json::{Value as JsonValue, json};
    use uuid::Uuid;

    use super::PrincipalConstraint;
    use crate::{
        policies::{
            Effect, Policy, PolicyId, action::ActionConstraint, resource::ResourceConstraint,
            tests::check_policy,
        },
        test_utils::check_serialization,
    };

    #[track_caller]
    pub(crate) fn check_principal(
        constraint: PrincipalConstraint,
        value: JsonValue,
        cedar_string: impl AsRef<str>,
    ) -> Result<(), Box<dyn Error>> {
        let cedar_constraint = constraint.to_cedar();
        let cedar_string = cedar_string.as_ref();

        assert_eq!(cedar_constraint.to_string(), cedar_string);
        PrincipalConstraint::try_from_cedar(&cedar_constraint)?;

        let policy = Policy {
            id: PolicyId::new(Uuid::new_v4()),
            effect: Effect::Permit,
            principal: Some(constraint),
            action: ActionConstraint::All {},
            resource: ResourceConstraint::Global {},
            constraints: None,
        };

        check_policy(
            &policy,
            json!({
                "id": policy.id,
                "effect": "permit",
                "principal": &value,
                "action": {
                    "type": "all",
                },
                "resource": {
                    "type": "global",
                },
            }),
            formatdoc!(
                "permit(
                  {cedar_string},
                  action,
                  resource
                ) when {{
                  true
                }};"
            ),
        )?;

        check_serialization(&policy.principal, value);

        Ok(())
    }
}
