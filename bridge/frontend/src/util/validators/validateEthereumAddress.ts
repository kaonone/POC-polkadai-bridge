import Web3 from 'web3';

export function validateEthereumAddress(value: string): string | undefined {
  return value && Web3.utils.isAddress(value.toLowerCase()) ? undefined : 'Enter a valid Ethereum address';
}