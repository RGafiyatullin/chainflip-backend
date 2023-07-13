// INSTRUCTIONS
//
// This command takes four arguments.
// It will request a new swap with the provided parameters
// Argument 1 is the source currency ("btc", "eth", "dot" or "usdc")
// Argument 2 is the destination currency ("btc", "eth", "dot" or "usdc")
// Argument 3 is the destination address
// Argument 4 is the broker fee in basis points
// For example: pnpm tsx ./commands/new_swap.ts dot btc n1ocq2FF95qopwbEsjUTy3ZrawwXDJ6UsX 100

import { runWithTimeout } from '../shared/utils';
import { Asset } from '@chainflip-io/cli/.';
import { newSwap } from '../shared/new_swap';

async function newSwapCommand() {
  const sourceToken = process.argv[2].toUpperCase() as Asset;
  const destToken = process.argv[3].toUpperCase() as Asset;
  const destAddress = process.argv[4];
  const fee = process.argv[5];

  console.log(`Requesting swap ${sourceToken} -> ${destToken}`);

  await newSwap(sourceToken, destToken, destAddress, fee);

  process.exit(0);
}

runWithTimeout(newSwapCommand(), 60000).catch((error) => {
  console.error(error);
  process.exit(-1);
});
