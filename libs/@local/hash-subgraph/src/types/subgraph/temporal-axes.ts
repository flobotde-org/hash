import type {
  PinnedTemporalAxis,
  PinnedTemporalAxisUnresolved,
  QueryTemporalAxes as QueryTemporalAxesBp,
  QueryTemporalAxesUnresolved as QueryTemporalAxesUnresolvedBp,
  SubgraphTemporalAxes as SubgraphTemporalAxesBp,
  VariableTemporalAxis,
  VariableTemporalAxisUnresolved,
} from "@blockprotocol/graph";
import type { Subtype } from "@local/advanced-types/subtype";

/**
 * Defines the two possible combinations of pinned/variable temporal axes that are used in queries that return
 * {@link Subgraph}`s.
 *
 * The {@link VariableTemporalAxisUnresolved} is optionally bounded, in the absence of provided bounds an inclusive
 * bound at the timestamp at point of resolving is assumed.
 */
export type QueryTemporalAxesUnresolved = Subtype<
  QueryTemporalAxesUnresolvedBp,
  | {
      variable: VariableTemporalAxisUnresolved<"decisionTime">;
      pinned: PinnedTemporalAxisUnresolved<"transactionTime">;
    }
  | {
      variable: VariableTemporalAxisUnresolved<"transactionTime">;
      pinned: PinnedTemporalAxisUnresolved<"decisionTime">;
    }
>;

/**
 * Defines the two possible combinations of pinned/variable temporal axes that are used in responses to queries that
 * return {@link Subgraph}`s.
 *
 * The {@link VariableTemporalAxis} is bounded according to the input of the query.
 */
export type QueryTemporalAxes = Subtype<
  QueryTemporalAxesBp,
  | {
      variable: VariableTemporalAxis<"decisionTime">;
      pinned: PinnedTemporalAxis<"transactionTime">;
    }
  | {
      variable: VariableTemporalAxis<"transactionTime">;
      pinned: PinnedTemporalAxis<"decisionTime">;
    }
>;

/**
 * Denotes the temporal axes used in constructing the {@link Subgraph}.
 */
export type SubgraphTemporalAxes = Subtype<
  SubgraphTemporalAxesBp,
  {
    /**
     * The {@link QueryTemporalAxesUnresolved} provided in the query
     */
    initial: QueryTemporalAxesUnresolved;
    /**
     * The {@link QueryTemporalAxes} used when resolving the {@link Subgraph}
     */
    resolved: QueryTemporalAxes;
  }
>;
