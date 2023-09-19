#!/usr/bin/env -S pnpm tsx
// INSTRUCTIONS
//
// This command takes no arguments.
// It will setup pools and zero to infinity range orders for all currencies
// For example: ./commands/setup_swaps.ts

import { cryptoWaitReady } from '@polkadot/util-crypto';
import { Asset } from '@chainflip-io/cli';
import { getChainflipApi, runWithTimeout } from '../shared/utils';
import { createLpPool } from '../shared/create_lp_pool';
import { provideLiquidity } from '../shared/provide_liquidity';
import { rangeOrder } from '../shared/range_order';
import { submitGovernanceExtrinsic } from '../shared/cf_governance';

const deposits = new Map<Asset, number>([
  ['DOT', 10000],
  ['ETH', 100],
  ['BTC', 10],
  ['USDC', 1000000],
  ['FLIP', 10000],
]);

const price = new Map<Asset, number>([
  ['DOT', 10],
  ['ETH', 1000],
  ['BTC', 10000],
  ['USDC', 1],
  ['FLIP', 10],
]);

async function main(): Promise<void> {
  await cryptoWaitReady();

  await Promise.all([
    createLpPool('ETH', price.get('ETH')!),
    createLpPool('DOT', price.get('DOT')!),
    createLpPool('BTC', price.get('BTC')!),
    createLpPool('FLIP', price.get('FLIP')!),
  ]);

  await Promise.all([
    provideLiquidity('USDC', deposits.get('USDC')!),
    provideLiquidity('ETH', deposits.get('ETH')!),
    provideLiquidity('DOT', deposits.get('DOT')!),
    provideLiquidity('BTC', deposits.get('BTC')!),
    provideLiquidity('FLIP', deposits.get('FLIP')!),
  ]);

  await Promise.all([
    rangeOrder('ETH', deposits.get('ETH')! * 0.9999),
    rangeOrder('DOT', deposits.get('DOT')! * 0.9999),
    rangeOrder('BTC', deposits.get('BTC')! * 0.9999),
    rangeOrder('FLIP', deposits.get('FLIP')! * 0.9999),
  ]);
  const chainflip = await getChainflipApi();

  const updateBuyInterval = chainflip.tx.liquidityPools.updateBuyInterval(5);
  await submitGovernanceExtrinsic(updateBuyInterval);

  const updateSupplyUpdateInterval = chainflip.tx.emissions.updateSupplyUpdateInterval(20);
  await submitGovernanceExtrinsic(updateSupplyUpdateInterval);

  console.log('=== Swaps Setup completed ===');

  process.exit(0);
}

runWithTimeout(main(), 2400000).catch((error) => {
  console.error(error);
  process.exit(-1);
});
