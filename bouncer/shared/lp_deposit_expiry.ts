import { Keyring } from '@polkadot/keyring';
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { observeEvent, getChainflipApi, lpMutex } from '../shared/utils';
import { sendBtc } from '../shared/send_btc';
import { submitGovernanceExtrinsic } from '../shared/cf_governance';

export async function testLpDepositExpiry() {
  await cryptoWaitReady();
  const keyring = new Keyring({ type: 'sr25519' });
  const lpUri = process.env.LP_URI ?? '//LP_1';
  const lp = keyring.createFromUri(lpUri);

  const chainflip = await getChainflipApi();

  console.log('=== Testing expiry of funded LP deposit address ===');
  const originalExpiryTime = Number(await chainflip.query.liquidityProvider.lpTTL());
  console.log('Setting expiry time for LP addresses to 10 blocks');

  await submitGovernanceExtrinsic(chainflip.tx.liquidityProvider.setLpTtl(10));
  await observeEvent('liquidityProvider:LpTtlSet', chainflip);

  console.log('Requesting new BTC LP deposit address');
  lpMutex.runExclusive(async () => {
    await chainflip.tx.liquidityProvider
      .requestLiquidityDepositAddress('Btc')
      .signAndSend(lp, { nonce: -1 });
  });

  const depositEventResult = await observeEvent(
    'liquidityProvider:LiquidityDepositAddressReady',
    chainflip,
  );
  const ingressAddress = depositEventResult.data.depositAddress.Btc;

  console.log('Funding BTC LP deposit address of ' + ingressAddress + ' with 1 BTC');

  await sendBtc(ingressAddress, 1);
  await observeEvent('liquidityProvider:LiquidityDepositAddressExpired', chainflip);

  console.log('Restoring expiry time for LP addresses to ' + originalExpiryTime + ' blocks');
  await submitGovernanceExtrinsic(chainflip.tx.liquidityProvider.setLpTtl(originalExpiryTime));

  await observeEvent('liquidityProvider:LpTtlSet', chainflip);

  console.log('=== LP deposit expiry test complete ===');
}
