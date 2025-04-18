import type { EntityId, OwnedById } from "@blockprotocol/type-system";
import { extractOwnedByIdFromEntityId } from "@blockprotocol/type-system";
import * as Sentry from "@sentry/nextjs";
import { useCallback } from "react";

import { useOrgs } from "./use-orgs";
import { useUsers } from "./use-users";

export const useGetOwnerForEntity = () => {
  /*
   * This is a simple way of getting all users and orgs to find an entity's owner's name
   * This will not scale as it relies on all users and orgs being available in the frontend
   *
   * @todo H-2723 make it possible to request owners along with entities from the graph
   */
  const { users, loading: usersLoading } = useUsers();
  const { orgs, loading: orgsLoading } = useOrgs();

  const loading = usersLoading || orgsLoading;

  return useCallback(
    (params: { entityId: EntityId } | { ownedById: OwnedById }) => {
      const ownedById =
        "entityId" in params
          ? extractOwnedByIdFromEntityId(params.entityId)
          : params.ownedById;

      if (loading || !users?.length || !orgs?.length) {
        return {
          ownedById,
          shortname: "",
        };
      }

      const owner =
        users.find((user) => ownedById === user.accountId) ??
        orgs.find((org) => ownedById === org.accountGroupId);

      if (!owner) {
        Sentry.captureException(
          new Error(
            `Owner with accountId ${ownedById} not found in entities table – possibly a caching issue if it has been created mid-session`,
          ),
        );
        return {
          ownedById,
          shortname: "unknown",
        };
      }

      return {
        ownedById,
        shortname: owner.shortname ?? "incomplete-user-account",
      };
    },
    [loading, orgs, users],
  );
};
