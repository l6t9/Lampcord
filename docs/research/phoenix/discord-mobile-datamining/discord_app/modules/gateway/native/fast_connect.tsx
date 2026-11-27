// discord_app/modules/gateway/native/fast_connect.tsx
import LoggerDefault from "../../debug/Logger.tsx";
import TTITrackerDefault from "../../tti_analytics/TTITracker.tsx";
import AppStartPerformanceDefault from "../../../../discord_common/js/packages/app-start-performance/AppStartPerformance.tsx";
import _mod17 from "../../../../_runtime/metro/00017__.js";
import KvCacheVersionConstants from "../../app_database/modules/KvCacheVersionConstants.tsx";
import QosToken from "../qos/QosToken.tsx";
import discord_common_AnalyticsUtilsAll from "../../../../discord_common/js/packages/analytics-utils/AnalyticsUtils.tsx";
import isCacheEnabled from "../../cache/isCacheEnabled.native.tsx";
import RequestGatewaySocketAll from "../RequestGatewaySocket.tsx";
import GatewayCapabilities from "../GatewayCapabilities.tsx";
import NativeFastConnectModuleDefault from "../../../../discord_common/js/packages/rtn-codegen/js/NativeFastConnectModule.tsx";
import checkEnv from "../../../utils/checkEnv.tsx";
import size from "../../../../_runtime/metro/00002__.js";

function createFastConnectSocket() {
  if (null != window.WebSocket) {
    let obj = obj3(14910);
    constants = obj.getConstants();
    let flag;
    if (constants != null) {
      flag = constants.supportsZstd;
    }
    if (flag == null) {
      flag = false;
    }
    let str2 = "zlib-stream";
    if (flag) {
      str2 = "zstd-stream";
    }
    const _window = window;
    const _window2 = window;
    const _HermesInternal = HermesInternal;
    const combined =
      "" +
      window.GLOBAL_ENV.GATEWAY_ENDPOINT +
      "/?encoding=json&v=" +
      window.GLOBAL_ENV.API_VERSION +
      "&compress=" +
      str2;
    obj.log(`[FAST CONNECT] ${tmp7}`);
    const _Date = Date;
    closure_0 = Date.now();
    const tmp9 = obj3(14897)(combined);
    const _parseFloat = parseFloat;
    const parsed = parseFloat(tmp9._socketId);
    const _isNaN = isNaN;
    if (isNaN(parsed)) {
      obj2.log("[FAST CONNECT] Unable to create socketId from NaN value ", tmp9._socketId);
    } else {
      const tmp3Result = tmp3(14910);
      if (flag) {
        const result = tmp3Result.enableZstdStreamSupport(parsed);
      } else {
        const result1 = tmp3Result.enableZlibStreamSupport(parsed);
      }
      obj3 = { open: false, gateway: combined, identify: false, messages: [], clientState: null, userId: null };
      tmp9.onopen = () => {
        const obj = AppStartPerformanceDefault;
        obj.mark("\u{1F310}", "Fastconnect socket opened");
        obj.log("connected and identified in " + Date.now() - closure_0 + "ms didIdentify:" + obj3.identify);
        obj3.open = true;
      };
      const fn = () => {
        obj3(dependencyMap[12]).mark("\u{1F310}", "Fastconnect socket close");
        window._ws = null;
      };
      tmp9.onerror = fn;
      tmp9.onclose = fn;
      tmp9.onmessage = (arg0) => {
        AppStartPerformanceDefault.mark("\u{1F310}", "Fastconnect socket message");
        const messages = obj3.messages;
        messages.push(arg0);
      };
      const _window3 = window;
      const obj4 = { ws: tmp9, state: obj3 };
      window._ws = obj4;
      tmp3(10).mark("\u{1F310}", "Fastconnect socket created");
      const tmp3Result2 = tmp3(10);
    }
    obj2 = obj;
  } else {
    obj.log("Skipping fast_connect because `window.WebSocket` does not exist.");
  }
}
const AppState = _mod17.AppState;
let closure_5 = KvCacheVersionConstants.VERSION_TO_FORCE_RESYNCING_ALL_DATA;
let d = new LoggerDefault("FAST CONNECT");
d.info("initial app state (import time)", AppState.currentState);
let c7 = null;
const fastConnectSocket = createFastConnectSocket();
let result = size.fileFinishedImporting("modules/gateway/native/fast_connect.tsx");

