// discord_common/js/packages/libdiscore/js_shim/js/shim.native.tsx
import LIBDISCORE_JSI from "../../mobile/js/index.tsx";
import initLibdiscore from "initLibdiscore.native.tsx";
import size from "../../../../../../_runtime/metro/00002__.js";

let result = size.fileFinishedImporting("../discord_common/js/packages/libdiscore/js_shim/js/shim.native.tsx");

export function isBlockedDomain(arg0) {
  const BlockedDomainsStore = LIBDISCORE_JSI.BlockedDomainsStore;
  return BlockedDomainsStore.isBlockedDomain(arg0);
}
export function startFetchingBlockedDomains(combined) {
  const BlockedDomainsStore = LIBDISCORE_JSI.BlockedDomainsStore;
  const result = BlockedDomainsStore.startFetchingBlockedDomains(combined);
}
export function consumeLogs() {
  return LIBDISCORE_JSI.consumeLogs();
}
export function isUnsupportedBrowser() {
  return false;
}
export function getExperimentCacher() {
  return LIBDISCORE_JSI.ExperimentCacher;
}
export function getHttpClientAPI() {
  return LIBDISCORE_JSI.getHttpClientAPI();
}
export const rustMultiply = LIBDISCORE_JSI.rustMultiply;
export const crash = LIBDISCORE_JSI.crash;
export const generateLaunchSignature = LIBDISCORE_JSI.generateLaunchSignature;
export const getFluxApi = LIBDISCORE_JSI.getFluxApi;
export const isLibdiscoreInitialized = initLibdiscore.isLibdiscoreInitialized;
