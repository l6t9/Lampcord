// discord_app/modules/libdiscore/DispatcherBridge.tsx
import LoggerDefault from "../debug/Logger.tsx";
import SentryUtilsDefault from "../../utils/SentryUtils.native.tsx";
import libdiscoreExperiments from "libdiscoreExperiments.tsx";
import DevSettingsStore from "../devtools/dev_settings/DevSettingsStore.tsx";
import RawGuildEmojiStore from "../emojis/RawGuildEmojiStore.tsx";
import GuildStickersStore from "../stickers/GuildStickersStore.tsx";
import NoteStore from "../user_profile/notes/NoteStore.tsx";
import AuthenticationStore from "../../stores/AuthenticationStore.tsx";
import GuildRoleStore from "../../stores/GuildRoleStore.tsx";
import GuildStore from "../../stores/GuildStore.tsx";

require = fn;
const AnalyticEvents = fn(1071).AnalyticEvents;
const timestampProducer = new LoggerDefault("DispatcherBridge");
let items = [NoteStore, GuildStore, GuildRoleStore, RawGuildEmojiStore, GuildStickersStore];
let closure_7 = {
  GUILD_MEMBER_ADD(arg0) {
    const obj = {};
    const merged = Object.assign(arg0);
    obj.currentUserId = AuthenticationStore.getId();
    return obj;
  },
  CONNECTION_OPEN(guilds) {
    return { guilds: guilds.guilds, unavailableGuilds: guilds.unavailableGuilds };
  },
  CACHE_LOADED(guilds) {
    return { guilds: guilds.guilds };
  },
  CACHE_LOADED_LAZY(guilds) {
    return { guilds: guilds.guilds };
  },
  BACKGROUND_SYNC(guilds) {
    return { guilds: guilds.guilds };
  },
};
const set = new Set(["libdiscore", "typescript-libdiscore-dual-read"]);
_instance_members_initializer_DispatcherBridge_ = function () {
  this.tokenToStore = new Map();
  this.disabledFromFatalError = false;
};
class DispatcherBridge {
  constructor(arg0) {
    obj1 = Object.create(new.target.prototype);
    closure_4 = obj1;
    tmp2 = _instance_members_initializer_DispatcherBridge_();
    closure_0 = obj1;
    if (0 !== global.length) {
      tmp32 = closure_0;
      tmp33 = actionHandler;
      FLUX_API = closure_0(actionHandler[9]).FLUX_API;
      tmp34 = null;
      if (null != FLUX_API) {
        try {
          items = [];
          tmp5 = global;
          iter = global[Symbol.iterator]();
          tmp6 = globalThis;
          str2 = "]";
          str3 = ", mode: ";
          str4 = " => [token: ";
          str5 = "";
          tmp7 = global;
          nextResult = iter.next();
          tmp8 = iter;
          while (iter !== undefined) {
            name = nextResult.getName();
            result = nextResult.connectWithLibdiscore(FLUX_API);
            tokenToStore = obj1.tokenToStore;
            result1 = tokenToStore.set(result, nextResult);
            tmp12 = items;
            _HermesInternal = HermesInternal;
            str6 = "";
            tmp13 = name;
            str7 = " => [token: ";
            tmp14 = result;
            str8 = ", mode: ";
            str9 = "]";
            arr1 = items.push("" + name + " => [token: " + result + ", mode: " + nextResult.getMode() + "]");
            continue;
          }
          tmp16 = closure_6;
          length = global.length;
          tmp17 = items;
          str10 = ", ";
          _HermesInternal2 = HermesInternal;
          str11 = ".";
          str12 = " store(s), mapping: ";
          str13 = "Connected ";
          tmp18 = length;
          str14 = ".";
          infoResult = closure_6.info("Connected " + length + " store(s), mapping: " + items.join(", ") + ".");
          registeredActionTypes = FLUX_API.getRegisteredActionTypes();
          length2 = registeredActionTypes.length;
          _HermesInternal3 = HermesInternal;
          str15 = " bridged action(s): ";
          str16 = "Registering ";
          tmp20 = length2;
          infoResult1 = closure_6.info(
            "Registering " + length2 + " bridged action(s): " + registeredActionTypes.join(", ") + ".",
          );
          actionHandler = function actionHandler(type) {
            if (!obj.disabledFromFatalError) {
              const _performance = performance;
              const nowResult = performance.now();
              if (null != closure_1_7[type.type]) {
                const _JSON2 = JSON;
                const obj2 = { type: type.type };
                const merged = Object.assign(tmp3(type));
                let json = JSON.stringify(obj2);
              } else {
                const _JSON = JSON;
                json = JSON.stringify(type);
              }
              const obj3 = { kind: "json_stringify_action", durationMillis: null };
              const _performance2 = performance;
              obj3.durationMillis = performance.now() - nowResult;
              const TelemetryExperiment = obj(actionHandler[10]).TelemetryExperiment;
              const shouldCollectMetricsResult = TelemetryExperiment.shouldCollectMetrics();
              const iter = FLUX_API.dispatchAction(json, shouldCollectMetricsResult);
              if (iter.ok) {
                const _performance3 = performance;
                const diff = performance.now() - nowResult;
                ({ metrics, storeResults } = iter.value);
                items = [];
                for (const item10056 of storeResults) {
                  if (null != item10056.error) {
                    let handleStoreErrorResult = obj.handleStoreError(item10056, arg0.type);
                  } else {
                    let arr = items.push(item10056);
                  }
                  continue;
                }
                function _loop(iter2) {
                  type = iter2;
                  obj.withStoreToken(iter2.storeToken, type.type, (applyChanges) => {
                    applyChanges.applyChanges(iter2.databaseChanges);
                  });
                }
                const iter2 = items[Symbol.iterator]();
                while (iter2 !== undefined) {
                  let _loopResult = _loop(iter2.next());
                  continue;
                }
                for (const item10082 of items) {
                  let withStoreTokenResult = obj.withStoreToken(item10082.storeToken, arg0.type, (doEmitChanges) => {
                    doEmitChanges.doEmitChanges(closure_0);
                  });
                  continue;
                }
                if (null != metrics) {
                  if (shouldCollectMetricsResult) {
                    const items1 = [obj3];
                    HermesBuiltin.arraySpread(metrics.timings, 1);
                    if (_default.get("libdiscore_verbose_telemetry_logging")) {
                      let mapped = items1.map((kind) => " - " + kind.kind + ": " + kind.durationMillis + "ms");
                      const items2 = ["Timings", mapped.join("\n")];
                      const items3 = [items2, ,];
                      const mutations = metrics.mutations;
                      const mapped1 = mutations.map((metrics) => {
                        const entries = Object.entries(metrics.metrics);
                        const found = entries.filter((item) => {
                          [, tmp] = item;
                          return 0 !== tmp;
                        });
                        const mapped = found.map((item) => {
                          [tmp, tmp2] = item;
                          return " - " + tmp + ": " + tmp2;
                        });
                        return " * Record Type: " + metrics.recordType + "\n" + mapped.join("\n");
                      });
                      const items4 = ["Mutations", mapped1.join("\n")];
                      items3[1] = items4;
                      const memory = metrics.memory;
                      const mapped2 = memory.map((statistics) => {
                        const entries = Object.entries(statistics.statistics);
                        const mapped = entries.map((item) => {
                          [tmp, tmp2] = item;
                          return " - " + tmp + ": " + JSON.stringify(tmp2);
                        });
                        return " * Record Type: " + statistics.recordType + "\n" + mapped.join("\n");
                      });
                      const items5 = ["Memory Usage", mapped2.join("\n")];
                      items3[2] = items5;
                      let found = items3.filter((item) => {
                        [, arr] = item;
                        return arr.length > 0;
                      });
                      const mapped3 = found.map((item) => {
                        [tmp, tmp2] = item;
                        return "" + tmp + ":\n" + tmp2;
                      });
                      const _HermesInternal = HermesInternal;
                      logger.info("Handling action " + type.type + " took " + diff + "ms\n" + mapped3.join("\n\n"));
                    }
                    const obj4 = {
                      action_type: type.type,
                      total_duration_millis: diff,
                      timings: null,
                      mutations: null,
                      memory_usage: null,
                    };
                    const _JSON3 = JSON;
                    obj4.timings = JSON.stringify(items1);
                    const _JSON4 = JSON;
                    obj4.mutations = JSON.stringify(metrics.mutations);
                    const _JSON5 = JSON;
                    obj4.memory_usage = JSON.stringify(metrics.memory);
                    FLUX_API(actionHandler[11]).track(constants.LIBDISCORE_DISPATCH_BRIDGE_TELEMETRY, obj4);
                    const TelemetryExperiment2 = obj(actionHandler[10]).TelemetryExperiment;
                    TelemetryExperiment2.didEmit();
                    const obj8 = FLUX_API(actionHandler[11]);
                  }
                }
              } else {
                obj.handleFatalError(iter.error, type.type);
              }
            }
          };
          tmp22 = FLUX_API;
          tmp23 = actionHandler;
          obj2 = FLUX_API(actionHandler[12]);
          _Object = Object;
          tmp25 = closure_0;
          fromEntriesResult = Object.fromEntries(
            registeredActionTypes.map((item) => {
              items = [item, actionHandler];
              return items;
            }),
          );
          str17 = "LibDiscoreDispatcherBridge";
          tmp26 = obj2;
          registerResult = obj2.register(
            "LibDiscoreDispatcherBridge",
            fromEntriesResult,
            () => {},
            closure_0(actionHandler[12]).DispatchBand.Database,
          );
          _default = closure_0(actionHandler[13]).default;
          closure_3 = _default;
          addChangeListenerResult = _default.addChangeListener(() => {
            if ("active" !== _default.getState()) {
              let _Date = Date;
              closure_0 = Date.now();
              const result = FLUX_API.flushReplicationStates();
              if (result != null) {
                result.then((result) => {
                  if (result) {
                    const _Date = Date;
                    const _HermesInternal = HermesInternal;
                    logger.info("Successfully flushed replication states in " + Date.now() - closure_0 + "ms");
                  }
                });
              }
            }
          });
        } catch (tmp29) {
          tmp30 = closure_6;
          str18 = "Failed to initialize the dispatcher bridge";
          errorResult = closure_6.error("Failed to initialize the dispatcher bridge", tmp29);
        }
      } else {
        tmp3 = closure_6;
        str = "Not initializing DispatcherBridge, because kvStoreApi is unavailable.";
        infoResult2 = closure_6.info("Not initializing DispatcherBridge, because kvStoreApi is unavailable.");
      }
    }
    return obj1;
  }
}
const prototype = DispatcherBridge.prototype;
prototype["handleFatalError"] = function handleFatalError(error, type) {
  const self = this;
  error = new Error(error);
  const result = this.hasAnyAuthoritativeStore();
  logger.error("Fatal dispatch error for action", type, "hasAuthoritativeStore:", result, error);
  SentryUtilsDefault.captureException(error, {
    extra: { actionType: type, hasAuthoritativeStore: result },
    tags: { source: "libdiscore", errorKind: "fatal_dispatch" },
  });
  if (result) {
    const result1 = libdiscoreExperiments.clearLibdiscoreExperimentCache();
    throw error;
  } else {
    logger.warn("Disabling DispatcherBridge until restart");
    self.disabledFromFatalError = true;
    const tokenToStore = self.tokenToStore;
    const values = tokenToStore.values();
    for (const item10040 of values) {
      let result2 = item10040.disableDualReadValidation();
      continue;
    }
  }
  const obj3 = {
    extra: { actionType: type, hasAuthoritativeStore: result },
    tags: { source: "libdiscore", errorKind: "fatal_dispatch" },
  };
};
prototype["handleStoreError"] = function handleStoreError(storeToken, type) {
  const tokenToStore = this.tokenToStore;
  value = tokenToStore.get(storeToken.storeToken);
  let name;
  if (value != null) {
    name = value.getName();
  }
  if (name == null) {
    const _HermesInternal = HermesInternal;
    name = "unknown(token:" + storeToken.storeToken + ")";
  }
  let mode;
  if (value != null) {
    mode = value.getMode();
  }
  let str3 = storeToken.error;
  if (str3 == null) {
    str3 = "unknown store error";
  }
  const error = new Error(str3);
  logger.error("Store", name, "failed to handle action", type, "mode:", mode, error);
  SentryUtilsDefault.captureException(error, {
    extra: { actionType: type, storeName: name, storeMode: mode },
    tags: { source: "libdiscore", errorKind: "store_dispatch" },
  });
  if ("typescript-libdiscore-dual-read" !== mode) {
    if ("libdiscore" === mode) {
      const result = libdiscoreExperiments.clearLibdiscoreExperimentCache();
      let error1 = error;
    } else {
      const _Error = Error;
      const _HermesInternal3 = HermesInternal;
      error1 = new Error("unexpected storeMode '" + mode + "' for store " + name);
    }
    throw error1;
  } else {
    const _HermesInternal2 = HermesInternal;
    logger.warn("Store: " + name + " had unexpected error in Rust implementation, disabling moving forward");
    if (value != null) {
      const result1 = value.disableDualReadValidation();
    }
  }
  const obj = {
    extra: { actionType: type, storeName: name, storeMode: mode },
    tags: { source: "libdiscore", errorKind: "store_dispatch" },
  };
};
prototype["withStoreToken"] = function withStoreToken(storeToken, type, fn) {
  const tokenToStore = this.tokenToStore;
  value = tokenToStore.get(storeToken);
  if (null == value) {
    logger.warn("When dispatching action", type, "we got a store token", storeToken, "that is unknown");
  } else {
    fn(value);
  }
};
prototype["hasAnyAuthoritativeStore"] = function hasAnyAuthoritativeStore() {
  const tokenToStore = this.tokenToStore;
  const values = tokenToStore.values();
  const iter = values[Symbol.iterator]();
  while (iter !== undefined) {
    if ("libdiscore" === nextResult.getMode()) {
      iter.return();
      let flag = true;
      return true;
    }
  }
  return false;
};
const dispatcherBridge = new DispatcherBridge(items.filter((getMode) => set.has(getMode.getMode())));
const size = fn(2);
let result = size.fileFinishedImporting("modules/libdiscore/DispatcherBridge.tsx");

export default dispatcherBridge;
