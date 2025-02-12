import type { AccountId } from "@local/hash-graph-types/account";
import { useRouter } from "next/router";
import { useMemo } from "react";

import { useGetAccountIdForShortname } from "../../../../components/hooks/use-get-account-id-for-shortname";

export const useRouteNamespace = (): {
  loading: boolean;
  routeNamespace?: {
    accountId: AccountId;
    shortname?: string;
  };
} => {
  const router = useRouter();
  let shortname = router.query.shortname;

  if (Array.isArray(shortname)) {
    throw new Error("shortname can't be an array");
  }

  if (!shortname) {
    /**
     * router.query is not populated in [...slug-maybe-version].page.tsx, probably some combination of the rewrite @[shortname] routes and the fact it is a catch all.
     * We have to parse out the path ourselves.
     *
     * @see https://github.com/vercel/next.js/issues/50212 –– possibly related
     */
    shortname = router.asPath.match(/\/@([^/]+)/)?.[1];
  }

  const { loading, accountId } = useGetAccountIdForShortname(shortname);

  return useMemo(() => {
    if (!loading && accountId) {
      return {
        loading,
        routeNamespace: {
          accountId,
          shortname,
        },
      };
    } else {
      return { loading };
    }
  }, [loading, accountId, shortname]);
};
