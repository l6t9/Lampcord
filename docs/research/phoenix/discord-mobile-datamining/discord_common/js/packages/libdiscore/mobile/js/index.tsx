// discord_common/js/packages/libdiscore/mobile/js/index.tsx
import _mod17 from "../../../../../../_runtime/metro/00017__.js";
import global_types from "global_types.tsx";
import clock from "clock.tsx";
import size from "../../../../../../_runtime/metro/00002__.js";

const NativeModules = _mod17.NativeModules;
if (NativeModules.LibDiscoreModule) {
  let LibDiscoreModule = NativeModules.LibDiscoreModule;
} else {
  const _Proxy = Proxy;
  const obj = {
    get() {
      const error = new Error("The package 'react-native-libdiscore-jsi-module' doesn't seem to be linked");
      throw error;
    },
  };
  LibDiscoreModule = new Proxy({}, obj);
}
LibDiscoreModule.bridgeJSIFunctions();
const LIBDISCORE_JSI = global_types.typedGlobal.LIBDISCORE_JSI;
let result = size.fileFinishedImporting("../discord_common/js/packages/libdiscore/mobile/js/index.tsx");
const prototype = function BlockedDomainsStore() {
  return Object.create(new.target.prototype);
}.prototype;
prototype["isBlockedDomain"] = function isBlockedDomain(arg0) {
  return LIBDISCORE_JSI.isBlockedDomain(arg0);
};
prototype["startFetchingBlockedDomains"] = function startFetchingBlockedDomains(arg0) {
  const result = LIBDISCORE_JSI.startFetchingBlockedDomains(arg0);
};

export const ExperimentCacher = LIBDISCORE_JSI.ExperimentCacher;
export function rustMultiply(arg0, arg1) {
  return LIBDISCORE_JSI.rustMultiply(arg0, arg1);
}
export function consumeLogs() {
  return LIBDISCORE_JSI.consumeLogs();
}
export const monotonicNowMs = clock.monotonicNowMs;
export const BlockedDomainsStore = prototype;
export function getFluxApi() {
  return LIBDISCORE_JSI.fluxApi;
}
export function crash() {
  LIBDISCORE_JSI.crash();
}
export function registerDevLogListener(arg0) {
  const result = LIBDISCORE_JSI.registerDevLogListener(arg0);
}
export function generateLaunchSignature(globalObject) {
  return LIBDISCORE_JSI.generateLaunchSignature(globalObject);
}
export function getHttpClientAPI() {
  return {
    httpRequest: LIBDISCORE_JSI.httpRequest,
    getHttpRequestStatus: LIBDISCORE_JSI.getHttpRequestStatus,
    cancelHttpRequest: LIBDISCORE_JSI.cancelHttpRequest,
    getTrackedRequestCount: LIBDISCORE_JSI.getTrackedRequestCount,
  };
}
