import { deleteKratosIdentity } from "@apps/hash-api/src/auth/ory-kratos";
import { ensureSystemGraphIsInitialized } from "@apps/hash-api/src/graph/ensure-system-graph-is-initialized";
import type { Org } from "@apps/hash-api/src/graph/knowledge/system-types/org";
import type { User } from "@apps/hash-api/src/graph/knowledge/system-types/user";
import { joinOrg } from "@apps/hash-api/src/graph/knowledge/system-types/user";
import {
  archiveEntityType,
  createEntityType,
  getClosedEntityTypes,
  getClosedMultiEntityType,
  getEntityTypeById,
  getEntityTypes,
  getEntityTypeSubgraph,
  getEntityTypeSubgraphById,
  unarchiveEntityType,
  updateEntityType,
} from "@apps/hash-api/src/graph/ontology/primitive/entity-type";
import { createPropertyType } from "@apps/hash-api/src/graph/ontology/primitive/property-type";
import type {
  ClosedEntityType,
  ClosedMultiEntityType,
} from "@blockprotocol/type-system";
import { atLeastOne } from "@blockprotocol/type-system";
import { Logger } from "@local/hash-backend-utils/logger";
import { publicUserAccountId } from "@local/hash-backend-utils/public-user-account-id";
import type {
  ClosedEntityTypeWithMetadata,
  EntityTypeWithMetadata,
  PropertyTypeWithMetadata,
} from "@local/hash-graph-types/ontology";
import type { OwnedById } from "@local/hash-graph-types/web";
import {
  currentTimeInstantTemporalAxes,
  fullTransactionTimeAxis,
  zeroedGraphResolveDepths,
} from "@local/hash-isomorphic-utils/graph-queries";
import { systemEntityTypes } from "@local/hash-isomorphic-utils/ontology-type-ids";
import type {
  ConstructEntityTypeParams,
  SystemDefinedProperties,
} from "@local/hash-isomorphic-utils/types";
import {
  isOwnedOntologyElementMetadata,
  linkEntityTypeUrl,
} from "@local/hash-subgraph";
import { getDataTypes, getPropertyTypes } from "@local/hash-subgraph/stdlib";
import { assert, beforeAll, describe, expect, it } from "vitest";

import { resetGraph } from "../../../test-server";
import {
  createTestImpureGraphContext,
  createTestOrg,
  createTestUser,
  textDataTypeId,
} from "../../../util";

const logger = new Logger({
  environment: "test",
  level: "debug",
  serviceName: "integration-tests",
});

const graphContext = createTestImpureGraphContext();

let testOrg: Org;
let testUser: User;
let testUser2: User;
let entityTypeSchema: ConstructEntityTypeParams;
let workerEntityType: EntityTypeWithMetadata;
let namePropertyType: PropertyTypeWithMetadata;
let favoriteBookPropertyType: PropertyTypeWithMetadata;
let knowsLinkEntityType: EntityTypeWithMetadata;
let previousAddressLinkEntityType: EntityTypeWithMetadata;
let addressEntityType: EntityTypeWithMetadata;

