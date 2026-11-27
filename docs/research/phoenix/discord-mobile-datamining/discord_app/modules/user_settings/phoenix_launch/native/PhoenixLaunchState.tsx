// discord_app/modules/user_settings/phoenix_launch/native/PhoenixLaunchState.tsx
import LoggerDefault from "../../../debug/Logger.tsx";
import LaunchClient2 from "LaunchClient.tsx";
import NativePhoenixLaunchModuleDefault from "../../../../../discord_common/js/packages/rtn-codegen/js/NativePhoenixLaunchModule.tsx";
import asyncGeneratorStep from "../../../../../_runtime/00005_asyncGeneratorStep.js";
import noop from "../../../../../_runtime/metro/00019__.js";

require = fn;
let closure_8 = async function _readPhoenixLaunchState() {
  if (c5 === 2) {
    c5 = 3;
    throw new TypeError("Generator functions may not be called on executing generators");
  } else if (tmp6 === 3) {
    if (arg0 === 1) {
      throw value;
    } else if (arg0 === 2) {
      const obj2 = { value, done: true };
      return obj2;
    } else {
      return { value: "HermesInternal", done: null };
    }
  } else {
    try {
      c5 = 2;
      if (0 === c4) {
        if (arg0 === 1) {
          c5 = 3;
          throw value;
        } else if (arg0 === 2) {
          c5 = 3;
          const obj3 = { value, done: true };
          return obj3;
        } else {
          const hasPhoenix = tmp3;
          closure_0 = tmp7;
          closure_128_0 = undefined;
          let hasPhoenix2;
          let client;
          if (null != NativePhoenixLaunchModuleDefault) {
            c3 = 1;
            c4 = 2;
            c5 = 1;
            const obj4 = { value: NativePhoenixLaunchModuleDefault.getState(), done: false };
            return obj4;
          }
        }
      } else {
        if (1 === tmp7) {
          c3 = 0;
          closure_128_3 = closure_2;
          closure_129_7 = null;
          closure_129_5.error("Failed to read the launch client", closure_128_3);
        } else if (arg0 === 1) {
          c5 = 3;
          throw value;
        } else if (arg0 !== 2) {
          closure_128_0 = value;
          hasPhoenix2 = closure_128_0.hasPhoenix;
          client = closure_128_0.client;
          closure_129_0(closure_129_2[5]).batchUpdates(() =>
            state.setState({
              hasPhoenix,
              shouldLaunchPhoenix: closure_1_2 === closure_0(closure_2[6]).LaunchClient.PHOENIX,
            }),
          );
          c3 = 0;
          const obj = closure_129_0(closure_129_2[5]);
        }
        c3 = 0;
        c5 = 3;
        const obj5 = { value, done: true };
        return obj5;
      }
      c5 = 3;
    } catch (tmp23) {
      closure_2 = tmp23;
      if (tmp4 === c3) {
        c5 = tmp2;
        throw tmp23;
      } else {
        c4 = tmp;
      }
    }
  }
};
function storeLaunchClient() {
  const self = this;
  const apply = closure_10.apply;
  if (typeof apply === "unknown") {
    let applyArgumentsResult = HermesBuiltin.applyArguments(self);
  } else {
    applyArgumentsResult = apply(self, arguments);
  }
  return applyArgumentsResult;
}
let closure_10 = async function _storeLaunchClient() {
  closure_2 = tmp5;
  closure_1 = tmp2;
  closure_129_0 = closure_0;
  if (null == NativePhoenixLaunchModuleDefault) {
    return false;
  }
  await NativePhoenixLaunchModuleDefault.setClient(closure_0);
  closure_129_1 = value;
  if (closure_129_1) {
    closure_130_0(closure_130_2[5]).batchUpdates(() =>
      state.setState({ shouldLaunchPhoenix: closure_1_0 === closure_0(closure_2[6]).LaunchClient.PHOENIX }),
    );
    closure_130_0(closure_130_2[5]);
  }
  return closure_129_1;
};
let closure_11 = async function _setShouldLaunchPhoenix(arg0) {
  if (c6 === 2) {
    c6 = 3;
    throw new TypeError("Generator functions may not be called on executing generators");
  } else if (tmp6 === 3) {
    if (arg0 === 1) {
      throw value;
    } else if (arg0 === 2) {
      const obj2 = { value, done: true };
      return obj2;
    } else {
      return { value: "HermesInternal", done: null };
    }
  } else {
    try {
      c6 = 2;
      if (0 === c5) {
        if (arg0 === 1) {
          c6 = 3;
          throw value;
        } else if (arg0 === 2) {
          c6 = 3;
          const obj3 = { value, done: true };
          return obj3;
        } else {
          closure_2 = tmp3;
          closure_1 = tmp7;
          c4 = 1;
          let tmp16 = storeLaunchClient;
          const LaunchClient = LaunchClient2.LaunchClient;
          if (closure_0) {
            let LEGACY = LaunchClient.PHOENIX;
          } else {
            LEGACY = LaunchClient.LEGACY;
          }
          tmp16 = tmp16(LEGACY);
          c5 = 2;
          c6 = 1;
        }
      } else {
        if (1 === tmp7) {
          c4 = 0;
          closure_129_0 = closure_3;
          closure_130_5.error("Failed to set the launch client", closure_129_0);
          c6 = 3;
        } else if (arg0 === 1) {
          c6 = 3;
          throw value;
        } else if (arg0 !== 2) {
          c4 = 0;
        }
        c4 = 0;
        c6 = 3;
        const obj = { value, done: true };
        return obj;
      }
    } catch (tmp19) {
      closure_3 = tmp19;
      if (tmp4 === c4) {
        c6 = tmp2;
        throw tmp19;
      } else {
        c5 = tmp;
      }
    }
  }
};
let closure_5 = new LoggerDefault("PhoenixLaunchState");
const identity = fn(1257);
let closure_6 = identity.createWithEqualityFn(() => ({ hasPhoenix: false, shouldLaunchPhoenix: false }));
let c7 = null;
const size = fn(2);
const result = size.fileFinishedImporting("modules/user_settings/phoenix_launch/native/PhoenixLaunchState.tsx");

export function useIsPhoenixInstalled() {
  const effect = noop.useEffect(() => {
    if (closure_7 == null) {
      closure_7 = (function readPhoenixLaunchState() {
        const self = this;
        const apply = closure_1_8.apply;
        if (typeof apply === "unknown") {
          let applyArgumentsResult = HermesBuiltin.applyArguments(self);
        } else {
          applyArgumentsResult = apply(self, arguments);
        }
        return applyArgumentsResult;
      })();
    }
  }, []);
  return closure_6((hasPhoenix) => hasPhoenix.hasPhoenix);
}
export function useShouldLaunchPhoenix() {
  return closure_6((shouldLaunchPhoenix) => shouldLaunchPhoenix.shouldLaunchPhoenix);
}
export { storeLaunchClient };
export function setShouldLaunchPhoenix() {
  const self = this;
  const apply = closure_11.apply;
  if (typeof apply === "unknown") {
    let applyArgumentsResult = HermesBuiltin.applyArguments(self);
  } else {
    applyArgumentsResult = apply(self, arguments);
  }
  return applyArgumentsResult;
}
