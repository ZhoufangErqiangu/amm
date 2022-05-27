import { SolletWalletAdapter } from "@solana/wallet-adapter-sollet";
import store from "@/store";

export const wallet = new SolletWalletAdapter({ network: "devnet" });

export async function connectWallet() {
  await wallet.connect();
  store.commit("connect", wallet);
  return wallet.connected;
}

export async function disconnectWallet() {
  await wallet.disconnect();
  store.commit("discontent", wallet);
  return wallet.connected;
}