beforeAll(async () => {
  await ensureSystemGraphIsInitialized({ logger, context: graphContext });

  testUser = await createTestUser(graphContext, "entity-type-test-1", logger);
  testUser2 = await createTestUser(graphContext, "entity-type-test-2", logger);

  const authentication = { actorId: testUser.accountId };

  testOrg = await createTestOrg(
    graphContext,
    authentication,
    "entitytypetestorg",
  );
  await joinOrg(graphContext, authentication, {
    userEntityId: testUser2.entity.metadata.recordId.entityId,
    orgEntityId: testOrg.entity.metadata.recordId.entityId,
  });

  await Promise.all([
    createEntityType(graphContext, authentication, {
      ownedById: testUser.accountId as OwnedById,
      schema: {
        title: "Worker",
        description: "A worker",
        type: "object",
        properties: {},
      },
      relationships: [
        {
          relation: "viewer",
          subject: {
            kind: "public",
          },
        },
      ],
    }).then((val) => {
      workerEntityType = val;
    }),
    createEntityType(graphContext, authentication, {
      ownedById: testUser.accountId as OwnedById,
      schema: {
        title: "Address",
        description: "An address",
        type: "object",
        properties: {},
      },
      relationships: [
        {
          relation: "viewer",
          subject: {
            kind: "public",
          },
        },
      ],
    }).then((val) => {
      addressEntityType = val;
    }),
    createPropertyType(graphContext, authentication, {
      ownedById: testUser.accountId as OwnedById,
      schema: {
        title: "Favorite Book",
        description: "Favorite book of the user",
        oneOf: [{ $ref: textDataTypeId }],
      },
      relationships: [
        {
          relation: "viewer",
          subject: {
            kind: "public",
          },
        },
      ],
    }).then((val) => {
      favoriteBookPropertyType = val;
    }),
    createPropertyType(graphContext, authentication, {
      ownedById: testUser.accountId as OwnedById,
      schema: {
        title: "Name",
        description: "The name of the user",
        oneOf: [{ $ref: textDataTypeId }],
      },
      relationships: [
        {
          relation: "viewer",
          subject: {
            kind: "public",
          },
        },
      ],
    }).then((val) => {
      namePropertyType = val;
    }),
    createEntityType(graphContext, authentication, {
      ownedById: testUser.accountId as OwnedById,
      schema: {
        title: "Knows",
        description: "Knows of someone",
        type: "object",
        allOf: [{ $ref: linkEntityTypeUrl }],
        properties: {},
        ...({} as Record<SystemDefinedProperties, never>),
      },
      relationships: [
        {
          relation: "viewer",
          subject: {
            kind: "public",
          },
        },
      ],
    }).then((val) => {
      knowsLinkEntityType = val;
    }),
    createEntityType(graphContext, authentication, {
      ownedById: testUser.accountId as OwnedById,
      schema: {
        title: "Previous Address",
        description: "A previous address of something.",
        type: "object",
        allOf: [{ $ref: linkEntityTypeUrl }],
        properties: {},
      },
      relationships: [
        {
          relation: "viewer",
          subject: {
            kind: "public",
          },
        },
      ],
    }).then((val) => {
      previousAddressLinkEntityType = val;
    }),
  ]);

  entityTypeSchema = {
    title: "Some",
    description: "An object",
    type: "object",
    properties: {
      [favoriteBookPropertyType.metadata.recordId.baseUrl]: {
        $ref: favoriteBookPropertyType.schema.$id,
      },
      [namePropertyType.metadata.recordId.baseUrl]: {
        $ref: namePropertyType.schema.$id,
      },
    },
    links: {
      [knowsLinkEntityType.schema.$id]: {
        type: "array",
        items: {
          oneOf: [{ $ref: workerEntityType.schema.$id }],
        },
      },
      [previousAddressLinkEntityType.schema.$id]: {
        type: "array",
        items: {
          oneOf: [{ $ref: addressEntityType.schema.$id }],
        },
      },
    },
  };

  return async () => {
    await deleteKratosIdentity({
      kratosIdentityId: testUser.kratosIdentityId,
    });
    await deleteKratosIdentity({
      kratosIdentityId: testUser2.kratosIdentityId,
    });
    await resetGraph();
  };
});

