import BN from 'bn.js';
import { O } from 'ts-toolbelt';
import { GenericAccountId, u64 } from '@polkadot/types';

// [Endpoint]: [Request, ConvertedRequestForApi, ApiResponse, ConvertedResponse]
interface ISignatures {
  'query.token.balance': [string, GenericAccountId, u64, BN];
}

export type Endpoint = keyof ISignatures;
export type EndpointWithRequest = keyof O.Filter<ISignatures, [null, ...any[]], 'implements->'>;
export type EndpointWithoutRequest = Exclude<Endpoint, EndpointWithRequest>;

export type Request<E extends Endpoint> = ISignatures[E][0];
export type ConvertedRequestForApi<E extends Endpoint> = ISignatures[E][1];
export type ApiResponse<E extends Endpoint> = ISignatures[E][2];
export type ConvertedResponse<E extends Endpoint> = ISignatures[E][3];

// tslint:disable-next-line: no-empty-interface
export interface IOption<E extends Endpoint> {
  defaultValue?: ConvertedResponse<E>;
  isSuspendedCall?: boolean;
  getCacheKey?(endpoint: E, args?: Request<E>): string;
}

export interface IOptionWithRequest<E extends Endpoint> extends IOption<E> {
  args: Request<E>;
}

export interface ICallMeta {
  error: string | null;
  loaded: boolean;
  updatedAt: number;
}

export type ICallResult<E extends Endpoint> = [ConvertedResponse<E> | null, ICallMeta];

export type ToRequestConverter<E extends Endpoint> = (request: Request<E>) => ConvertedRequestForApi<E>;
export type ToRequestConverters = {
  [E in EndpointWithRequest]: ToRequestConverter<E>;
};

export type FromResponseConverters = {
  [E in Endpoint]: (response: ApiResponse<E>) => ConvertedResponse<E>;
};

/**** CHAIN TYPES ****/
