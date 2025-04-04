import type { BaseUrl } from "@blockprotocol/type-system";
import { frontendUrl } from "@local/hash-isomorphic-utils/environment";
import type { SystemTypeWebShortname } from "@local/hash-isomorphic-utils/ontology-types";
import { systemTypeWebShortnames } from "@local/hash-isomorphic-utils/ontology-types";

export const getTypeBaseUrl = ({
  slug,
  namespaceWithAt,
  kind,
}: {
  slug: string;
  namespaceWithAt: `@${string}`;
  kind: "entity-type" | "data-type";
}): BaseUrl =>
  `${
    // To be removed in H-1172: Temporary provision until https://app.hash.ai migrated to https://hash.ai
    // To be replaced with simply 'frontendUrl'
    (
      [...systemTypeWebShortnames, "ftse"].includes(
        namespaceWithAt.slice(1) as SystemTypeWebShortname,
      ) && frontendUrl === "http://localhost:3000"
    ) || frontendUrl === "https://app.hash.ai"
      ? "https://hash.ai"
      : frontendUrl
  }/${namespaceWithAt}/types/${kind}/${slug}/` as BaseUrl;
