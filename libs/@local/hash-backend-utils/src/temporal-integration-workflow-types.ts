import type {
  ActorEntityUuid,
  EntityId,
  OwnedById,
  VersionedUrl,
} from "@blockprotocol/type-system";
import type { Team } from "@linear/sdk";
import type { Entity, SerializedEntity } from "@local/hash-graph-sdk/entity";

export type PartialEntity = {
  properties: Partial<Entity["properties"]>;
  entityTypeId: VersionedUrl;
};

export const supportedLinearTypes = ["Issue", "User"] as const;

export type SupportedLinearType = (typeof supportedLinearTypes)[number];

export type CreateHashEntityFromLinearData = <
  T extends SupportedLinearType = SupportedLinearType,
>(params: {
  authentication: { actorId: ActorEntityUuid };
  linearId: string;
  linearType: T;
  linearApiKey: string;
  ownedById: OwnedById;
}) => Promise<void>;

export type UpdateHashEntityFromLinearData = <
  T extends SupportedLinearType = SupportedLinearType,
>(params: {
  authentication: { actorId: ActorEntityUuid };
  linearId: string;
  linearType: T;
  linearApiKey: string;
  ownedById: OwnedById;
}) => Promise<void>;

export type ReadLinearTeamsWorkflow = (params: {
  apiKey: string;
}) => Promise<Team[]>;

export type SyncWorkspaceWorkflow = (params: {
  authentication: { actorId: ActorEntityUuid };
  apiKey: string;
  workspaceOwnedById: OwnedById;
  teamIds: string[];
}) => Promise<void>;

export type UpdateLinearDataWorkflow = (params: {
  apiKey: string;
  authentication: { actorId: ActorEntityUuid };
  linearId: string;
  entityTypeIds: [VersionedUrl, ...VersionedUrl[]];
  entity: SerializedEntity;
}) => Promise<void>;

export type SyncQueryToGoogleSheetWorkflow = (params: {
  integrationEntityId: EntityId;
  userAccountId: ActorEntityUuid;
}) => Promise<void>;

export type WorkflowTypeMap = {
  syncWorkspace: SyncWorkspaceWorkflow;
  readLinearTeams: ReadLinearTeamsWorkflow;

  createHashEntityFromLinearData: CreateHashEntityFromLinearData;
  updateHashEntityFromLinearData: UpdateHashEntityFromLinearData;

  updateLinearData: UpdateLinearDataWorkflow;
  /** @todo: add `createLinearData` */
};
