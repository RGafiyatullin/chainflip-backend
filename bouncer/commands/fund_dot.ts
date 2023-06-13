// INSTRUCTIONS
//
// This command takes two arguments.
// It will fund the polkadot address provided as the first argument with the amount of
// tokens provided in the second argument. The token amount is interpreted in DOT.
//
// For example: pnpm tsx ./commands/fund_dot.ts 12QTpTMELPfdz2xr9AeeavstY8uMcpUqeKWDWiwarskk4hSB 1.2
// will send 1.2 DOT to account 12QTpTMELPfdz2xr9AeeavstY8uMcpUqeKWDWiwarskk4hSB

import { ApiPromise, WsProvider } from '@polkadot/api';
import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { runWithTimeout } from '../shared/utils';

async function main() {
  const polkadot_endpoint = process.env.POLKADOT_ENDPOINT || 'ws://127.0.0.1:9945';
  const polkadotAddress = process.argv[2];
  const dotAmount = process.argv[3].trim();

  let planckAmount;
  if (!dotAmount.includes('.')) {
    planckAmount = dotAmount + '0000000000';
  } else {
    const amount_parts = dotAmount.split('.');
    planckAmount = amount_parts[0] + amount_parts[1].padEnd(10, '0').substr(0, 10);
  }
  await cryptoWaitReady();
  const keyring = new Keyring({ type: 'sr25519' });
  const alice = keyring.createFromUri('//Alice');
  const polkadot = await ApiPromise.create({ provider: new WsProvider(polkadot_endpoint), noInitWarn: true });

  let response = await polkadot.query.system.account(alice.address);
  let nonce = JSON.parse(JSON.stringify(response)).nonce;
  console.log(`Current nonce: ${nonce}`);
  let retries = 0;
  const maxRetries = 20;
  while (retries < maxRetries) {
    try {
      const transfer = polkadot.tx.balances.transfer(polkadotAddress, parseInt(planckAmount));
      console.log(`Transaction hash: ${txHash}`);
      process.exit(0);
    } catch (error) {
      console.error(error);
      if (error.message.includes('1010: Invalid Transaction:')) {
        nonce++;
        retries++;
        console.log(`Retrying with nonce ${nonce} in 5 seconds...`);
        await new Promise((resolve) => setTimeout(resolve, 5000));
      } else {
        process.exit(-1);
      }
    }
  }
  console.error(`Maximum retries (${maxRetries}) reached.`);
  process.exit(-1);
}

runWithTimeout(main(), 20000).catch((error) => {
  console.error(error);
  process.exit(-1);
});
