import type {
  Edges as EdgesBp,
  EntityRevisionId,
  KnowledgeGraphRootedEdges as KnowledgeGraphRootedEdgesBp,
  OntologyRootedEdges as OntologyRootedEdgesBp,
} from "@blockprotocol/graph";
import type {
  BaseUrl,
  EntityId,
  OntologyTypeVersion,
} from "@blockprotocol/type-system";
import type { Subtype } from "@local/advanced-types/subtype";

import type {
  KnowledgeGraphOutwardEdge,
  OntologyOutwardEdge,
} from "./edges/variants.js";

export * from "./edges/kind.js";
export * from "./edges/outward-edge.js";
export * from "./edges/variants.js";

export type OntologyRootedEdges = Subtype<
  OntologyRootedEdgesBp,
  {
    [baseUrl: BaseUrl]: {
      [revisionId: OntologyTypeVersion]: OntologyOutwardEdge[];
    };
  }
>;

export type KnowledgeGraphRootedEdges = Subtype<
  KnowledgeGraphRootedEdgesBp,
  {
    [entityId: EntityId]: {
      [fromTime: EntityRevisionId]: KnowledgeGraphOutwardEdge[];
    };
  }
>;

export type Edges = OntologyRootedEdges & KnowledgeGraphRootedEdges;
/**
 * This provides a sanity check that we've almost correctly expressed `Edges` as a subtype of the Block Protocol one.
 *
 * We unfortunately need these two different types because in the Block Protocol we had to use `|` instead of `&` due
 * to overlapping index types. We _wanted_ to use `&` but it produces unsatisfiable types. However, because we have
 * branded types here (thus the index types do not overlap) we can do better in HASH and use `&`, although this confuses
 * TypeScript and it thinks they are incompatible. Thus, the strange check type.
 */
export type _CheckEdges = Subtype<
  EdgesBp,
  OntologyRootedEdges | KnowledgeGraphRootedEdges
>;
