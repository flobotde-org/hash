import {
  BaseUrl,
  ENTITY_TYPE_META_SCHEMA,
  VersionedUrl,
} from "@blockprotocol/type-system";
import {
  EntityType,
  EntityTypeStructuralQuery,
  OntologyTemporalMetadata,
  UpdateEntityTypeRequest,
} from "@local/hash-graph-client";
import { ConstructEntityTypeParams } from "@local/hash-graphql-shared/graphql/types";
import { frontendUrl } from "@local/hash-isomorphic-utils/environment";
import {
  currentTimeInstantTemporalAxes,
  zeroedGraphResolveDepths,
} from "@local/hash-isomorphic-utils/graph-queries";
import { generateTypeId } from "@local/hash-isomorphic-utils/ontology-types";
import {
  EntityTypeMetadata,
  EntityTypeRootType,
  EntityTypeWithMetadata,
  linkEntityTypeUrl,
  OntologyTypeRecordId,
  ontologyTypeRecordIdToVersionedUrl,
  OwnedById,
  Subgraph,
} from "@local/hash-subgraph";
import {
  getRoots,
  mapGraphApiSubgraphToSubgraph,
} from "@local/hash-subgraph/stdlib";

import { NotFoundError } from "../../../lib/error";
import { ImpureGraphFunction } from "../..";
import { getNamespaceOfAccountOwner } from "./util";

/**
 * Create an entity type.
 *
 * @param params.ownedById - the id of the account who owns the entity type
 * @param params.schema - the `EntityType`
 * @param params.actorId - the id of the account that is creating the entity type
 */
export const createEntityType: ImpureGraphFunction<
  {
    ownedById: OwnedById;
    schema: ConstructEntityTypeParams;

    labelProperty?: BaseUrl;
  },
  Promise<EntityTypeWithMetadata>
> = async (ctx, authentication, params) => {
  const { ownedById, labelProperty } = params;
  const namespace = await getNamespaceOfAccountOwner(ctx, authentication, {
    ownerId: params.ownedById,
  });

  const entityTypeId = generateTypeId({
    namespace,
    kind: "entity-type",
    title: params.schema.title,
  });

  const schema = {
    $schema: ENTITY_TYPE_META_SCHEMA,
    kind: "entityType" as const,
    $id: entityTypeId,
    ...params.schema,
  };

  const { graphApi } = ctx;

  const { data: metadata } = await graphApi.createEntityType(
    authentication.actorId,
    {
      ownedById,
      schema,
      labelProperty,
    },
  );

  return { schema, metadata: metadata as EntityTypeMetadata };
};

/**
 * Get entity types by a structural query.
 *
 * @param params.query the structural query to filter entity types by.
 */
export const getEntityTypes: ImpureGraphFunction<
  {
    query: EntityTypeStructuralQuery;
  },
  Promise<Subgraph<EntityTypeRootType>>
> = async ({ graphApi }, { actorId }, { query }) => {
  return await graphApi
    .getEntityTypesByQuery(actorId, query)
    .then(({ data }) => {
      const subgraph = mapGraphApiSubgraphToSubgraph<EntityTypeRootType>(data);

      return subgraph;
    });
};

/**
 * Get an entity type by its versioned URL.
 *
 * @param params.entityTypeId the unique versioned URL for an entity type.
 */
export const getEntityTypeById: ImpureGraphFunction<
  {
    entityTypeId: VersionedUrl;
  },
  Promise<EntityTypeWithMetadata>
> = async (context, authentication, params) => {
  const { entityTypeId } = params;

  const [entityType] = await getEntityTypes(context, authentication, {
    query: {
      filter: {
        equal: [{ path: ["versionedUrl"] }, { parameter: entityTypeId }],
      },
      graphResolveDepths: zeroedGraphResolveDepths,
      temporalAxes: currentTimeInstantTemporalAxes,
    },
  }).then(getRoots);

  if (!entityType) {
    throw new NotFoundError(
      `Could not find entity type with ID "${entityTypeId}"`,
    );
  }

  return entityType;
};

/**
 * Get an entity type rooted subgraph by its versioned URL.
 *
 * If the type does not already exist within the Graph, and is an externally-hosted type, this will also load the type into the Graph.
 */
export const getEntityTypeSubgraphById: ImpureGraphFunction<
  Omit<EntityTypeStructuralQuery, "filter"> & {
    entityTypeId: VersionedUrl;
  },
  Promise<Subgraph<EntityTypeRootType>>
