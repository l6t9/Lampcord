// discord_app/modules/user_settings/phoenix_launch/native/PhoenixRolloutManager.tsx
import LoggerDefault from "../../../debug/Logger.tsx";
import NativePhoenixLaunchModuleDefault from "../../../../../discord_common/js/packages/rtn-codegen/js/NativePhoenixLaunchModule.tsx";
import asyncGeneratorStep from "../../../../../_runtime/00005_asyncGeneratorStep.js";
import AuthenticationStore from "../../../../stores/AuthenticationStore.tsx";
import AutomaticLifecycleManager from "../../../../lib/AutomaticLifecycleManager.tsx";

const require = fn;
let closure_6 = async function _applyPhoenixRollout() {
  if (c3 === 2) {
    c3 = 3;
    throw new TypeError("Generator functions may not be called on executing generators");
  } else if (tmp4 === 3) {
    if (arg0 === 1) {
      throw value;
    } else if (arg0 === 2) {
      const obj5 = { value, done: true };
      return obj5;
    } else {
      return { value: "HermesInternal", done: null };
    }
  } else {
    try {
      c3 = 2;
      if (0 === c2) {
        if (arg0 === 1) {
          c3 = 3;
          throw value;
        } else if (arg0 === 2) {
          c3 = 3;
          const obj6 = { value, done: true };
          return obj6;
        } else {
          closure_1 = tmp5;
          closure_0 = tmp2;
          closure_128_0 = undefined;
          let hasPhoenix;
          let client;
          let forcedClient;
          if (null != NativePhoenixLaunchModuleDefault) {
            id = id.getId();
            if (null != id) {
              if ("" !== id) {
                c2 = 1;
                c3 = 1;
                const obj8 = { value: NativePhoenixLaunchModuleDefault.getState(), done: false };
                return obj8;
              }
            }
          }
          c3 = 3;
        }
      } else if (1 === tmp5) {
        if (arg0 === 1) {
          c3 = 3;
          throw value;
        } else if (arg0 === 2) {
          c3 = 3;
          const obj9 = { value, done: true };
          return obj9;
        } else {
          closure_128_0 = value;
          hasPhoenix = closure_128_0.hasPhoenix;
          client = closure_128_0.client;
          if (hasPhoenix) {
            forcedClient = closure_129_1(closure_129_2[4]).getConfig({
              location: "PhoenixRolloutManager",
            }).forcedClient;
            let tmp15 = null != forcedClient;
            if (tmp15) {
              tmp15 = forcedClient !== closure_129_0(closure_129_2[5]).toLaunchClient(client);
              const obj3 = closure_129_0(closure_129_2[5]);
            }
            if (tmp15) {
              c2 = 2;
              c3 = 1;
              const obj10 = { value: closure_129_0(closure_129_2[6]).storeLaunchClient(forcedClient), done: false };
              return obj10;
            }
            const obj2 = closure_129_1(closure_129_2[4]);
          }
        }
      } else if (arg0 === 1) {
        c3 = 3;
        throw value;
      } else if (arg0 !== 2) {
        if (!value) {
          closure_129_5.error("The launch switch was not stored");
        }
      }
      c3 = 3;
      const obj = { value, done: true };
      return obj;
    } catch (tmp31) {
      c3 = tmp;
      throw tmp31;
    }
  }
};
let closure_5 = new LoggerDefault("PhoenixRolloutManager");
class PhoenixRolloutManager extends tmp3 {
  constructor() {
    applyArgumentsResult = HermesBuiltin.applyArguments(new.target, new.target);
    applyArgumentsResult.actions = { POST_CONNECTION_OPEN: applyArgumentsResult.handlePostConnectionOpen };
    return applyArgumentsResult;
  }
}
PhoenixRolloutManager.prototype["handlePostConnectionOpen"] = function handlePostConnectionOpen() {
  return (async () => {
    closure_1 = tmp3;
    await (function applyPhoenixRollout() {
      const self = this;
      const apply = closure_1_6.apply;
      if (typeof apply === "unknown") {
        let applyArgumentsResult = HermesBuiltin.applyArguments(self);
      } else {
        applyArgumentsResult = apply(self, arguments);
      }
      return applyArgumentsResult;
    })();
    if (1 === tmp7) {
      c3 = 0;
      closure_128_0 = closure_2;
      closure_129_5.error("Unable to apply the Phoenix rollout", closure_128_0);
      c5 = 3;
    } else if (arg0 === 1) {
      c5 = 3;
      throw value;
    } else if (arg0 !== 2) {
      c3 = 0;
    }
    return value;
  })();
};
const phoenixRolloutManager = new PhoenixRolloutManager();
const size = fn(2);
const result = size.fileFinishedImporting("modules/user_settings/phoenix_launch/native/PhoenixRolloutManager.tsx");

export default phoenixRolloutManager;
