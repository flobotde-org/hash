import { Effect, Layer, Ref } from "effect";
import { GenericTag } from "effect/Context";

import { createProto } from "../utils.js";
import * as RequestId from "./models/request/RequestId.js";

const TypeId: unique symbol = Symbol(
  "@local/harpc-client/wire-protocol/RequestIdProducer",
);
export type TypeId = typeof TypeId;

export interface RequestIdProducer {
  readonly [TypeId]: TypeId;
}

interface RequestIdProducerImpl extends RequestIdProducer {
  next: Ref.Ref<number>;
}

const RequestIdProducerProto: Omit<RequestIdProducer, "value"> = {
  [TypeId]: TypeId,
};

export const RequestIdProducer = GenericTag<RequestIdProducer>(
  TypeId.description!,
);

const make = () =>
  Effect.gen(function* () {
    const next = yield* Ref.make(0);

    return createProto(RequestIdProducerProto, { next }) as RequestIdProducer;
  });

export const layer = Layer.effect(RequestIdProducer, make());

export const next = (producer: RequestIdProducer) =>
  Effect.gen(function* () {
    const impl = producer as RequestIdProducerImpl;

    const id = yield* Ref.getAndUpdate(impl.next, (value) =>
      value === RequestId.MAX_VALUE ? RequestId.MIN_VALUE : value + 1,
    );

    return RequestId.makeUnchecked(id);
  });
