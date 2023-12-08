#!/usr/bin/env -S pnpm tsx
// INSTRUCTIONS
// Updates the polkadot network to a new spec version via a runtime update with no other changes to the code.
// The new spec version will be +1 from the current polkadot spec version.
// The first time the script is run, it will be faster because it uses a precompiled runtime.
// Subsequent runs will be slower because it needs to compile the runtime.

import {
  bumpAndBuildPolkadotRuntime,
  pushPolkadotRuntimeUpgrade,
} from '../shared/polkadot_runtime_update';
import { runWithTimeout } from '../shared/utils';
import { getCurrentRuntimeVersion } from '../shared/utils/bump_spec_version';

const POLKADOT_ENDPOINT_PORT = 9947;

async function main(): Promise<void> {
  // Bump the spec version
  const [wasmPath, expectedSpecVersion] = await bumpAndBuildPolkadotRuntime();

  // Submit the runtime upgrade
  await pushPolkadotRuntimeUpgrade(wasmPath);

  // Check the polkadot spec version has changed
  const postUpgradeSpecVersion = await getCurrentRuntimeVersion(POLKADOT_ENDPOINT_PORT);
  if (postUpgradeSpecVersion.specVersion !== expectedSpecVersion) {
    throw new Error(
      `Polkadot runtime upgrade failed. Currently at version ${postUpgradeSpecVersion.specVersion}, expected to be at ${expectedSpecVersion}`,
    );
  }

  process.exit(0);
}

runWithTimeout(main(), 400000).catch((error) => {
  console.error(error);
  process.exit(-1);
});
