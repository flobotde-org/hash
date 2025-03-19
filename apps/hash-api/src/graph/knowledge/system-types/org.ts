import type {
  ActorGroupId,
  EntityId,
  EntityUuid,
  OwnedById,
} from "@blockprotocol/type-system";
import { extractBaseUrl } from "@blockprotocol/type-system";
import { EntityTypeMismatchError } from "@local/hash-backend-utils/error";
import { createWebMachineActor } from "@local/hash-backend-utils/machine-actors";
import type { Entity } from "@local/hash-graph-sdk/entity";
import { currentTimeInstantTemporalAxes } from "@local/hash-isomorphic-utils/graph-queries";
import {
  systemEntityTypes,
  systemPropertyTypes,
} from "@local/hash-isomorphic-utils/ontology-type-ids";
import { simplifyProperties } from "@local/hash-isomorphic-utils/simplify-properties";
import { mapGraphApiEntityToEntity } from "@local/hash-isomorphic-utils/subgraph-mapping";
import type {
  Organization,
  OrganizationNamePropertyValueWithMetadata,
  OrganizationPropertiesWithMetadata,
} from "@local/hash-isomorphic-utils/system-types/shared";
import type { ActorGroupEntityId } from "@local/hash-subgraph";
import { extractActorGroupId } from "@local/hash-subgraph/stdlib";
import { versionedUrlFromComponents } from "@local/hash-subgraph/type-system-patch";

import { logger } from "../../../logger";
import {
  createAccountGroup,
  createWeb,
} from "../../account-permission-management";
import type {
  ImpureGraphFunction,
  PureGraphFunction,
} from "../../context-types";
import { modifyEntityTypeAuthorizationRelationships } from "../../ontology/primitive/entity-type";
import { systemAccountId } from "../../system-account";
import {
  createEntity,
  getLatestEntityById,
  updateEntity,
} from "../primitive/entity";
import {
  shortnameIsInvalid,
  shortnameIsRestricted,
  shortnameIsTaken,
} from "./account.fields";

export type Org = {
  accountGroupId: ActorGroupId;
  orgName: string;
  shortname: string;
  entity: Entity<Organization>;
};

function assertOrganizationEntity(
  entity: Entity,
  permitOlderVersions = false,
): asserts entity is Entity<Organization> {
  if (
    !entity.metadata.entityTypeIds.find((entityTypeId) =>
      permitOlderVersions
        ? extractBaseUrl(entityTypeId) ===
          systemEntityTypes.organization.entityTypeBaseUrl
        : entityTypeId === systemEntityTypes.organization.entityTypeId,
    )
  ) {
    throw new EntityTypeMismatchError(
      entity.metadata.recordId.entityId,
      systemEntityTypes.organization.entityTypeId,
      entity.metadata.entityTypeIds,
    );
  }
}

export const getOrgFromEntity: PureGraphFunction<
  { entity: Entity; permitOlderVersions?: boolean },
  Org
> = ({ entity, permitOlderVersions }) => {
  assertOrganizationEntity(entity, permitOlderVersions);

  const { organizationName: orgName, shortname } = simplifyProperties(
    entity.properties,
  );

  return {
    accountGroupId: extractActorGroupId(
      entity.metadata.recordId.entityId as ActorGroupEntityId,
    ),
    shortname,
    orgName,
    entity,
  };
};

/**
 * Create a system organization entity.
 *
 * @param params.shortname - the shortname of the organization
 * @param params.name - the name of the organization
 * @param params.providedInfo - optional metadata about the organization
 * @param params.websiteUrl - the website of the organization
 *
 * @see {@link createEntity} for the documentation of the remaining parameters
 */
export const createOrg: ImpureGraphFunction<
  {
    shortname: string;
    name: string;
    orgAccountGroupId?: ActorGroupId;
    websiteUrl?: string | null;
    entityTypeVersion?: number;
    bypassShortnameValidation?: boolean;
  },
  Promise<Org>
