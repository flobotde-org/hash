export const scalars = {
  BaseUrl: "@blockprotocol/type-system#BaseUrl",
  VersionedUrl: "@blockprotocol/type-system#VersionedUrl",

  Date: "string",

  JSONObject: "@blockprotocol/core#JsonObject",
  QueryOperationInput: "@blockprotocol/graph#QueryOperationInput",

  TextToken: "@local/hash-isomorphic-utils/types#TextToken",

  HasIndexedContentProperties:
    "@local/hash-isomorphic-utils/system-types/shared#HasIndexedContentProperties",
  HasSpatiallyPositionedContentProperties:
    "@local/hash-isomorphic-utils/system-types/canvas#HasSpatiallyPositionedContentProperties",

  DataTypeWithMetadata: "@blockprotocol/type-system#DataTypeWithMetadata",
  ConstructDataTypeParams:
    "@local/hash-graph-types/ontology#ConstructDataTypeParams",
  DataTypeFullConversionTargetsMap:
    "@local/hash-graph-types/ontology#DataTypeFullConversionTargetsMap",
  DataTypeDirectConversionsMap:
    "@local/hash-graph-types/ontology#DataTypeDirectConversionsMap",

  ClosedMultiEntityType: "@blockprotocol/type-system#ClosedMultiEntityType",
  ClosedMultiEntityTypesRootMap:
    "@local/hash-graph-types/ontology#ClosedMultiEntityTypesRootMap",
  ClosedMultiEntityTypesDefinitions:
    "@local/hash-graph-types/ontology#ClosedMultiEntityTypesDefinitions",
  EntityTypeWithMetadata: "@blockprotocol/type-system#EntityTypeWithMetadata",
  ConstructEntityTypeParams:
    "@local/hash-isomorphic-utils/types#ConstructEntityTypeParams",

  PropertyTypeWithMetadata:
    "@blockprotocol/type-system#PropertyTypeWithMetadata",
  ConstructPropertyTypeParams:
    "@local/hash-isomorphic-utils/types#ConstructPropertyTypeParams",

  SerializedEntity: "@local/hash-graph-sdk/entity#SerializedEntity",
  EntityRecordId: "@blockprotocol/type-system#EntityRecordId",
  EntityMetadata: "@blockprotocol/type-system#EntityMetadata",
  EntityRelationAndSubject: "@local/hash-subgraph#EntityRelationAndSubject",
  EntityValidationReport:
    "@local/hash-graph-types/validation#EntityValidationReport",
  CountEntitiesParams: "@local/hash-graph-client#CountEntitiesParams",
  GetEntitySubgraphRequest:
    "@local/hash-isomorphic-utils/types#GetEntitySubgraphRequest",
  EntityTemporalMetadata: "@blockprotocol/type-system#EntityTemporalMetadata",
  PropertyObject: "@blockprotocol/type-system#PropertyObject",
  PropertyArray: "@blockprotocol/type-system#PropertyArray",
  PropertyValue: "@blockprotocol/type-system#PropertyValue",
  PropertyObjectWithMetadata:
    "@blockprotocol/type-system#PropertyObjectWithMetadata",
  PropertyPatchOperation: "@blockprotocol/type-system#PropertyPatchOperation",
  DiffEntityInput: "@local/hash-subgraph#DiffEntityInput",
  DiffEntityResult: "@local/hash-graph-client#DiffEntityResult",
  ValidateEntityParamsComponents:
    "@local/hash-graph-client#ValidateEntityParamsComponents",
  EntityQueryCursor: "@local/hash-graph-client#EntityQueryCursor",
  CreatedByIdsMap: "@local/hash-graph-sdk/entity#CreatedByIdsMap",
  TypeIdsMap: "@local/hash-graph-sdk/entity#TypeIdsMap",
  TypeTitlesMap: "@local/hash-graph-sdk/entity#TypeTitlesMap",
  WebIdsMap: "@local/hash-graph-sdk/entity#WebIdsMap",

  Filter: "@local/hash-graph-client#Filter",

  AggregatedUsageRecord:
    "@local/hash-isomorphic-utils/service-usage#AggregatedUsageRecord",

  UserPermissionsOnEntities:
    "@local/hash-isomorphic-utils/types#UserPermissionsOnEntities",
  UserPermissions: "@local/hash-isomorphic-utils/types#UserPermissions",
  UserPermissionsOnEntityType:
    "@local/hash-isomorphic-utils/types#UserPermissionsOnEntityType",
  UserPermissionsOnDataType:
    "@local/hash-isomorphic-utils/types#UserPermissionsOnDataType",
  ProspectiveUserProperties:
    "@local/hash-isomorphic-utils/system-types/prospectiveuser#ProspectiveUserProperties",

  GraphElementVertexId: "@local/hash-subgraph#GraphElementVertexId",
  Edges: "@local/hash-subgraph#Edges",
  SerializedVertices: "@local/hash-subgraph#SerializedVertices",
  LinkData: "@blockprotocol/type-system#LinkData",
  SubgraphTemporalAxes: "@local/hash-subgraph#SubgraphTemporalAxes",

  OwnedById: "@blockprotocol/type-system#OwnedById",
  EditionCreatedById: "@local/hash-subgraph#EditionCreatedById",
  ActorId: "@blockprotocol/type-system#ActorId",
  ActorGroupId: "@blockprotocol/type-system#ActorGroupId",
  AuthorizationSubjectId:
    "@local/hash-graph-types/authorization#AuthorizationSubjectId",
  EntityId: "@blockprotocol/type-system#EntityId",

  EntityUuid: "@blockprotocol/type-system#EntityUuid",
  Uuid: "@local/hash-graph-types/branded#Uuid",

  OntologyTemporalMetadata: "@local/hash-graph-client#OntologyTemporalMetadata",

  FlowTrigger: "@local/hash-isomorphic-utils/flows/types#FlowTrigger",
  FlowDataSources: "@local/hash-isomorphic-utils/flows/types#FlowDataSources",
  FlowDefinition: "@local/hash-isomorphic-utils/flows/types#FlowDefinition",
  FlowInputs: "@local/hash-isomorphic-utils/flows/types#FlowInputs",
  ExternalInputRequest:
    "@local/hash-isomorphic-utils/flows/types#ExternalInputRequest",
  ExternalInputResponseWithoutUser:
    "@local/hash-isomorphic-utils/flows/types#ExternalInputResponseWithoutUser",
  StepInput: "@local/hash-isomorphic-utils/flows/types#StepInput",
  StepRunOutput: "@local/hash-isomorphic-utils/flows/types#StepRunOutput",
  StepProgressLog: "@local/hash-isomorphic-utils/flows/types#StepProgressLog",
};

export const _localRelativeScalars = Object.fromEntries(
  Object.entries(scalars).map(([key, value]) => [
    key,
    value.replace(/@local\/hash-isomorphic-utils\/([^#]+)(#.*)/g, "../$1.js$2"),
  ]),
);
