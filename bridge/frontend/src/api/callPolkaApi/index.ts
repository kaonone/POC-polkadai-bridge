import { Observable, BehaviorSubject } from 'rxjs';
import { switchMap, map } from 'rxjs/operators';
import { Codec } from '@polkadot/types/types';
import { ApiRx } from '@polkadot/api';

import {
  EndpointWithoutRequest, EndpointWithRequest, Endpoint, Request, ConvertedResponse,
} from './types';
import { fromResponseConverters } from './fromResponse';
import { toRequestConverters } from './toRequest';

function callPolkaApi<E extends EndpointWithoutRequest>(
  substrateApi: Observable<ApiRx>,
  endpoint: E,
): Observable<ConvertedResponse<E>>;
function callPolkaApi<E extends EndpointWithRequest>(
  substrateApi: Observable<ApiRx>,
  endpoint: E,
  args: Request<E>,
): Observable<ConvertedResponse<E>>;
function callPolkaApi<E extends Endpoint>(
  substrateApi: Observable<ApiRx>,
  endpoint: E,
  args?: Request<E>,
): Observable<ConvertedResponse<E>>;
function callPolkaApi<E extends Endpoint>(
  substrateApi: Observable<ApiRx>,
  endpoint: E,
  args?: Request<E>,
): Observable<ConvertedResponse<E>> {
  return substrateApi.pipe(switchMap(api => {
    const [area, section, method] = endpoint.split('.');
    if (!isArea(area)) {
      throw new Error(`Unknown api.${area}, expected ${availableAreas.join(', ')}`);
    }

    const toRequestConverter =
      toRequestConverters[endpoint as EndpointWithRequest] || null;
    const convertedArgs = args && toRequestConverter ? toRequestConverter(args as Request<EndpointWithRequest>) : [];
    const argsForRequest = Array.isArray(convertedArgs) ? convertedArgs : [convertedArgs];

    let response: Observable<Codec>;
    if (area === 'consts') {
      const apiResponse = api.consts[section] && api.consts[section][method];
      if (!apiResponse) {
        throw new Error(`Unable to find api.${area}.${section}.${method}`);
      }
      response = new BehaviorSubject(apiResponse);
    } else {
      const apiMethod = api[(area as 'query')][section] && api[(area as 'query')][section][method];
      if (!apiMethod) {
        throw new Error(`Unable to find api.${area}.${section}.${method}`);
      }
      response = apiMethod(...argsForRequest);
    }

    return response.pipe(
      map(value => fromResponseConverters[endpoint](value as any)),
    );
  }));
}

const availableAreas = ['consts', 'rpc', 'query', 'derive'] as const;
type Area = (typeof availableAreas)[number];

function isArea(value: string): value is Area {
  return (availableAreas as readonly string[]).includes(value);
}

export { callPolkaApi };