> = async (ctx, authentication, params) => {
  const {
    bypassShortnameValidation,
    shortname,
    name,
    websiteUrl,
    entityTypeVersion,
  } = params;

  if (!bypassShortnameValidation && shortnameIsInvalid({ shortname })) {
    throw new Error(`The shortname "${shortname}" is invalid`);
  }

  if (
    (!bypassShortnameValidation && shortnameIsRestricted({ shortname })) ||
    (await shortnameIsTaken(ctx, authentication, { shortname }))
  ) {
    throw new Error(
      `An account or an account group with shortname "${shortname}" already exists.`,
    );
  }

  let orgAccountGroupId: ActorGroupId;
  if (params.orgAccountGroupId) {
    orgAccountGroupId = params.orgAccountGroupId;
  } else {
    orgAccountGroupId = await createAccountGroup(ctx, authentication, {});

    await createWeb(ctx, authentication, {
      ownedById: orgAccountGroupId as OwnedById,
      owner: { kind: "accountGroup", subjectId: orgAccountGroupId },
    });

    await createWebMachineActor(ctx, authentication, {
      ownedById: orgAccountGroupId as OwnedById,
      logger,
    });
  }

  const properties: OrganizationPropertiesWithMetadata = {
    value: {
      "https://hash.ai/@h/types/property-type/shortname/": {
        value: shortname,
        metadata: {
          dataTypeId:
            "https://blockprotocol.org/@blockprotocol/types/data-type/text/v/1",
        },
      },
      "https://hash.ai/@h/types/property-type/organization-name/": {
        value: name,
        metadata: {
          dataTypeId:
            "https://blockprotocol.org/@blockprotocol/types/data-type/text/v/1",
        },
      },
      ...(websiteUrl !== undefined && websiteUrl !== null
        ? {
            "https://hash.ai/@h/types/property-type/website-url/": {
              value: websiteUrl,
              metadata: {
                dataTypeId: "https://hash.ai/@h/types/data-type/uri/v/1",
              },
            },
          }
        : {}),
    },
  };

  const entityTypeId =
    typeof entityTypeVersion === "undefined"
      ? systemEntityTypes.organization.entityTypeId
      : versionedUrlFromComponents(
          systemEntityTypes.organization.entityTypeBaseUrl,
          entityTypeVersion,
        );

  try {
    await modifyEntityTypeAuthorizationRelationships(
      ctx,
      { actorId: systemAccountId },
      [
        {
          operation: "touch",
          relationship: {
            relation: "instantiator",
            subject: {
              kind: "account",
              subjectId: authentication.actorId,
            },
            resource: {
              kind: "entityType",
              resourceId: entityTypeId,
            },
          },
        },
      ],
    );

    const entity = await createEntity(ctx, authentication, {
      ownedById: orgAccountGroupId as OwnedById,
      properties,
      entityTypeIds: [entityTypeId],
      entityUuid: orgAccountGroupId as string as EntityUuid,
      relationships: [
        {
          relation: "viewer",
          subject: {
            kind: "public",
          },
        },
        {
          relation: "setting",
          subject: {
            kind: "setting",
            subjectId: "administratorFromWeb",
          },
        },
      ],
    });

    return getOrgFromEntity({
      entity,
      permitOlderVersions: entityTypeVersion !== undefined,
    });
  } finally {
    if (authentication.actorId !== systemAccountId) {
      await modifyEntityTypeAuthorizationRelationships(
        ctx,
        { actorId: systemAccountId },
        [
          {
            operation: "delete",
            relationship: {
              relation: "instantiator",
              subject: {
                kind: "account",
                subjectId: authentication.actorId,
              },
              resource: {
                kind: "entityType",
                resourceId: entityTypeId,
              },
            },
          },
        ],
      );
    }
  }
};

/**
 * Get a system organization entity by its entity id.
 *
 * @param params.entityId - the entity id of the organization
 */
export const getOrgById: ImpureGraphFunction<
  { entityId: EntityId },
  Promise<Org>
> = async (ctx, authentication, { entityId }) => {
  const entity = await getLatestEntityById(ctx, authentication, {
    entityId,
  });

  return getOrgFromEntity({ entity });
};

/**
 * Get a system organization entity by its shortname.
 *
 * @param params.shortname - the shortname of the organization
 */
export const getOrgByShortname: ImpureGraphFunction<
  { permitOlderVersions?: boolean; shortname: string },
  Promise<Org | null>
> = async ({ graphApi }, { actorId }, params) => {
  const [orgEntity, ...unexpectedEntities] = await graphApi
    .getEntities(actorId, {
      filter: {
        all: [
          {
            equal: [
              { path: ["type(inheritanceDepth = 0)", "baseUrl"] },
              { parameter: systemEntityTypes.organization.entityTypeBaseUrl },
            ],
          },
          {
            equal: [
              {
                path: [
                  "properties",
                  systemPropertyTypes.shortname.propertyTypeBaseUrl,
                ],
              },
              { parameter: params.shortname },
            ],
          },
        ],
      },
      // TODO: Should this be an all-time query? What happens if the org is
      //       archived/deleted, do we want to allow orgs to replace their
      //       shortname?
      //   see https://linear.app/hash/issue/H-757
      temporalAxes: currentTimeInstantTemporalAxes,
      includeDrafts: false,
    })
    .then(({ data: response }) =>
      response.entities.map((entity) =>
        mapGraphApiEntityToEntity(entity, actorId),
      ),
    );

  if (unexpectedEntities.length > 0) {
    throw new Error(
      `Critical: More than one org entity with shortname ${params.shortname} found in the graph.`,
    );
  }

  return orgEntity
    ? getOrgFromEntity({
        entity: orgEntity,
        permitOlderVersions: params.permitOlderVersions,
      })
    : null;
};

/**
 * Whether an org name is invalid
 *
 * @param params.orgName - the org name
 */
export const orgNameIsInvalid: PureGraphFunction<
  { orgName: string },
  boolean
> = ({ orgName }) => {
  return orgName === "";
};

/**
 * Update the name of an Organization
 *
 * @param params.org - the org
 * @param params.updatedOrgName - the new name to assign to the Organization
 * @param params.actorId - the id of the account updating the name
 */
export const updateOrgName: ImpureGraphFunction<
  { org: Org; updatedOrgName: string },
  Promise<Org>,
  false,
  true
> = async (ctx, authentication, params) => {
  const { org, updatedOrgName } = params;

  if (orgNameIsInvalid({ orgName: updatedOrgName })) {
    throw new Error(`Organization name "${updatedOrgName}" is invalid.`);
  }

  const updatedEntity = await updateEntity(ctx, authentication, {
    entity: org.entity,
    propertyPatches: [
      {
        op: "replace",
        path: [systemPropertyTypes.organizationName.propertyTypeBaseUrl],
        property: {
          value: updatedOrgName,
          metadata: {
            dataTypeId:
              "https://blockprotocol.org/@blockprotocol/types/data-type/text/v/1",
          },
        } satisfies OrganizationNamePropertyValueWithMetadata,
      },
    ],
  });

  return getOrgFromEntity({ entity: updatedEntity });
};
