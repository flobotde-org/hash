import { useLazyQuery } from "@apollo/client";
import { mapGqlSubgraphFieldsFragmentToSubgraph } from "@local/hash-isomorphic-utils/graph-queries";
import { EntityTypeRootType } from "@local/hash-subgraph";
import { useCallback } from "react";

import {
  QueryEntityTypesQuery,
  QueryEntityTypesQueryVariables,
} from "../../../../graphql/api-types.gen";
import { queryEntityTypesQuery } from "../../../../graphql/queries/ontology/entity-type.queries";
import { QueryEntityTypesMessageCallback } from "./ontology-types-shim";

export const useBlockProtocolQueryEntityTypes = (): {
  queryEntityTypes: QueryEntityTypesMessageCallback;
} => {
  const [queryFn] = useLazyQuery<
    QueryEntityTypesQuery,
    QueryEntityTypesQueryVariables
  >(queryEntityTypesQuery, {
    fetchPolicy: "cache-and-network",
  });

  const queryEntityTypes = useCallback<QueryEntityTypesMessageCallback>(
    async ({ data }) => {
      if (!data) {
        return {
          errors: [
            {
              code: "INVALID_INPUT",
              message: "'data' must be provided for queryEntityTypes",
            },
          ],
        };
      }

      const { graphResolveDepths, latestOnly, includeArchived } = data;

      /**
       * @todo Add filtering to this query using structural querying.
       *   This may mean having the backend use structural querying and relaying
       *   or doing it from here.
       *   https://app.asana.com/0/1202805690238892/1202890614880643/f
       */
      const response = await queryFn({
        variables: {
          constrainsValuesOn: { outgoing: 255 },
          constrainsPropertiesOn: { outgoing: 255 },
          constrainsLinksOn: { outgoing: 1 },
          constrainsLinkDestinationsOn: { outgoing: 1 },
          inheritsFrom: { outgoing: 255 },
          ...graphResolveDepths,
          latestOnly,
          includeArchived,
        },
      });

      if (!response.data) {
        return {
          errors: [
            {
              code: "INVALID_INPUT",
              message: "Error calling queryEntityTypes",
            },
          ],
        };
      }

      /** @todo - Is there a way we can ergonomically encode this in the GraphQL type? */
      const subgraph =
        mapGqlSubgraphFieldsFragmentToSubgraph<EntityTypeRootType>(
          response.data.queryEntityTypes,
        );

      return { data: subgraph };
    },
    [queryFn],
  );

  return { queryEntityTypes };
};
