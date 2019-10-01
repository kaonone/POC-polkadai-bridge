import { Observable, from, merge, empty } from "rxjs";
import { skipUntil, mergeMap, throttleTime } from "rxjs/operators";
import Contract from "web3/eth/contract";
import { BlockType } from "web3/eth/types";
import { EventEmitter } from "web3/types";

import { fromWeb3Event } from "~/util/fromWeb3Event";

interface ISubscribeEventOptions {
  filter?: object;
  fromBlock?: BlockType;
  topics?: string[];
}

interface IOptions<IV, RV> {
  eventsForReload?: "none" | "all" | Array<[string, ISubscribeEventOptions?]>;
  reloadTrigger$?: Observable<any>;
  args?: Array<string | number>;
  convert?(value: IV): RV;
}

function identity(value: any) {
  return value;
}

export function getContractData$<IV, RV>(
  contract: Contract,
  method: string,
  options: IOptions<IV, RV> = {}
): Observable<RV> {
  const {
    eventsForReload = "all",
    reloadTrigger$ = empty(),
    args = [],
    convert = identity
  } = options;

  const load = async () => {
    const data = await contract.methods[method](...args).call();
    return convert(data);
  };

  const emitters = [
    eventsForReload === "all" ? contract.events.allEvents() : null
  ]
    .concat(
      Array.isArray(eventsForReload)
        ? eventsForReload.map(([event, filterOptions]) =>
            contract.events[event](filterOptions)
          )
        : []
    )
    .filter((value): value is EventEmitter => Boolean(value));

  const first$ = from(load());
  const fromEvents$ = merge(
    ...emitters.map(emitter => fromWeb3Event(emitter, "data"))
  ).pipe(
    skipUntil(first$),
    throttleTime(200),
    mergeMap(() => from(load()), 1)
  );

  return merge(first$, fromEvents$, reloadTrigger$);
}
