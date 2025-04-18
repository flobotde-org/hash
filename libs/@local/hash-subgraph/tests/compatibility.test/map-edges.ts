import type {
  BaseUrl,
  EntityId,
  OntologyTypeVersion,
  Timestamp,
} from "@blockprotocol/type-system";
import { isEntityId, validateBaseUrl } from "@blockprotocol/type-system";
import type {
  Edges as EdgesGraphApi,
  EntityIdWithInterval as EntityIdWithIntervalGraphApi,
  ExclusiveBound as ExclusiveBoundGraphApi,
  KnowledgeGraphOutwardEdge as KnowledgeGraphOutwardEdgeGraphApi,
  OntologyOutwardEdge as OntologyOutwardEdgeGraphApi,
  OntologyTypeVertexId as OntologyTypeVertexIdGraphApi,
} from "@local/hash-graph-client";

import type { Edges, OutwardEdge } from "../../src/main.js";
import {
  isKnowledgeGraphOutwardEdge,
  isOntologyOutwardEdge,
} from "../../src/main.js";

export const mapOutwardEdge = (
  outwardEdge: OntologyOutwardEdgeGraphApi | KnowledgeGraphOutwardEdgeGraphApi,
): OutwardEdge => {
  switch (outwardEdge.kind) {
    // Ontology edge-kind cases
    case "CONSTRAINS_PROPERTIES_ON":
    case "CONSTRAINS_LINKS_ON":
    case "CONSTRAINS_LINK_DESTINATIONS_ON":
    case "INHERITS_FROM":
    case "CONSTRAINS_VALUES_ON": {
      return {
        ...outwardEdge,
        rightEndpoint: {
          baseId: outwardEdge.rightEndpoint.baseId as BaseUrl,
          revisionId: outwardEdge.rightEndpoint
            .revisionId as OntologyTypeVersion,
        },
      };
    }
    // Knowledge-graph edge-kind cases
    case "HAS_LEFT_ENTITY":
    case "HAS_RIGHT_ENTITY": {
      return {
        ...outwardEdge,
        rightEndpoint: {
          entityId: outwardEdge.rightEndpoint.entityId as EntityId,
          interval: {
            start: {
              kind: "inclusive",
              limit: outwardEdge.rightEndpoint.interval.start
                .limit as Timestamp,
            },
            end:
              outwardEdge.rightEndpoint.interval.end.kind === "unbounded"
                ? {
                    kind: "unbounded",
                  }
                : {
                    kind: "exclusive",
                    limit: outwardEdge.rightEndpoint.interval.end
                      .limit as Timestamp,
                  },
          },
        },
      };
    }
    // Shared edge-kind cases
    case "IS_OF_TYPE": {
      return outwardEdge.reversed
        ? {
            ...outwardEdge,
            reversed: outwardEdge.reversed,
            rightEndpoint: {
              entityId: (
                outwardEdge.rightEndpoint as EntityIdWithIntervalGraphApi
              ).entityId as EntityId,
              interval: {
                start: {
                  kind: "inclusive",
                  limit: (
                    outwardEdge.rightEndpoint as EntityIdWithIntervalGraphApi
                  ).interval.start.limit as Timestamp,
                },
                end:
                  (outwardEdge.rightEndpoint as EntityIdWithIntervalGraphApi)
                    .interval.end.kind === "unbounded"
                    ? {
                        kind: "unbounded",
                      }
                    : {
                        kind: "exclusive",
                        limit: (
                          (
                            outwardEdge.rightEndpoint as EntityIdWithIntervalGraphApi
                          ).interval.end as ExclusiveBoundGraphApi
                        ).limit as Timestamp,
                      },
              },
            },
          }
        : {
            ...outwardEdge,
            reversed: outwardEdge.reversed,
            rightEndpoint: {
              baseId: (
                outwardEdge.rightEndpoint as OntologyTypeVertexIdGraphApi
              ).baseId as BaseUrl,
              revisionId: (
                outwardEdge.rightEndpoint as OntologyTypeVertexIdGraphApi
              ).revisionId as OntologyTypeVersion,
            },
          };
    }
  }
};

export const mapEdges = (edges: EdgesGraphApi): Edges => {
  const mappedEdges: Edges = {};

  // Trying to build this with `Object.fromEntries` breaks tsc and leads to `any` typed values
  for (const [baseId, inner] of Object.entries(edges)) {
    const result = validateBaseUrl(baseId);
    if (result.type === "Ok") {
      // ------------ Ontology Type case ----------------
      const baseUrl = result.inner;

      mappedEdges[baseUrl] = Object.fromEntries(
        Object.entries(inner).map(([version, outwardEdges]) => {
          const versionNumber = Number(version);

          if (Number.isNaN(versionNumber)) {
            throw new Error(
              `Unrecognized ontology type version, expected a number but got: ${version}`,
            );
          }

          const mappedOutwardEdges = outwardEdges.map((outwardEdge) => {
            const mappedOutwardEdge = mapOutwardEdge(outwardEdge);
            if (isKnowledgeGraphOutwardEdge(mappedOutwardEdge)) {
              throw new Error(
                `Expected an ontology outward edge but found:\n${JSON.stringify(
                  outwardEdge,
                )}`,
              );
            }
            return mappedOutwardEdge;
          });

          return [versionNumber, mappedOutwardEdges];
        }),
      );
    } else if (isEntityId(baseId)) {
      // ------------ Entity (knowledge-graph) case ----------------
      mappedEdges[baseId] = Object.fromEntries(
        Object.entries(inner).map(([version, outwardEdges]) => {
          const timestamp = Date.parse(version);

          if (Number.isNaN(timestamp)) {
            throw new Error(
              `Unrecognized timestamp, expected an ISO-formatted timestamp but got: ${version}`,
            );
          }

          const mappedOutwardEdges = outwardEdges.map((outwardEdge) => {
            const mappedOutwardEdge = mapOutwardEdge(outwardEdge);
            if (isOntologyOutwardEdge(mappedOutwardEdge)) {
              throw new Error(
                `Expected an ontology outward edge but found:\n${JSON.stringify(
                  outwardEdge,
                )}`,
              );
            }
            return mappedOutwardEdge;
          });

          return [version, mappedOutwardEdges];
        }),
      );
    } else {
      throw new Error(
        `Unrecognized or invalid vertices outer key type: ${baseId}`,
      );
    }
  }

  return mappedEdges;
};
