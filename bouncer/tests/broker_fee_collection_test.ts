#!/usr/bin/env -S pnpm tsx
import assert from 'assert';
import Keyring from '@polkadot/keyring';
import { Assets, assetDecimals } from '@chainflip-io/cli';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { amountToFineAmount, getChainflipApi, observeEvent, runWithTimeout } from '../shared/utils';
import { testSwap } from '../shared/swapping';
import { defaultCommissionBps } from '../shared/new_swap';

const testSwapAmount = 1000;
const expectedDepositGasFee = 3500;

async function main(): Promise<void> {
  console.log('\x1b[36m%s\x1b[0m', '=== Running broker fee collection test ===');

  await cryptoWaitReady();
  const keyring = new Keyring({ type: 'sr25519' });
  const broker1 = keyring.createFromUri('//BROKER_1');
  const chainflip = await getChainflipApi();

  // Check account role
  const role = JSON.stringify(
    await chainflip.query.accountRoles.accountRoles(broker1.address),
  ).replaceAll('"', '');
  console.log('role:', role);
  console.log('broker1.address:', broker1.address);
  assert.strictEqual(role, 'Broker', `Broker has unexpected role: ${role}`);

  // Check the broker fees before the swap
  const earnedBrokerFeesBefore = BigInt(
    (await chainflip.query.swapping.earnedBrokerFees(broker1.address, Assets.ETH)).toString(),
  );
  console.log('earnedBrokerFeesBefore:', earnedBrokerFeesBefore);

  // Run a swap
  const observeSwapScheduledEvent = observeEvent(':SwapScheduled', chainflip);
  await testSwap(
    Assets.ETH,
    Assets.FLIP,
    undefined,
    undefined,
    undefined,
    testSwapAmount.toString(),
  );

  // Check the broker fees after the swap
  const earnedBrokerFeesAfter = BigInt(
    (await chainflip.query.swapping.earnedBrokerFees(broker1.address, Assets.ETH)).toString(),
  );
  console.log('earnedBrokerFeesAfter:', earnedBrokerFeesAfter);

  // Get values from the swap event
  const swapScheduledEvent = await observeSwapScheduledEvent;
  const brokerCommission = BigInt(
    JSON.stringify(swapScheduledEvent.data.brokerCommission)
      .replaceAll('"', '')
      .replaceAll(',', ''),
  );
  console.log('brokerCommission:', brokerCommission);
  const depositAmount = BigInt(
    JSON.stringify(swapScheduledEvent.data.depositAmount).replaceAll('"', '').replaceAll(',', ''),
  );
  const increase = earnedBrokerFeesAfter - earnedBrokerFeesBefore;
  console.log('increase:', increase);
  const expectedIncrease = BigInt(
    amountToFineAmount(
      (testSwapAmount * (defaultCommissionBps / 10000)).toString(),
      assetDecimals.ETH,
    ),
  );

  // Check that the detected increase matches the swap event values and it is close to the expected amount (after the deposit gas fee is accounted for)
  assert.strictEqual(
    increase,
    brokerCommission,
    `Mismatch between brokerCommission from the swap event and the detected increase`,
  );
  assert.strictEqual(
    increase,
    depositAmount / BigInt(1 / (defaultCommissionBps / 10000)),
    `Mismatch between depositAmount from the swap event and the detected increase from broker commission`,
  );
  assert(
    increase >= expectedIncrease - BigInt(expectedDepositGasFee) && increase <= expectedIncrease,
    `Incorrect broker fees earned from swap, expected between ${
      expectedIncrease - BigInt(expectedDepositGasFee)
    } and ${expectedIncrease}, got ${increase}. Did gas fees change?`,
  );

  process.exit(0);
}

runWithTimeout(main(), 300000).catch((error) => {
  console.error(error);
  process.exit(-1);
});