describe("Entity type CRU", () => {
  let createdEntityType: EntityTypeWithMetadata;

  it("can create an entity type", async () => {
    const authentication = { actorId: testUser.accountId };

    createdEntityType = await createEntityType(graphContext, authentication, {
      ownedById: testOrg.accountGroupId as OwnedById,
      schema: entityTypeSchema,
      relationships: [
        {
          relation: "setting",
          subject: {
            kind: "setting",
            subjectId: "updateFromWeb",
          },
        },
        {
          relation: "viewer",
          subject: {
            kind: "public",
          },
        },
      ],
    });
  });

  it("can read an entity type", async () => {
    const authentication = { actorId: testUser.accountId };

    const fetchedEntityType = await getEntityTypeById(
      graphContext,
      authentication,
      {
        entityTypeId: createdEntityType.schema.$id,
      },
    );

    expect(fetchedEntityType.schema).toEqual(createdEntityType.schema);
  });

  it("can read a closed entity type", async () => {
    const authentication = { actorId: testUser.accountId };

    const userType = await getEntityTypeById(graphContext, authentication, {
      entityTypeId: systemEntityTypes.user.entityTypeId,
    });
    const actorType = await getEntityTypeById(graphContext, authentication, {
      entityTypeId: systemEntityTypes.actor.entityTypeId,
    });

    const fetchedEntityType = await getClosedEntityTypes(
      graphContext,
      authentication,
      {
        filter: {
          equal: [
            { path: ["versionedUrl"] },
            { parameter: userType.schema.$id },
          ],
        },
        temporalAxes: currentTimeInstantTemporalAxes,
      },
    );

    // It's not specified how `required` is ordered, so we need to sort it before comparing
    for (const entityType of fetchedEntityType) {
      if (entityType.schema.required) {
        entityType.schema.required.sort();
      }
    }

    expect(fetchedEntityType).toEqual([
      {
        metadata: userType.metadata,
        schema: {
          $id: userType.schema.$id,
          title: userType.schema.title,
          description: userType.schema.description,
          properties: {
            ...userType.schema.properties,
            ...actorType.schema.properties,
          },
          required: atLeastOne(
            Array.from(
              new Set([
                ...(userType.schema.required ?? []),
                ...(actorType.schema.required ?? []),
              ]),
            ).toSorted(),
          ),
          links: {
            ...(userType.schema.links ?? {}),
            ...(userType.schema.links ?? {}),
          },
          allOf: [
            {
              depth: 0,
              $id: systemEntityTypes.user.entityTypeId,
              icon: "/icons/types/user.svg",
            },
            {
              depth: 1,
              $id: systemEntityTypes.actor.entityTypeId,
              icon: "/icons/types/user.svg",
              labelProperty:
                "https://blockprotocol.org/@blockprotocol/types/property-type/display-name/",
            },
          ],
        } satisfies ClosedEntityType,
      },
    ]);
  });

  it("can read a closed multi-entity type", async () => {
    const authentication = { actorId: testUser.accountId };

    const closedEntityTypes = (await getClosedEntityTypes(
      graphContext,
      authentication,
      {
        filter: {
          any: [
            {
              equal: [
                { path: ["versionedUrl"] },
                { parameter: systemEntityTypes.user.entityTypeId },
              ],
            },
            {
              equal: [
                { path: ["versionedUrl"] },
                { parameter: systemEntityTypes.actor.entityTypeId },
              ],
            },
          ],
        },
        temporalAxes: currentTimeInstantTemporalAxes,
      },
    )) as [ClosedEntityTypeWithMetadata, ...ClosedEntityTypeWithMetadata[]];

    const { entityType: closedMultiEntityType, definitions } =
      await getClosedMultiEntityType(graphContext, authentication, {
        entityTypeIds: [
          systemEntityTypes.user.entityTypeId,
          systemEntityTypes.actor.entityTypeId,
        ],
        temporalAxes: currentTimeInstantTemporalAxes,
        includeDrafts: false,
        includeResolved: "resolved",
      });

    // It's not specified how `required` is ordered, so we need to sort it before comparing
    if (closedMultiEntityType.required) {
      closedMultiEntityType.required.sort();
    }

    const allOf = atLeastOne(
      closedEntityTypes.map(
        (closedEntityType) =>
          ({
            $id: closedEntityType.schema.$id,
            title: closedEntityType.schema.title,
            description: closedEntityType.schema.description,
            allOf: closedEntityType.schema.allOf,
          }) as ClosedMultiEntityType["allOf"][0],
      ),
    );
    assert(allOf !== undefined);

    expect(closedMultiEntityType).toEqual({
      allOf,
      properties: closedEntityTypes.reduce(
        (acc, closedEntityType) => {
          return { ...acc, ...closedEntityType.schema.properties };
        },
        {} as ClosedMultiEntityType["properties"],
      ),
      required: atLeastOne(
        Array.from(
          new Set(
            closedEntityTypes.flatMap(
              (closedEntityType) => closedEntityType.schema.required ?? [],
            ),
          ),
        ).toSorted(),
      ),
      links: closedEntityTypes.reduce(
        (acc, closedEntityType) => {
          return { ...acc, ...closedEntityType.schema.links };
        },
        {} as ClosedMultiEntityType["links"],
      ),
    } satisfies ClosedMultiEntityType);

    const subgraph = await getEntityTypeSubgraph(graphContext, authentication, {
      filter: {
        any: [
          {
            equal: [
              { path: ["versionedUrl"] },
              { parameter: systemEntityTypes.user.entityTypeId },
            ],
          },
          {
            equal: [
              { path: ["versionedUrl"] },
              { parameter: systemEntityTypes.actor.entityTypeId },
            ],
          },
        ],
      },
      graphResolveDepths: {
        ...zeroedGraphResolveDepths,
        constrainsPropertiesOn: { outgoing: 255 },
        constrainsValuesOn: { outgoing: 255 },
      },
      temporalAxes: currentTimeInstantTemporalAxes,
    });

    for (const propertyType of getPropertyTypes(subgraph)) {
      expect(propertyType.schema).toEqual(
        definitions?.propertyTypes[propertyType.schema.$id],
      );
    }
    for (const dataType of getDataTypes(subgraph)) {
      expect(definitions?.dataTypes[dataType.schema.$id]).toBeDefined();
    }
  });

  const updatedTitle = "New text!";

  it("can update an entity type", async () => {
    expect(
      isOwnedOntologyElementMetadata(createdEntityType.metadata) &&
        createdEntityType.metadata.provenance.edition.createdById,
    ).toBe(testUser.accountId);

    const authentication = { actorId: testUser2.accountId };

    const updatedEntityType = await updateEntityType(
      graphContext,
      authentication,
      {
        entityTypeId: createdEntityType.schema.$id,
        schema: { ...entityTypeSchema, title: updatedTitle },
        relationships: [
          {
            relation: "setting",
            subject: {
              kind: "setting",
              subjectId: "updateFromWeb",
            },
          },
          { relation: "instantiator", subject: { kind: "public" } },
        ],
        provenance: {
          actorType: "machine",
          origin: {
            type: "api",
          },
        },
      },
      // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
    ).catch((err) => Promise.reject(err.data));

    expect(
      isOwnedOntologyElementMetadata(updatedEntityType.metadata) &&
        updatedEntityType.metadata.provenance.edition.createdById,
    ).toBe(testUser2.accountId);
  });

  it("can archive a entity type", async () => {
    const authentication = { actorId: testUser.accountId };

    await archiveEntityType(graphContext, authentication, {
      entityTypeId: createdEntityType.schema.$id,
    });

    const [archivedEntityType] = await getEntityTypes(
      graphContext,
      authentication,
      {
        filter: {
          equal: [
            { path: ["versionedUrl"] },
            { parameter: createdEntityType.schema.$id },
          ],
        },
        temporalAxes: fullTransactionTimeAxis,
      },
    );

    expect(
      await getEntityTypes(graphContext, authentication, {
        filter: {
          equal: [
            { path: ["versionedUrl"] },
            { parameter: createdEntityType.schema.$id },
          ],
        },
        temporalAxes: currentTimeInstantTemporalAxes,
      }),
    ).toHaveLength(0);

    expect(
      archivedEntityType?.metadata.temporalVersioning.transactionTime.end.kind,
    ).toBe("exclusive");

    await unarchiveEntityType(graphContext, authentication, {
      entityTypeId: createdEntityType.schema.$id,
    });

    const [unarchivedEntityType] = await getEntityTypes(
      graphContext,
      authentication,
      {
        filter: {
          equal: [
            { path: ["versionedUrl"] },
            { parameter: createdEntityType.schema.$id },
          ],
        },
        temporalAxes: fullTransactionTimeAxis,
      },
    );

    expect(
      unarchivedEntityType?.metadata.temporalVersioning.transactionTime.end
        .kind,
    ).toBe("unbounded");
  });

  it.skip("can load an external type on demand", async () => {
    const authentication = { actorId: testUser.accountId };

    const entityTypeId =
      "https://blockprotocol.org/@blockprotocol/types/entity-type/thing/v/1";

    await expect(
      getEntityTypeById(
        graphContext,
        { actorId: publicUserAccountId },
        { entityTypeId },
      ),
    ).rejects.toThrow("Could not find entity type with ID");

    await expect(
      getEntityTypeSubgraphById(graphContext, authentication, {
        entityTypeId,
        graphResolveDepths: zeroedGraphResolveDepths,
        temporalAxes: currentTimeInstantTemporalAxes,
      }),
    ).resolves.not.toThrow();
  });
});
