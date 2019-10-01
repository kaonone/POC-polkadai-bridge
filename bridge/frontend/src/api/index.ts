import Web3 from "web3";
import Contract from "web3/eth/contract";
import { Observable, interval, from, fromEventPattern } from "rxjs";
import BN from "bn.js";
import { switchMap, skipWhile } from 'rxjs/operators';
import { ApiRx } from '@polkadot/api';
import { web3Enable, web3AccountsSubscribe, web3FromAddress } from '@polkadot/extension-dapp';
import { InjectedAccountWithMeta } from '@polkadot/extension-inject/types';
import { decodeAddress } from "@polkadot/util-crypto";
import { u8aToHex } from "@polkadot/util";

import { ETH_NETWORK_CONFIG, DEFAULT_DECIMALS } from "~/env";
import bridgeAbi from "~/abis/bridge.json";
import erc20Abi from "~/abis/erc20.json";
import { getContractData$ } from "~/util/getContractData$";
import { toBaseUnit } from '~util/toBaseUnit';
import { callPolkaApi } from './callPolkaApi';

export class Api {
  private _daiContract: Contract;
  private _bridgeContract: Contract;

  constructor(private _web3: Web3, private _substrateApi: Observable<ApiRx>) {
    this._daiContract = new this._web3.eth.Contract(
      erc20Abi,
      ETH_NETWORK_CONFIG.contracts.dai
    );
    this._bridgeContract = new this._web3.eth.Contract(
      bridgeAbi,
      ETH_NETWORK_CONFIG.contracts.bridge,
    );
  }

  public async sendToEthereum(from: string, to: string, amount: string): Promise<void> {
    const substrateApi = await this._substrateApi.toPromise();
    const substrateWeb3 = await web3FromAddress(from);
    substrateApi.setSigner(substrateWeb3.signer);

    const units = toBaseUnit(amount, DEFAULT_DECIMALS).toString();
    const transfer = substrateApi.tx.bridge.setTransfer(to, units);

    await new Promise((resolve, reject) => {
      transfer.signAndSend(from).subscribe({
        complete: resolve,
        error: reject,
        next: ({ isCompleted, isError }) => {
          isError && reject('tx.bridge.setTransfer extrinsic is failed');
          isCompleted && resolve();
        }
      });
    });
  }

  public async sendToSubstrate(from: string, to: string, amount: string): Promise<void> {
    const units = toBaseUnit(amount, DEFAULT_DECIMALS).toString();
    await this.approveBridge(from, units);
    await this.sendToBridge(from, to, units);
  }

  private async approveBridge(from: string, amount: string): Promise<void> {
    const allowance: string = await this._daiContract.methods.allowance(from, ETH_NETWORK_CONFIG.contracts.bridge).call();

    if (new BN(amount).lte(new BN(allowance))) {
      return;
    }

    await this._daiContract.methods.approve(ETH_NETWORK_CONFIG.contracts.bridge, amount).send({ from });
  }

  private async sendToBridge(from: string, to: string, amount: string): Promise<void> {
    const formatedToAddress = u8aToHex(decodeAddress(to));
    const bytesAddress = this._web3.utils.hexToBytes(formatedToAddress);
    await this._bridgeContract.methods.setTransfer(amount, bytesAddress).send({ from });
  }

  public getEthValidators$(): Observable<string[]> {
    return from([[
      '6a8357ae0173737209af59152ee30a786dbade70',
      '93880d6508e3ffee5a4376939d3322f2f11b56d1',
      '9194ad793e72052992f9a1b3b8eaef5463300f87',
    ]]);
    return getContractData$<string[], string[]>(this._bridgeContract, "validators", {
      eventsForReload: [
        ["ValidatorShipTransferred"],
      ],
    });
  }

  public getEthBalance$(_address: string): Observable<BN> {
    const address = _address.toLowerCase();

    return getContractData$<string, BN>(this._daiContract, "balanceOf", {
      args: [address],
      eventsForReload: [
        ["Transfer", { filter: { _from: address } }],
        ["Transfer", { filter: { _to: address } }]
      ],
      convert: value => new BN(value),
    });
  }

  public getSubstrateBalance$(_address: string): Observable<BN> {
    return callPolkaApi(this._substrateApi, 'query.token.balance', _address);
  }

  public getEthAccount$(): Observable<string | null> {
    return from(getEthAccount(this._web3)).pipe(
      skipWhile(account => !account),
      switchMap(() => interval(1000).pipe(
        switchMap(() => getEthAccount(this._web3)),
      )),
    );
  }

  public getSubstrateAccounts$(): Observable<InjectedAccountWithMeta[]> {
    return from(web3Enable('Akropolis Network Dapp')).pipe(
      switchMap((injectedExtensions) => injectedExtensions.length
        ? fromEventPattern<InjectedAccountWithMeta[]>(
          emitter => web3AccountsSubscribe(emitter),
          (_, signal: ReturnType<typeof web3AccountsSubscribe>) => signal.then(unsubscribe => unsubscribe()),
        )
        : new Observable<InjectedAccountWithMeta[]>(subscriber => subscriber.error(new Error('Injected extensions not found'))),
      )
    );
  }
}

async function getEthAccount(web3: Web3): Promise<string | null> {
  // Modern dapp browsers...
  if (window.ethereum) {
    try {
      // Request account access
      await window.ethereum.enable();
    } catch (error) {
      console.error('User denied account access');
      throw error;
    }
  }

  const accounts = await web3.eth.getAccounts();
  return accounts[0] || null;
}
