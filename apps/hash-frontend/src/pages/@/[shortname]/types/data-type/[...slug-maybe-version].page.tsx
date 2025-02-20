import type { DataTypeWithMetadata } from "@blockprotocol/graph";
import { componentsFromVersionedUrl } from "@local/hash-subgraph/type-system-patch";
import { Buffer } from "buffer/";
import { useRouter } from "next/router";
import { useMemo } from "react";

import type { NextPageWithLayout } from "../../../../../shared/layout";
import { getLayoutWithSidebar } from "../../../../../shared/layout";
import { DataType } from "../../../../shared/data-type";
import { useRouteNamespace } from "../../shared/use-route-namespace";
import { getTypeBaseUrl } from "../shared/get-type-base-url";

const Page: NextPageWithLayout = () => {
  const router = useRouter();

  const isDraft = !!router.query.draft;
  const { loading: loadingNamespace, routeNamespace } = useRouteNamespace();

  /**
   * router.query is not populated in this page, probably some combination of the rewrite @[shortname] routes and the fact this is a catch all.
   * We have to parse out the path ourselves.
   *
   * @see https://github.com/vercel/next.js/issues/50212 –– possibly related
   *
   * @example /@hash/types/data-type/integer/v/1
   * @example /@hash/types/data-type/integer
   */
  const [
    _,
    shortnameWithAt,
    _types,
    _dataType,
    slug,
    _v,
    requestedVersionString,
  ] = router.asPath.split("/") as [
    "",
    `@${string}`,
    "types",
    "data-type",
    string,
    "v" | undefined,
    `${number}` | undefined,
  ];

  const dataTypeBaseUrl = !isDraft
    ? getTypeBaseUrl({
        slug,
        namespaceWithAt: shortnameWithAt,
        kind: "data-type",
      })
    : undefined;

  const draftDataType = useMemo(() => {
    if (router.query.draft) {
      const dataTypeSchema = JSON.parse(
        Buffer.from(
          decodeURIComponent(router.query.draft.toString()),
          "base64",
        ).toString("utf8"),
      );

      const { baseUrl, version } = componentsFromVersionedUrl(
        dataTypeSchema.$id,
      );

      return {
        metadata: {
          recordId: {
            baseUrl,
            version,
          },
        },
        schema: dataTypeSchema,
      } satisfies DataTypeWithMetadata;
    } else {
      return null;
    }
  }, [router.query.draft]);

  const requestedVersion = requestedVersionString
    ? parseInt(requestedVersionString, 10)
    : null;

  if (!routeNamespace) {
    if (loadingNamespace) {
      return null;
    } else {
      throw new Error("Namespace for valid data type somehow missing");
    }
  }

  return (
    <DataType
      ownedById={routeNamespace.ownedById}
      draftNewDataType={draftDataType}
      dataTypeBaseUrl={dataTypeBaseUrl}
      key={`${dataTypeBaseUrl}-${requestedVersion}`}
      requestedVersion={requestedVersion}
    />
  );
};

Page.getLayout = (page) =>
  getLayoutWithSidebar(page, {
    fullWidth: true,
  });

export default Page;