export { createFastConnectSocket };
export function closeFastConnectSocket() {
  if (null != window._ws) {
    const _window = window;
    ws.close();
    const _window2 = window;
    window._ws = null;
    c7 = null;
  }
}
export function getLastFastConnectIdentifyUserId() {
  return c7;
}
export function identifyWebSocket() {
  if (null != window._ws) {
    const beginFastConnect = TTITrackerDefault.beginFastConnect;
    beginFastConnect.measure(() => {
      const loadFastConnectNativeModule = TTITrackerDefault.loadFastConnectNativeModule;
      const measureResult = loadFastConnectNativeModule.measure(() => closure_1_1(closure_1_3[8]).getConstants());
      let token = measureResult.token;
      if (token == null) {
        token = null;
      }
      if (null != token) {
        if ("" !== token) {
          const _window = window;
          state = _ws.state;
          let tmp7 = str2;
          if (measureResult.userId == null) {
            tmp7 = null;
          }
          c7 = tmp7;
          let tmp8 = str2;
          if (measureResult.userId == null) {
            tmp8 = null;
          }
          state.userId = tmp8;
          let derivedQosData = null;
          if (null != measureResult.userId) {
            derivedQosData = NativeFastConnectModuleDefault.getDerivedQosData(str2);
            const tmpResult = NativeFastConnectModuleDefault;
          }
          let prop = measureResult.analyticsInstallation;
          if (prop == null) {
            prop = null;
          }
          const qosTokenFromDerivedData = QosToken.buildQosTokenFromDerivedData(derivedQosData, true);
          logger.info(
            "prepareIdentify: app state: ",
            AppState.currentState,
            "qosTokenPresent: ",
            qosTokenFromDerivedData.length > 0,
          );
          const d = { token, properties: null, capabilities: null, client_state: null, qos_token: null };
          const obj3 = {};
          const merged = Object.assign(discord_common_AnalyticsUtilsAll.getSuperProperties());
          obj3.client_app_state = AppState.currentState;
          obj3.is_fast_connect = true;
          obj3.gateway_connect_reasons = RequestGatewaySocketAll.describeConnectionReasons();
          if (null != prop) {
            const obj4 = { installation_id: prop };
            let obj7 = obj4;
          } else {
            obj7 = {};
          }
          const merged1 = Object.assign(obj7);
          d.properties = obj3;
          d.capabilities = GatewayCapabilities.getClientCapabilities();
          const obj8 = { guild_versions: {} };
          d.client_state = obj8;
          d.qos_token = qosTokenFromDerivedData;
          const _JSON = JSON;
          const obj9 = { op: 2, d };
          const json = JSON.stringify(obj9);
          const tmpResult2 = NativeFastConnectModuleDefault;
          let str1;
          if (measureResult.userId != null) {
            str1 = str2.toString();
          }
          if (str1 == null) {
            str1 = null;
          }
          const _parseFloat = parseFloat;
          const parsed = parseFloat(_ws.ws._socketId);
          const tmp11Result = GatewayCapabilities;
          let tmp25;
          if (tmp11Result2.isCacheEnabled()) {
            tmp25 = closure_1_5;
          }
          tmpResult2.prepareIdentify(str1, json, parsed, tmp25);
          state.identify = true;
          state.clientState = d.client_state;
          tmp11Result2 = isCacheEnabled;
        }
      }
      logger.log("Skipping fast_connect because we could not find a token to connect with.");
    });
  } else {
    obj.log("Skipping identifyWebSocket because socket is null");
  }
}