> = async (context, authentication, params) => {
  const { graphResolveDepths, temporalAxes, entityTypeId } = params;

  const query: EntityTypeStructuralQuery = {
    filter: {
      equal: [{ path: ["versionedUrl"] }, { parameter: entityTypeId }],
    },
    graphResolveDepths,
    temporalAxes,
  };

  let subgraph = await getEntityTypes(context, authentication, {
    query,
  });

  if (subgraph.roots.length === 0 && !entityTypeId.startsWith(frontendUrl)) {
    await context.graphApi.loadExternalEntityType(authentication.actorId, {
      entityTypeId,
    });

    subgraph = await getEntityTypes(context, authentication, {
      query,
    });
  }

  return subgraph;
};

/**
 * Update an entity type.
 *
 * @param params.entityTypeId - the id of the entity type that's being updated
 * @param params.schema - the updated `EntityType`
 * @param params.actorId - the id of the account that is updating the entity type
 */
export const updateEntityType: ImpureGraphFunction<
  {
    entityTypeId: VersionedUrl;
    schema: ConstructEntityTypeParams;

    labelProperty?: BaseUrl;
  },
  Promise<EntityTypeWithMetadata>
> = async ({ graphApi }, authentication, params) => {
  const { entityTypeId, schema, labelProperty } = params;
  const updateArguments: UpdateEntityTypeRequest = {
    typeToUpdate: entityTypeId,
    schema: {
      kind: "entityType",
      $schema: ENTITY_TYPE_META_SCHEMA,
      ...schema,
    },
    labelProperty,
  };

  const { data: metadata } = await graphApi.updateEntityType(
    authentication.actorId,
    updateArguments,
  );

  const { recordId } = metadata;

  return {
    schema: {
      kind: "entityType",
      $schema: ENTITY_TYPE_META_SCHEMA,
      ...schema,
      $id: ontologyTypeRecordIdToVersionedUrl(recordId as OntologyTypeRecordId),
    },
    metadata: metadata as EntityTypeMetadata,
  };
};

// Return true if any type in the provided entity type's ancestors is a link entity type
export const isEntityTypeLinkEntityType: ImpureGraphFunction<
  Pick<EntityType, "allOf">,
  Promise<boolean>
> = async (context, authentication, params) => {
  const { allOf } = params;

  if (allOf?.some(({ $ref }) => $ref === linkEntityTypeUrl)) {
    return true;
  }

  const parentTypes = await Promise.all(
    (allOf ?? []).map(async ({ $ref }) =>
      getEntityTypeById(context, authentication, {
        entityTypeId: $ref as VersionedUrl,
      }),
    ),
  );

  return new Promise((resolve) => {
    const promises = parentTypes.map((parent) =>
      isEntityTypeLinkEntityType(context, authentication, parent.schema).then(
        (isLinkType) => {
          if (isLinkType) {
            // Resolve as soon as we have encountered a link type, instead of waiting for all parent types to be checked
            resolve(true);
          }
        },
      ),
    );

    void Promise.all(promises).then(() =>
      // If we haven't resolved yet, then none of the parent types are link types. If we have resolved this is a no-op.
      resolve(false),
    );
  });
};

/**
 * Archives a data type
 *
 * @param params.entityTypeId - the id of the entity type that's being archived
 * @param params.actorId - the id of the account that is archiving the entity type
 */
export const archiveEntityType: ImpureGraphFunction<
  {
    entityTypeId: VersionedUrl;
  },
  Promise<OntologyTemporalMetadata>
> = async ({ graphApi }, { actorId }, params) => {
  const { entityTypeId } = params;

  const { data: temporalMetadata } = await graphApi.archiveEntityType(actorId, {
    typeToArchive: entityTypeId,
  });

  return temporalMetadata;
};

/**
 * Unarchives a data type
 *
 * @param params.entityTypeId - the id of the entity type that's being unarchived
 * @param params.actorId - the id of the account that is unarchiving the entity type
 */
export const unarchiveEntityType: ImpureGraphFunction<
  {
    entityTypeId: VersionedUrl;
  },
  Promise<OntologyTemporalMetadata>
> = async ({ graphApi }, { actorId }, params) => {
  const { entityTypeId } = params;

  const { data: temporalMetadata } = await graphApi.unarchiveEntityType(
    actorId,
    {
      typeToUnarchive: entityTypeId,
    },
  );

  return temporalMetadata;
};
