import { ToRequestConverters } from './types';
import { GenericAccountId } from '@polkadot/types';

export const toRequestConverters: ToRequestConverters = {
  'query.token.balance': address => new GenericAccountId(address),
};
