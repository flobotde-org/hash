import {
  type FastCheck,
  Data,
  Effect,
  Either,
  Equal,
  Hash,
  Inspectable,
  pipe,
  Pipeable,
  Predicate,
} from "effect";

import { U16_MAX, U16_MIN } from "../constants.js";
import { createProto, implDecode, implEncode } from "../utils.js";
import { MutableBuffer } from "../binary/index.js";

const TypeId: unique symbol = Symbol(
  "@local/harpc-client/wire-protocol/types/ProcedureId",
);

export type TypeId = typeof TypeId;

export class ProcedureIdTooLargeError extends Data.TaggedError(
  "ProcedureIdTooLargeError",
)<{ received: number }> {
  get message() {
    return `Procedure ID too large: ${this.received}, expected between ${U16_MIN} and ${U16_MAX}`;
  }
}

export class ProcedureIdTooSmallError extends Data.TaggedError(
  "ProcedureIdTooSmallError",
)<{ received: number }> {
  get message() {
    return `Procedure ID too small: ${this.received}, expected between ${U16_MIN} and ${U16_MAX}`;
  }
}

export interface ProcedureId
  extends Equal.Equal,
    Inspectable.Inspectable,
    Pipeable.Pipeable {
  readonly [TypeId]: TypeId;

  readonly value: number;
}

const ProcedureIdProto: Omit<ProcedureId, "value"> = {
  [TypeId]: TypeId,

  [Equal.symbol](this: ProcedureId, that: Equal.Equal) {
    return (
      // eslint-disable-next-line @typescript-eslint/no-use-before-define
      isProcedureId(that) && Equal.equals(this.value, that.value)
    );
  },

  [Hash.symbol](this: ProcedureId) {
    return pipe(
      Hash.hash(this[TypeId]),
      Hash.combine(Hash.hash(this.value)),
      Hash.cached(this),
    );
  },

  toString(this: ProcedureId) {
    return `ProcedureId(${this.value.toString()})`;
  },

  toJSON(this: ProcedureId) {
    return {
      _id: "ProcedureId",
      id: this.value,
    };
  },

  [Inspectable.NodeInspectSymbol]() {
    return this.toJSON();
  },

  pipe() {
    // eslint-disable-next-line prefer-rest-params
    return Pipeable.pipeArguments(this, arguments);
  },
};

/** @internal */
export const makeUnchecked = (value: number): ProcedureId =>
  createProto(ProcedureIdProto, { value });

export const make = (
  id: number,
): Effect.Effect<
  ProcedureId,
  ProcedureIdTooSmallError | ProcedureIdTooLargeError
> => {
  if (id < U16_MIN) {
    return Effect.fail(new ProcedureIdTooSmallError({ received: id }));
  }
  if (id > U16_MAX) {
    return Effect.fail(new ProcedureIdTooLargeError({ received: id }));
  }

  return Effect.succeed(makeUnchecked(id));
};

export type EncodeError = Effect.Effect.Error<ReturnType<typeof encode>>;

export const encode = implEncode((buffer, procedureId: ProcedureId) =>
  MutableBuffer.putU16(buffer, procedureId.value),
);

export type DecodeError = Effect.Effect.Error<ReturnType<typeof decode>>;

export const decode = implDecode((buffer) =>
  MutableBuffer.getU16(buffer).pipe(Either.map(makeUnchecked)),
);

export const isProcedureId = (value: unknown): value is ProcedureId =>
  Predicate.hasProperty(value, TypeId);

export const isReserved = (value: ProcedureId) =>
  (value.value & 0xf0_00) === 0xf0_00;

export const arbitrary = (fc: typeof FastCheck) =>
  fc.integer({ min: U16_MIN, max: U16_MAX }).map(makeUnchecked);
