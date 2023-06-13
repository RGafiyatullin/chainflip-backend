// INSTRUCTIONS
//
// This command takes two arguments.
// It will fund the ethereum address provided as the first argument with the amount of
// tokens provided in the second argument. The token amount is interpreted in ETH
//
// For example: pnpm tsx ./commands/fund_eth.ts 0xcf1dc766fc2c62bef0b67a8de666c8e67acf35f6 1.2
// will send 1.2 ETH to account 0xcf1dc766fc2c62bef0b67a8de666c8e67acf35f6

import Web3 from 'web3';
import { runWithTimeout } from '../shared/utils';

async function main() {
  const eth_endpoint = process.env.ETH_ENDPOINT || "http://127.0.0.1:8545";
  const ethereumAddress = process.argv[2];
  const ethAmount = process.argv[3].trim();
  var weiAmount;
  if (ethAmount.indexOf('.') == -1) {
    weiAmount = ethAmount + "000000000000000000";
  } else {
    const amount_parts = ethAmount.split('.');
    weiAmount = amount_parts[0] + amount_parts[1].padEnd(18, '0').substr(0, 18);
  }
  const web3 = new Web3(eth_endpoint);
  const whaleKey = process.env.ETH_USDC_WHALE || '0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80';
  const whalePubKey = web3.eth.accounts.privateKeyToAccount(whaleKey).address;

  console.log("Whale pubkey: " + whalePubKey);

  let nonce = await web3.eth.getTransactionCount(whalePubKey);
  let txHash;
  let retries = 0;
  while (!txHash && retries < 50) {
    try {
      const tx = {
        to: ethereumAddress,
        value: weiAmount,
        gas: 2000000,
        nonce: nonce
      };
      console.log('Transferring ' + ethAmount + ' ETH to ' + ethereumAddress + ' with nonce ' + nonce);
      const signedTx = await web3.eth.accounts.signTransaction(tx, whaleKey);
      txHash = await web3.eth.sendSignedTransaction(signedTx.rawTransaction as string);
    } catch (error) {
      console.error(error);
      await new Promise(resolve => setTimeout(resolve, 5000));
      retries++;
      nonce++;
    }
  }
  process.exit(0);
}

runWithTimeout(main(), 1000000).catch((error) => {
  console.error(error);
  process.exit(-1);
});
