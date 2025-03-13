#![allow(unreachable_pub, reason = "his file is used as module in other tests")]

use hash_graph_types::account::AccountId;
use type_system::knowledge::entity::id::EntityUuid;
use uuid::Uuid;

pub const ALICE: AccountId = AccountId::new(Uuid::from_fields(1, 0, 0, &[0; 8]));
pub const BOB: AccountId = AccountId::new(Uuid::from_fields(2, 0, 0, &[0; 8]));

pub const ENTITY_A: EntityUuid = EntityUuid::new(Uuid::from_fields(0, 1, 0, &[0; 8]));
pub const ENTITY_B: EntityUuid = EntityUuid::new(Uuid::from_fields(0, 2, 0, &[0; 8]));
