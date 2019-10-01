import { checkAddress } from '@polkadot/util-crypto';
import { SUBSTRATE_DEFAULT_ADDRESS_PREFIX } from '~env';

export function validateSubstrateAddress(value: string): string | undefined {
  try {
    const [isValid, error] = checkAddress(value, SUBSTRATE_DEFAULT_ADDRESS_PREFIX);
    if (!isValid && error) {
      throw new Error(error);
    }
    return undefined;
  } catch (error) {
    return 'Enter a valid Substrate address';
  }
}
