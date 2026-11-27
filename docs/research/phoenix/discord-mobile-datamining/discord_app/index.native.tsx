// discord_app/index.native.tsx
import TTITracker from "modules/tti_analytics/TTITracker.tsx";
import _mod17 from "../_runtime/metro/00017__.js";
import isTTITest from "modules/tti_analytics/isTTITest.tsx";
import installSystrace from "modules/systrace/native/installSystrace.tsx";
import logAppStart from "modules/debug/logAppStart.tsx";
import fast_connect from "modules/gateway/native/fast_connect.tsx";
import polyfills from "polyfills.android.tsx";
import checkEnv from "utils/checkEnv.tsx";
import SentryUtils from "utils/SentryUtils.native.tsx";
import size from "../_runtime/metro/00002__.js";

let GenerateInvite = require;
let f20263 = dependencyMap;
const polyfillsEnd = TTITracker.default.imports.polyfillsEnd;
polyfillsEnd.record();
const sentryEnd = TTITracker.default.imports.sentryEnd;
sentryEnd.record();
if (isTTITest.isTTITest) {
  installSystrace.installSystrace();
}
const AppRegistry = _mod17.AppRegistry;
AppRegistry.registerComponent("Discord", () => GenerateInvite(f20263[9]).default);
const runnable = AppRegistry.getRunnable("Discord");
AppRegistry.registerRunnable("Discord", () => {
  GenerateInvite = [...arguments];
  return GenerateInvite(f20263[10]).default("Main", () => {
    closure_2(...closure_0);
  });
});
AppRegistry.registerComponent("Share", () => GenerateInvite(f20263[11]).default);
const runnable2 = AppRegistry.getRunnable("Share");
AppRegistry.registerRunnable("Share", () => {
  GenerateInvite = [...arguments];
  return GenerateInvite(f20263[10]).default("Share", () => closure_3(...closure_0));
});
GenerateInvite = "BackgroundSync";
f20263 = () => GenerateInvite(f20263[13]);
AppRegistry.registerHeadlessTask("BackgroundSync", () => {
  closure_0 = GenerateInvite(f20263[12]).default;
  return (arg0) => closure_0(GenerateInvite, f20263, arg0);
});
if (isTTITest.isTTITest) {
  GenerateInvite = "TTITestAction";
  f20263 = () => GenerateInvite(f20263[14]);
  AppRegistry.registerHeadlessTask("TTITestAction", () => {
    closure_0 = GenerateInvite(f20263[12]).default;
    return (arg0) => closure_0(GenerateInvite, f20263, arg0);
  });
}
GenerateInvite = "Disconnect";
f20263 = () => GenerateInvite(f20263[15]);
AppRegistry.registerHeadlessTask("Disconnect", () => {
  closure_0 = GenerateInvite(f20263[12]).default;
  return (arg0) => closure_0(GenerateInvite, f20263, arg0);
});
GenerateInvite = "MarkAsRead";
f20263 = () => GenerateInvite(f20263[16]);
AppRegistry.registerHeadlessTask("MarkAsRead", () => {
  closure_0 = GenerateInvite(f20263[12]).default;
  return (arg0) => closure_0(GenerateInvite, f20263, arg0);
});
GenerateInvite = "MuteAction";
f20263 = () => GenerateInvite(f20263[17]);
AppRegistry.registerHeadlessTask("MuteAction", () => {
  closure_0 = GenerateInvite(f20263[12]).default;
  return (arg0) => closure_0(GenerateInvite, f20263, arg0);
});
GenerateInvite = "ToggleDeafen";
f20263 = () => GenerateInvite(f20263[18]);
AppRegistry.registerHeadlessTask("ToggleDeafen", () => {
  closure_0 = GenerateInvite(f20263[12]).default;
  return (arg0) => closure_0(GenerateInvite, f20263, arg0);
});
GenerateInvite = "ToggleSelfMute";
f20263 = () => GenerateInvite(f20263[19]);
AppRegistry.registerHeadlessTask("ToggleSelfMute", () => {
  closure_0 = GenerateInvite(f20263[12]).default;
  return (arg0) => closure_0(GenerateInvite, f20263, arg0);
});
GenerateInvite = "DismissCallAction";
f20263 = () => GenerateInvite(f20263[20]);
AppRegistry.registerHeadlessTask("DismissCallAction", () => {
  closure_0 = GenerateInvite(f20263[12]).default;
  return (arg0) => closure_0(GenerateInvite, f20263, arg0);
});
GenerateInvite = "DirectReply";
f20263 = () => GenerateInvite(f20263[21]);
AppRegistry.registerHeadlessTask("DirectReply", () => {
  closure_0 = GenerateInvite(f20263[12]).default;
  return (arg0) => closure_0(GenerateInvite, f20263, arg0);
});
GenerateInvite = "SelectVoiceChannel";
f20263 = () => GenerateInvite(f20263[22]);
AppRegistry.registerHeadlessTask("SelectVoiceChannel", () => {
  closure_0 = GenerateInvite(f20263[12]).default;
  return (arg0) => closure_0(GenerateInvite, f20263, arg0);
});
GenerateInvite = "GenerateInvite";
f20263 = () => GenerateInvite(f20263[23]);
AppRegistry.registerHeadlessTask("GenerateInvite", () => {
  closure_0 = GenerateInvite(f20263[12]).default;
  return (arg0) => closure_0(GenerateInvite, f20263, arg0);
});
const result = size.fileFinishedImporting("index.native.tsx");

// === Orphan Functions ===

function global() {
  let __BUNDLE_START_TIME__;
  let __DEV__;
  let process;
  let __METRO_GLOBAL_PREFIX__;
  if (globalThis.nativePerformanceNow) {
    let nativePerformanceNowResult = globalThis.nativePerformanceNow();
  } else {
    let _Date = Date;
    nativePerformanceNowResult = Date.now();
  }
  globalThis.__BUNDLE_START_TIME__ = nativePerformanceNowResult;
  globalThis.__DEV__ = false;
  globalThis.process = process || {};
  globalThis.__METRO_GLOBAL_PREFIX__ = "";
  if (!env) {
    env = {};
  }
  process.env = env;
  let str = process.env.NODE_ENV;
  if (!str) {
    str = "production";
  }
  let self = this;
  process.env.NODE_ENV = str;
  if (typeof globalThis !== "undefined") {
    let _window = globalThis;
  } else {
    let _global = global;
    if (typeof global !== "undefined") {
      _window = global;
    } else {
      let _window3 = window;
      _window = self;
      if (typeof window !== "undefined") {
        _window = window;
      }
    }
  }
  let React = _window;
  function metroRequire(id, arg1) {
    if (null === id) {
      const _Error = Error;
      const error = new Error("Cannot find module");
      throw error;
    } else {
      value = duration.get(id);
      if (value) {
        if (value.isInitialized) {
          let _exports = value.publicModule.exports;
        }
        return _exports;
      }
      _exports = (function loadModuleImplementation(id, value2) {
        value = value2;
        if (!value2) {
          value = value2;
          if (length.length > 0) {
            let num2 = map.get(id);
            if (num2 == null) {
              num2 = 0;
            }
            value = value2;
            if (null != tmp8[num2]) {
              tmp10(id);
              value = closure_1.get(id);
              map.delete(id);
            }
          }
        }
        const nativeRequire = __timingFunction.nativeRequire;
        value2 = value;
        if (!value) {
          value2 = value;
          if (nativeRequire) {
            const tmp17 = closure_11(id);
            nativeRequire(tmp17.localId, tmp17.segmentId);
            value2 = closure_1.get(id);
          }
        }
        if (value2) {
          if (value2.hasError) {
            throw value2.error;
          } else {
            value2.isInitialized = true;
            dependencyMap = value2.dependencyMap;
            __timingFunction = tmp14.__timingFunction;
            const __timingFunctionResult = __timingFunction();
            closure_1 = __timingFunctionResult;
            if (false === c3) {
              c3 = true;
              tmp14.__getTotalRequireTime = () => React2 + (__timingFunction() - __timingFunctionResult);
            }
            try {
              const publicModule = value2.publicModule;
              publicModule.id = id;
              tmp20(tmp14, closure_6, closure_7, closure_8, publicModule, publicModule.exports, dependencyMap);
              value2.factory = undefined;
              value2.dependencyMap = undefined;
              if (tmp22) {
                c3 = false;
                closure_2 = closure_2 + (__timingFunction() - __timingFunctionResult);
                tmp14.__getTotalRequireTime = () => closure_2;
              }
              return publicModule.exports;
            } catch (tmp32) {
              if (tmp6) {
                c3 = tmp3;
                closure_2 = closure_2 + (tmp5() - tmp4);
                tmp.__getTotalRequireTime = () => closure_2;
              }
              throw tmp32;
            }
          }
        } else {
          throw (function unknownModuleError(id) {
            return Error('Requiring unknown module "' + id + '".');
          })(id);
        }
      })(id, value);
    }
  }
  let timestampProducer = metroRequire;
  function metroImportDefault(prototype) {
    value = duration.get(prototype);
    if (value) {
      if (value.importedDefault !== closure_4) {
        return value.importedDefault;
      }
    }
    if (null === prototype) {
      const _Error = Error;
      const error = new Error("Cannot find module");
      throw error;
    } else {
      const value2 = duration.get(prototype);
      if (value2) {
        if (value2.isInitialized) {
          let _exports = value2.publicModule.exports;
        }
        let _default = _exports;
        if (_exports) {
          _default = _exports;
          if (_exports.__esModule) {
            _default = _exports.default;
          }
        }
        duration.get(prototype).importedDefault = _default;
        return _default;
      }
      _exports = (function loadModuleImplementation(id, value2) {
        value = value2;
        if (!value2) {
          value = value2;
          if (length.length > 0) {
            let num2 = map.get(id);
            if (num2 == null) {
              num2 = 0;
            }
            value = value2;
            if (null != tmp8[num2]) {
              tmp10(id);
              value = closure_1.get(id);
              map.delete(id);
            }
          }
        }
        const nativeRequire = __timingFunction.nativeRequire;
        value2 = value;
        if (!value) {
          value2 = value;
          if (nativeRequire) {
            const tmp17 = closure_11(id);
            nativeRequire(tmp17.localId, tmp17.segmentId);
            value2 = closure_1.get(id);
          }
        }
        if (value2) {
          if (value2.hasError) {
            throw value2.error;
          } else {
            value2.isInitialized = true;
            dependencyMap = value2.dependencyMap;
            __timingFunction = tmp14.__timingFunction;
            const __timingFunctionResult = __timingFunction();
            closure_1 = __timingFunctionResult;
            if (false === c3) {
              c3 = true;
              tmp14.__getTotalRequireTime = () => React2 + (__timingFunction() - __timingFunctionResult);
            }
            try {
              const publicModule = value2.publicModule;
              publicModule.id = id;
              tmp20(tmp14, closure_6, closure_7, closure_8, publicModule, publicModule.exports, dependencyMap);
              value2.factory = undefined;
              value2.dependencyMap = undefined;
              if (tmp22) {
                c3 = false;
                closure_2 = closure_2 + (__timingFunction() - __timingFunctionResult);
                tmp14.__getTotalRequireTime = () => closure_2;
              }
              return publicModule.exports;
            } catch (tmp32) {
              if (tmp6) {
                c3 = tmp3;
                closure_2 = closure_2 + (tmp5() - tmp4);
                tmp.__getTotalRequireTime = () => closure_2;
              }
              throw tmp32;
            }
          }
        } else {
          throw (function unknownModuleError(id) {
            return Error('Requiring unknown module "' + id + '".');
          })(id);
        }
      })(prototype, value2);
    }
  }
  function metroImportAll(id) {
    value = duration.get(id);
    if (value) {
      if (value.importedAll !== closure_4) {
        return value.importedAll;
      }
    }
    if (null === id) {
      const _Error = Error;
      const error = new Error("Cannot find module");
      throw error;
    } else {
      let value2 = duration.get(id);
      if (value2) {
        if (value2.isInitialized) {
          let _exports = value2.publicModule.exports;
        }
        if (!_exports) {
          const obj = {};
          if (_exports) {
            for (const key10017 in _exports) {
              let call = hasOwnProperty.call;
              if (typeof call === "unknown") {
                let callResult = hasOwnProperty(key10017);
              } else {
                callResult = call(_exports, key10017);
              }
              if (!callResult) {
                continue;
              } else {
                obj[key10017] = _exports[key10017];
                continue;
              }
              continue;
            }
          }
          obj.default = _exports;
          let tmp3 = obj;
        } else {
          tmp3 = _exports;
        }
        duration.get(id).importedAll = tmp3;
        return tmp3;
      }
      _exports = (function loadModuleImplementation(id, value2) {
        value = value2;
        if (!value2) {
          value = value2;
          if (length.length > 0) {
            let num2 = map.get(id);
            if (num2 == null) {
              num2 = 0;
            }
            value = value2;
            if (null != tmp8[num2]) {
              tmp10(id);
              value = closure_1.get(id);
              map.delete(id);
            }
          }
        }
        const nativeRequire = __timingFunction.nativeRequire;
        value2 = value;
        if (!value) {
          value2 = value;
          if (nativeRequire) {
            const tmp17 = closure_11(id);
            nativeRequire(tmp17.localId, tmp17.segmentId);
            value2 = closure_1.get(id);
          }
        }
        if (value2) {
          if (value2.hasError) {
            throw value2.error;
          } else {
            value2.isInitialized = true;
            dependencyMap = value2.dependencyMap;
            __timingFunction = tmp14.__timingFunction;
            const __timingFunctionResult = __timingFunction();
            closure_1 = __timingFunctionResult;
            if (false === c3) {
              c3 = true;
              tmp14.__getTotalRequireTime = () => React2 + (__timingFunction() - __timingFunctionResult);
            }
            try {
              const publicModule = value2.publicModule;
              publicModule.id = id;
              tmp20(tmp14, closure_6, closure_7, closure_8, publicModule, publicModule.exports, dependencyMap);
              value2.factory = undefined;
              value2.dependencyMap = undefined;
              if (tmp22) {
                c3 = false;
                closure_2 = closure_2 + (__timingFunction() - __timingFunctionResult);
                tmp14.__getTotalRequireTime = () => closure_2;
              }
              return publicModule.exports;
            } catch (tmp32) {
              if (tmp6) {
                c3 = tmp3;
                closure_2 = closure_2 + (tmp5() - tmp4);
                tmp.__getTotalRequireTime = () => closure_2;
              }
              throw tmp32;
            }
          }
        } else {
          throw (function unknownModuleError(id) {
            return Error('Requiring unknown module "' + id + '".');
          })(id);
        }
      })(id, value2);
    }
  }
  let React5 = metroImportAll;
  function unpackModuleId(arg0) {
    return { segmentId: arg0 >>> closure_9, localId: arg0 & closure_10 };
  }
  closure_11 = unpackModuleId;
  _window.__r = metroRequire;
  _window["" + globalThis.__METRO_GLOBAL_PREFIX__ + "__d"] = function define(factory, arg1, dependencyMap) {
    if (!duration.has(arg1)) {
      const obj = {
        dependencyMap,
        factory,
        hasError: false,
        importedAll: React4,
        importedDefault: React4,
        isInitialized: false,
        publicModule: null,
      };
      const obj2 = { exports: {} };
      obj.publicModule = obj2;
      const result = duration.set(arg1, obj);
    }
  };
  _window.__c = function clear() {
    const map = new Map();
    closure_1 = map;
    return map;
  };
  _window.__registerSegment = function registerSegment(arg0, arg1, arr) {
    closure_0 = arg0;
    closure_12[arg0] = arg1;
    if (arr) {
      const item = arr.forEach((item) => {
        let hasItem = set.has(item);
        if (!hasItem) {
          hasItem = map1.has(item);
        }
        if (!hasItem) {
          const result = map1.set(item, closure_0);
        }
      });
    }
  };
  let duration = new Map();
  c2 = 0;
  c3 = false;
  _window.__timingFunction = () => Date.now();
  _window.__getTotalRequireTime = () => c2;
  let React4 = {};
  let hasOwnProperty = {}.hasOwnProperty;
  metroRequire.importDefault = metroImportDefault;
  metroRequire.importAll = metroImportAll;
  metroRequire.context = function fallbackRequireContext() {
    const error = new Error("The experimental Metro feature `require.context` is not enabled in your project.");
    throw error;
  };
  metroRequire.resolveWeak = function fallbackRequireResolveWeak() {
    const error = new Error("require.resolveWeak cannot be called dynamically.");
    throw error;
  };
  let React6 = 16;
  let refs = 65535;
  metroRequire.unpackModuleId = unpackModuleId;
  metroRequire.packModuleId = function packModuleId(segmentId) {
    return (segmentId.segmentId << closure_9) + segmentId.localId;
  };
  let application_id = [];
  let map1 = new Map();
  if (typeof globalThis !== "undefined") {
    let _window2 = globalThis;
  } else {
    const _global2 = global;
    if (typeof global !== "undefined") {
      _window2 = global;
    } else {
      let _window4 = window;
      _window2 = self;
      if (typeof window !== "undefined") {
        _window2 = window;
      }
    }
  }
  closure_129_0 = _window2;
  function stub() {}
  function consoleCreateTaskStub() {
    return {
      run(fn) {
        return fn();
      },
    };
  }
  closure_130_0 = function stylizeNoColor(arg0, arg1) {
    return arg0;
  };
  closure_130_1 = function formatValue(formatValueCalls, name, arg2) {
    let obj = formatValueCalls;
    closure_1 = name;
    React2 = arg2;
    formatValueCalls.formatValueCalls = formatValueCalls.formatValueCalls + 1;
    if (formatValueCalls.formatValueCalls > 200) {
      const _HermesInternal4 = HermesInternal;
      return "[TOO BIG formatValueCalls " + formatValueCalls.formatValueCalls + " exceeded limit of 200]";
    } else {
      if (undefined === name) {
        let stylizeResult = formatValueCalls.stylize("undefined", "undefined");
      } else if (typeof name === "string") {
        const _JSON = JSON;
        const str3 = JSON.stringify(name);
        const str5 = JSON.stringify(name).replace(/^"|"$/g, "");
        stylizeResult = formatValueCalls.stylize(
          `'${JSON.stringify(name).replace(/^"|"$/g, "").replace(/'/g, "\\'").replace(/\\"/g, '"')}'`,
          "string",
        );
        const str7 = JSON.stringify(name).replace(/^"|"$/g, "").replace(/'/g, "\\'");
      } else if (typeof name === "number") {
        stylizeResult = formatValueCalls.stylize("" + name, "number");
      } else if (typeof name === "boolean") {
        stylizeResult = formatValueCalls.stylize("" + name, "boolean");
      } else if (null === name) {
        stylizeResult = formatValueCalls.stylize("null", "null");
      }
      if (stylizeResult) {
        return stylizeResult;
      } else {
        const _Object = Object;
        const keys = Object.keys(name);
        obj = {};
        const item = keys.forEach((item, index) => {
          obj[item] = true;
        });
        let tmp5 = typeof name === "object";
        let tmp6 = tmp5;
        if (typeof name === "object") {
          tmp6 = null !== name;
        }
        if (tmp6) {
          const _Object2 = Object;
          const call = toString.call;
          let tmp7 = "[object Error]" === (typeof call === "unknown" ? toString() : call(name));
          if (!tmp7) {
            const _Error = Error;
            tmp7 = name instanceof Error;
          }
          tmp6 = tmp7;
        }
        if (tmp6) {
          const _Error6 = Error;
          const call17 = toString7.call;
          return "[" + (typeof call17 === "unknown" ? toString7() : call17(name)) + "]";
        }
        if (0 === keys.length) {
          if (typeof name === "function") {
            let str43 = "";
            if (name.name) {
              str43 = `: ${name.name}`;
            }
            const _HermesInternal3 = HermesInternal;
            return formatValueCalls.stylize("[Function" + str43 + "]", "special");
          } else {
            let tmp54 = tmp5;
            if (typeof name === "object") {
              tmp54 = null !== name;
            }
            if (!tmp54) {
              if (tmp54) {
                const _RegExp3 = RegExp;
                const call16 = toString6.call;
                return formatValueCalls.stylize(typeof call16 === "unknown" ? toString6() : call16(name), "regexp");
              } else {
                let tmp10 = tmp5;
                if (typeof name === "object") {
                  tmp10 = null !== name;
                }
                if (!tmp10) {
                  if (tmp10) {
                    const _Date2 = Date;
                    const call15 = toString5.call;
                    return formatValueCalls.stylize(typeof call15 === "unknown" ? toString5() : call15(name), "date");
                  } else {
                    let tmp13 = tmp5;
                    if (typeof name === "object") {
                      tmp13 = null !== name;
                    }
                    if (tmp13) {
                      const _Object5 = Object;
                      const call4 = toString2.call;
                      let tmp14 = "[object Error]" === (typeof call4 === "unknown" ? toString2() : call4(name));
                      if (!tmp14) {
                        const _Error2 = Error;
                        tmp14 = name instanceof Error;
                      }
                      tmp13 = tmp14;
                    }
                    if (tmp13) {
                      const _Error5 = Error;
                      const call14 = toString4.call;
                      return "[" + (typeof call14 === "unknown" ? toString4() : call14(name)) + "]";
                    }
                  }
                } else {
                  const _Object4 = Object;
                  let str15 = Object.prototype.toString;
                  const call3 = str15.call;
                  str15 = "[object Date]";
                  const tmp11 = typeof call3 === "unknown" ? str15() : call3(name);
                }
              }
            } else {
              const _Object3 = Object;
              let str14 = Object.prototype.toString;
              const call2 = str14.call;
              str14 = "[object RegExp]";
              const tmp8 = typeof call2 === "unknown" ? str14() : call2(name);
            }
          }
        }
        let flag = false;
        c4 = false;
        items = ["{", "}"];
        const _Array = Array;
        if (Array.isArray(name)) {
          c4 = true;
          items = ["[", "]"];
          flag = true;
        }
        let str17 = "";
        let str18 = "";
        if (typeof name === "function") {
          let text = str17;
          if (name.name) {
            text = `: ${name.name}`;
          }
          const _HermesInternal = HermesInternal;
          str18 = " [Function" + text + "]";
        }
        let tmp16 = tmp5;
        if (typeof name === "object") {
          tmp16 = null !== name;
        }
        if (!tmp16) {
          if (!tmp16) {
            let tmp21 = tmp5;
            if (typeof name === "object") {
              tmp21 = null !== name;
            }
            if (!tmp21) {
              if (!tmp21) {
                let tmp26 = tmp5;
                if (typeof name === "object") {
                  tmp26 = null !== name;
                }
                if (tmp26) {
                  const _Object8 = Object;
                  const call9 = toString3.call;
                  let tmp27 = "[object Error]" === (typeof call9 === "unknown" ? toString3() : call9(name));
                  if (!tmp27) {
                    const _Error3 = Error;
                    tmp27 = name instanceof Error;
                  }
                  tmp26 = tmp27;
                }
                if (!tmp26) {
                  if (0 === keys.length) {
                    return items[0] + str18 + items[1];
                  }
                  if (arg2 >= 0) {
                    const seen = formatValueCalls.seen;
                    seen.push(name);
                    if (flag) {
                      closure_129_0 = formatValueCalls;
                      closure_129_1 = name;
                      closure_129_2 = arg2;
                      closure_129_3 = obj;
                      const items1 = [];
                      closure_129_4 = items1;
                      for (let num4 = 0; num4 < length; num4 = num4 + 1) {
                        let _String = String;
                        let StringResult = String(num4);
                        let _Object9 = Object;
                        hasOwnProperty = Object.prototype.hasOwnProperty;
                        let call11 = hasOwnProperty.call;
                        let push = items1.push;
                        if (typeof call11 === "unknown" ? hasOwnProperty(StringResult) : call11(name, StringResult)) {
                          let _String2 = String;
                          let flag3 = true;
                          let arr4 = push(React2(formatValueCalls, name, arg2, obj, String(num4), true));
                        } else {
                          let arr5 = push(str17);
                        }
                      }
                      const item1 = keys.forEach((item) => {
                        if (!item.match(/^\d+$/)) {
                          _true.push(c2(obj, closure_1, closure_2, obj, item, true));
                        }
                      });
                      let mapped = items1;
                      length = name.length;
                    } else {
                      mapped = keys.map((item) => c2(obj, closure_1, closure_2, obj, item, c4));
                    }
                    const seen1 = formatValueCalls.seen;
                    seen1.pop();
                    if (
                      mapped.reduce((acc, arr) => {
                        arr.indexOf("\n") >= 0;
                        return acc + arr.replace(/\u001b\[\d\d?m/g, "").length + 1;
                      }, 0) > 60
                    ) {
                      if (str17 !== str18) {
                        str17 = `${str18}
     `;
                      }
                      const sum = items[0] + str17;
                      let text1 = `${tmp44} ${arr3.join(",\n  ")} ${arr2[1]}`;
                    } else {
                      const sum1 = items[0] + str18;
                      text1 = `${tmp42} ${arr3.join(", ")} ${arr2[1]}`;
                    }
                  }
                  if (typeof name === "object") {
                    tmp5 = null !== name;
                  }
                  if (!tmp5) {
                    const stylize = formatValueCalls.stylize;
                    if (!tmp5) {
                      stylize("[Object]", "special");
                    }
                    const _RegExp2 = RegExp;
                    let str38 = RegExp.prototype.toString;
                    const call13 = str38.call;
                    str38 = "regexp";
                    stylize(typeof call13 === "unknown" ? str38() : call13(name), "regexp");
                    const tmp47 = typeof call13 === "unknown" ? str38() : call13(name);
                  } else {
                    const _Object10 = Object;
                    let str35 = Object.prototype.toString;
                    const call12 = str35.call;
                    str35 = "[object RegExp]";
                    const tmp45 = typeof call12 === "unknown" ? str35() : call12(name);
                  }
                } else {
                  const _Error4 = Error;
                  let str27 = Error.prototype.toString;
                  const call10 = str27.call;
                  const _HermesInternal2 = HermesInternal;
                  const text2 = `[${typeof call10 === "unknown" ? str27() : call10(name)}`;
                  str27 = " ";
                  const combined = " " + `[${typeof call10 === "unknown" ? str27() : call10(name)}` + "]";
                }
              } else {
                const _Date = Date;
                let str25 = Date.prototype.toUTCString;
                const call8 = str25.call;
                str25 = " ";
                const text3 = ` ${typeof call8 === "unknown" ? str25() : call8(name)}`;
                const tmp24 = typeof call8 === "unknown" ? str25() : call8(name);
              }
            } else {
              const _Object7 = Object;
              let str24 = Object.prototype.toString;
              const call7 = str24.call;
              str24 = "[object Date]";
              const tmp22 = typeof call7 === "unknown" ? str24() : call7(name);
            }
          } else {
            const _RegExp = RegExp;
            let str23 = RegExp.prototype.toString;
            const call6 = str23.call;
            str23 = " ";
            const text4 = ` ${typeof call6 === "unknown" ? str23() : call6(name)}`;
            const tmp19 = typeof call6 === "unknown" ? str23() : call6(name);
          }
        } else {
          const _Object6 = Object;
          let str22 = Object.prototype.toString;
          const call5 = str22.call;
          str22 = "[object RegExp]";
          const tmp17 = typeof call5 === "unknown" ? str22() : call5(name);
        }
      }
    }
  };
  closure_130_2 = function formatProperty(stylize, arg1, arg2, arg3, key10009, arg5) {
    let iter = Object.getOwnPropertyDescriptor(arg1, key10009);
    if (!iter) {
      const obj = { value: arg1[key10009] };
      iter = obj;
    }
    if (iter.get) {
      stylize = stylize.stylize;
      if (set) {
        let stylizeResult = stylize("[Getter/Setter]", "special");
      } else {
        stylizeResult = stylize("[Getter]", "special");
      }
    } else {
      let stylizeResult1;
      if (set) {
        stylizeResult1 = stylize.stylize("[Setter]", "special");
      }
      const _Object = Object;
      hasOwnProperty = Object.prototype.hasOwnProperty;
      const call = hasOwnProperty.call;
      if (!(typeof call === "unknown" ? hasOwnProperty(key10009) : call(arg3, key10009))) {
        let text = `${"[" + key10009}]`;
      }
      if (stylizeResult1) {
        if (undefined === text) {
          if (arg5) {
            if (key10009.match(/^\d+$/)) {
              return stylizeResult1;
            }
          }
          const _JSON = JSON;
          const str12 = JSON.stringify("" + key10009);
          if (str12.match(/^"([a-zA-Z_][a-zA-Z_0-9]*)"$/)) {
            text = stylize.stylize(str12.slice(1, str12.length - 1), "name");
          } else {
            const str14 = str12.replace(/'/g, "\\'");
            text = stylize.stylize(
              str12
                .replace(/'/g, "\\'")
                .replace(/\\"/g, '"')
                .replace(/(^"|"$)/g, "'"),
              "string",
            );
            const str16 = str12.replace(/'/g, "\\'").replace(/\\"/g, '"');
          }
        }
        return text + ": " + stylizeResult1;
      } else {
        const seen = stylize.seen;
        if (seen.indexOf(iter.value) < 0) {
          if (null === arg2) {
            let arr2 = duration(stylize, iter.value, null);
          } else {
            arr2 = duration(stylize, iter.value, arg2 - 1);
          }
          iter = arr2.indexOf("\n");
          if (iter <= -1) {
            let stylizeResult2 = arr2;
          } else {
            const parts = arr2.split("\n");
            let map = parts.map;
            if (arg5) {
              const mapped = map((arg0) => "  " + arg0);
              map = mapped.join("\n");
              let substr = map.slice(2);
            } else {
              const mapped1 = map((arg0) => "   " + arg0);
              substr = `
    ${obj2.join("\n")}`;
            }
          }
        } else {
          stylizeResult2 = stylize.stylize("[Circular]", "special");
        }
      }
    }
  };
  closure_129_1 = function inspect(arg0, arg1) {};
  closure_129_2 = "(index)";
  closure_129_3 = 1;
  closure_129_4 = 2;
  closure_129_5 = 3;
  closure_129_6 = [];
  let _console1 = _window2.console;
  if (_window2.nativeLoggingHook) {
    let obj = {
      time: stub,
      timeEnd: stub,
      timeStamp: stub,
      count: stub,
      countReset: stub,
      createTask: consoleCreateTaskStub,
    };
    if (_console1 == null) {
      _console1 = {};
    }
    let merged = Object.assign(_console1);
    obj.error = () => {
      if (1 === arguments.length) {
        if (typeof arguments[0] === "string") {
          let first = arguments[0];
        }
        const first1 = arguments[0];
        let tmp4 = stylize;
        let tmp5 = typeof first1 === "string";
        if (typeof first1 === "string") {
          tmp5 = "Warning: " === first1.slice(0, 9);
        }
        if (tmp5) {
          tmp5 = tmp4 >= closure_2_5;
        }
        if (tmp5) {
          tmp4 = closure_2_4;
        }
        let text1 = first;
        if (closure_2_6.length) {
          let str2 = first;
          const text = `${obj2.join("")}`;
          if (!first) {
            str2 = "";
          }
          text1 = `${tmp8} ${str2}`;
        }
        closure_2_0.nativeLoggingHook(text1, tmp4);
      }
      const call = map.call;
      const fn = (arg0) => {
        if (typeof duration === "function") {
          const obj = { seen: [], formatValueCalls: 0, stylize };
          return closure_1(obj, arg0, 10);
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      };
      if (typeof call === "unknown") {
        let mapped = map(fn);
      } else {
        mapped = call(arguments, fn);
      }
      first = mapped.join(", ");
    };
    obj.info = () => {
      if (1 === arguments.length) {
        if (typeof arguments[0] === "string") {
          let first = arguments[0];
        }
        const first1 = arguments[0];
        let tmp4 = stylize;
        let tmp5 = typeof first1 === "string";
        if (typeof first1 === "string") {
          tmp5 = "Warning: " === first1.slice(0, 9);
        }
        if (tmp5) {
          tmp5 = tmp4 >= closure_2_5;
        }
        if (tmp5) {
          tmp4 = closure_2_4;
        }
        let text1 = first;
        if (closure_2_6.length) {
          let str2 = first;
          const text = `${obj2.join("")}`;
          if (!first) {
            str2 = "";
          }
          text1 = `${tmp8} ${str2}`;
        }
        closure_2_0.nativeLoggingHook(text1, tmp4);
      }
      const call = map.call;
      const fn = (arg0) => {
        if (typeof duration === "function") {
          const obj = { seen: [], formatValueCalls: 0, stylize };
          return closure_1(obj, arg0, 10);
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      };
      if (typeof call === "unknown") {
        let mapped = map(fn);
      } else {
        mapped = call(arguments, fn);
      }
      first = mapped.join(", ");
    };
    obj.log = () => {
      if (1 === arguments.length) {
        if (typeof arguments[0] === "string") {
          let first = arguments[0];
        }
        const first1 = arguments[0];
        let tmp4 = stylize;
        let tmp5 = typeof first1 === "string";
        if (typeof first1 === "string") {
          tmp5 = "Warning: " === first1.slice(0, 9);
        }
        if (tmp5) {
          tmp5 = tmp4 >= closure_2_5;
        }
        if (tmp5) {
          tmp4 = closure_2_4;
        }
        let text1 = first;
        if (closure_2_6.length) {
          let str2 = first;
          const text = `${obj2.join("")}`;
          if (!first) {
            str2 = "";
          }
          text1 = `${tmp8} ${str2}`;
        }
        closure_2_0.nativeLoggingHook(text1, tmp4);
      }
      const call = map.call;
      const fn = (arg0) => {
        if (typeof duration === "function") {
          const obj = { seen: [], formatValueCalls: 0, stylize };
          return closure_1(obj, arg0, 10);
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      };
      if (typeof call === "unknown") {
        let mapped = map(fn);
      } else {
        mapped = call(arguments, fn);
      }
      first = mapped.join(", ");
    };
    obj.warn = () => {
      if (1 === arguments.length) {
        if (typeof arguments[0] === "string") {
          let first = arguments[0];
        }
        const first1 = arguments[0];
        let tmp4 = stylize;
        let tmp5 = typeof first1 === "string";
        if (typeof first1 === "string") {
          tmp5 = "Warning: " === first1.slice(0, 9);
        }
        if (tmp5) {
          tmp5 = tmp4 >= closure_2_5;
        }
        if (tmp5) {
          tmp4 = closure_2_4;
        }
        let text1 = first;
        if (closure_2_6.length) {
          let str2 = first;
          const text = `${obj2.join("")}`;
          if (!first) {
            str2 = "";
          }
          text1 = `${tmp8} ${str2}`;
        }
        closure_2_0.nativeLoggingHook(text1, tmp4);
      }
      const call = map.call;
      const fn = (arg0) => {
        if (typeof duration === "function") {
          const obj = { seen: [], formatValueCalls: 0, stylize };
          return closure_1(obj, arg0, 10);
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      };
      if (typeof call === "unknown") {
        let mapped = map(fn);
      } else {
        mapped = call(arguments, fn);
      }
      first = mapped.join(", ");
    };
    obj.trace = () => {
      if (1 === arguments.length) {
        if (typeof arguments[0] === "string") {
          let first = arguments[0];
        }
        const first1 = arguments[0];
        let tmp4 = stylize;
        let tmp5 = typeof first1 === "string";
        if (typeof first1 === "string") {
          tmp5 = "Warning: " === first1.slice(0, 9);
        }
        if (tmp5) {
          tmp5 = tmp4 >= closure_2_5;
        }
        if (tmp5) {
          tmp4 = closure_2_4;
        }
        let text1 = first;
        if (closure_2_6.length) {
          let str2 = first;
          const text = `${obj2.join("")}`;
          if (!first) {
            str2 = "";
          }
          text1 = `${tmp8} ${str2}`;
        }
        closure_2_0.nativeLoggingHook(text1, tmp4);
      }
      const call = map.call;
      const fn = (arg0) => {
        if (typeof duration === "function") {
          const obj = { seen: [], formatValueCalls: 0, stylize };
          return closure_1(obj, arg0, 10);
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      };
      if (typeof call === "unknown") {
        let mapped = map(fn);
      } else {
        mapped = call(arguments, fn);
      }
      first = mapped.join(", ");
    };
    React = 0;
    obj.debug = () => {
      if (1 === arguments.length) {
        if (typeof arguments[0] === "string") {
          let first = arguments[0];
        }
        const first1 = arguments[0];
        let tmp4 = stylize;
        let tmp5 = typeof first1 === "string";
        if (typeof first1 === "string") {
          tmp5 = "Warning: " === first1.slice(0, 9);
        }
        if (tmp5) {
          tmp5 = tmp4 >= closure_2_5;
        }
        if (tmp5) {
          tmp4 = closure_2_4;
        }
        let text1 = first;
        if (closure_2_6.length) {
          let str2 = first;
          const text = `${obj2.join("")}`;
          if (!first) {
            str2 = "";
          }
          text1 = `${tmp8} ${str2}`;
        }
        closure_2_0.nativeLoggingHook(text1, tmp4);
      }
      const call = map.call;
      const fn = (arg0) => {
        if (typeof duration === "function") {
          const obj = { seen: [], formatValueCalls: 0, stylize };
          return closure_1(obj, arg0, 10);
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      };
      if (typeof call === "unknown") {
        let mapped = map(fn);
      } else {
        mapped = call(arguments, fn);
      }
      first = mapped.join(", ");
    };
    obj.table = function consoleTablePolyfill(arr, arg1) {
      if (Array.isArray(arr)) {
        let mapped = arr.map((item, index) => {
          const obj = { [closure_1_2]: String(index) };
          const merged = Object.assign(obj, item);
          return obj;
        });
        items = mapped;
        let arr2 = mapped;
      } else {
        items = [];
        arr2 = items;
        let keys = Object.keys();
        if (keys !== undefined) {
          arr2 = items;
          while (keys[tmp] !== undefined) {
            if (!arr.hasOwnProperty(tmp4)) {
              continue;
            } else {
              let obj = {};
              obj[items3] = tmp4;
              let _Object = Object;
              let merged = Object.assign(obj, arr[tmp4]);
              arr = items.push(obj);
              continue;
            }
            continue;
          }
        }
      }
      if (0 !== arr2.length) {
        const _Array = Array;
        if (Array.isArray(arg1)) {
          const items1 = [items3];
          let combined = items1.concat(arg1);
        } else {
          const _Array2 = Array;
          const _Set = Set;
          const set = new Set();
          combined = Array.from(
            arr2.reduce((acc, item) => {
              const keys = Object.keys(item);
              item = keys.forEach((item) => acc.add(item));
              return acc;
            }, set),
          );
        }
        const items2 = [];
        items3 = [];
        let item = combined.forEach((item, index) => {
          items3[index] = item.length;
          for (let num = 0; num < items.length; num = num + 1) {
            let obj = items[num];
            if (item === c2) {
              let str = obj[item];
            } else {
              str = "";
              if (obj.hasOwnProperty(item)) {
                let tmp3 = obj[item];
                let tmp4 = typeof tmp3;
                str = "\u0192";
                if ("function" !== tmp4) {
                  if ("string" === tmp4) {
                    str = `${"'" + tmp3}'`;
                  } else if ("object" === tmp4) {
                    let str2 = "{\u2026}";
                    if (null == tmp3) {
                      str2 = "null";
                    }
                    str = str2;
                  } else {
                    let _String = String;
                    str = String(tmp3);
                  }
                }
              }
            }
            items2[num] = items2[num] || [];
            items2[num][index] = str;
            let _Math = Math;
            items3[index] = Math.max(items3[index], str.length);
          }
        });
        const mapped1 = items3.map((item) => {
          c0 = "-";
          const mapped = Array.apply(null, Array(item)).map(() => c0);
          return mapped.join("");
        });
        const mapped2 = mapped1.map((item, index) => {
          c0 = " ";
          const mapped = Array.apply(null, Array(items3[index] - item.length)).map(() => c0);
          return item + mapped.join("");
        });
        const text = `| ${obj2.join(" | ")}`;
        const mapped3 = combined.map((item, index) => {
          c0 = " ";
          const mapped = Array.apply(null, Array(items3[index] - item.length)).map(() => c0);
          return item + mapped.join("");
        });
        const items4 = [`| ${obj3.join(" | ")} |`, `${`| ${obj2.join(" | ")}`} |`];
        let num2 = 0;
        if (0 < arr2.length) {
          do {
            let arr9 = items2[num2];
            let mapped4 = arr9.map((item, index) => {
              c0 = " ";
              const mapped = Array.apply(null, Array(items3[index] - item.length)).map(() => c0);
              return item + mapped.join("");
            });
            let arr3 = items4.push("| " + mapped4.join(" | ") + " |");
            num2 = num2 + 1;
            length = arr2.length;
          } while (num2 < length);
        }
        items.nativeLoggingHook(
          `
      ${arr8.join("\n")}`,
          c3,
        );
      } else {
        items.nativeLoggingHook("", c3);
      }
    };
    obj.group = function consoleGroupPolyfill(arg0) {
      let str = arg0;
      const text = `${closure_6.join("")}┐`;
      if (!arg0) {
        str = "";
      }
      React.nativeLoggingHook(`${tmp2} ${str}`, c3);
      timestampProducer.push("\u2502");
    };
    obj.groupEnd = function consoleGroupEndPolyfill() {
      timestampProducer.pop();
      React.nativeLoggingHook(`${closure_6.join("")}┘ `, c3);
    };
    obj.groupCollapsed = function consoleGroupCollapsedPolyfill(arg0) {
      let str = arg0;
      const text = `${closure_6.join("")}┘`;
      if (!arg0) {
        str = "";
      }
      React.nativeLoggingHook(`${tmp2} ${str}`, c3);
      timestampProducer.push("\u2502");
    };
    obj.assert = function consoleAssertPolyfill(arg0, arg1) {
      if (!arg0) {
        React.nativeLoggingHook(`Assertion failed: ${arg1}`, hasOwnProperty);
      }
    };
    _window2.console = obj;
    if (true === _window2.RN$useAlwaysAvailableJSErrorHandling) {
      let _console2 = console;
      let _console3 = console;
      console.error = function () {
        items = [...arguments];
        metroImportDefault.apply(this, items);
        if (false !== console.reportErrorsAsExceptions) {
          let result;
          if (React.RN$inExceptionHandler != null) {
            result = RN$inExceptionHandler();
          }
          if (!result) {
            const first = items[0];
            let stack;
            if (first != null) {
              stack = first.stack;
            }
            let tmp4 = first;
            if (!stack) {
              if (typeof first !== "string") {
                const mapped = items.map((item) => {
                  let replaced = item;
                  if (typeof item !== "string") {
                    if (typeof duration === "function") {
                      const obj = { seen: [], formatValueCalls: 0, stylize };
                      replaced = closure_1(obj, item, 10).replace(/\n\s*/g, " ");
                      const str = closure_1(obj, item, 10);
                    } else {
                      throw new TypeError("Trying to call a non-function");
                    }
                  }
                  return replaced;
                });
                const _Error = Error;
                const error = new Error(mapped.join(" "));
                error.name = "console.error";
                tmp4 = error;
              }
            }
            const result1 = React.RN$handleException(tmp4, false, false);
          }
        }
      };
    }
    let _Object2 = Object;
    let _console4 = console;
    Object.defineProperty(console, "_isPolyfilled", { value: true, enumerable: false });
  } else if (!_console1) {
    closure_129_8 = tmp4;
    let obj2 = {
      debug: _window2.print || stub,
      error: _window2.print || stub,
      info: _window2.print || stub,
      log: _window2.print || stub,
      trace: _window2.print || stub,
      warn: _window2.print || stub,
      assert(arg0, arg1) {
        if (!arg0) {
          React5(`Assertion failed: ${arg1}`);
        }
      },
      clear: stub,
      count: stub,
      countReset: stub,
      dir: stub,
      dirxml: stub,
      group: stub,
      groupCollapsed: stub,
      groupEnd: stub,
      profile: stub,
      profileEnd: stub,
      table: stub,
      time: stub,
      timeEnd: stub,
      timeStamp: stub,
      createTask: consoleCreateTaskStub,
    };
    _window2.console = obj2;
    let _Object = Object;
    let _console = console;
    Object.defineProperty(console, "_isPolyfilled", { value: true, enumerable: false });
  }
  if (typeof globalThis !== "undefined") {
    self = globalThis;
  } else {
    const _global3 = global;
    if (typeof global !== "undefined") {
      self = global;
    } else {
      let _window5 = window;
      if (typeof window !== "undefined") {
        self = window;
      }
    }
  }
  closure_131_0 = 0;
  closure_131_1 =
    true === self.RN$useAlwaysAvailableJSErrorHandling
      ? self.RN$handleException
      : (arg0, arg1) => {
          throw arg0;
        };
  let obj3 = {
    setGlobalHandler(arg0) {
      closure_1 = arg0;
    },
    getGlobalHandler() {
      return closure_1;
    },
    reportError(arg0) {
      if (closure_1) {
        tmp(arg0, false);
      }
    },
    reportFatalError(error) {
      if (closure_1) {
        tmp(error, true);
      }
    },
    applyWithGuard(apply, self, items, arg3, arg4) {
      try {
        closure_0 = closure_0 + 1;
        closure_0 = closure_0 - 1;
        return apply.apply(self, items);
      } catch (tmp6) {
        closure_0 = closure_0 - 1;
        throw tmp6;
      }
    },
    applyWithGuardIfNeeded(apply, self, items) {
      if (React2.inGuard()) {
        let applyResult = apply.apply(self, items);
      } else {
        React2.applyWithGuard(apply, self, items);
        applyResult = null;
      }
      return applyResult;
    },
    inGuard() {
      return closure_0;
    },
    guard(name, arg1, arg2) {
      closure_0 = name;
      closure_1 = arg2;
      if (typeof name !== "function") {
        const _console = console;
        console.warn("A function must be passed to ErrorUtils.guard, got ", name);
        return null;
      } else {
        let str = arg1;
        if (arg1 == null) {
          str = name.name;
        }
        if (str == null) {
          str = "<generated guard>";
        }
        return function guarded() {
          items = [...arguments];
          let self = closure_1;
          if (closure_1 == null) {
            self = this;
          }
          return str.applyWithGuard(closure_0, self, items, null, str);
        };
      }
    },
  };
  closure_131_2 = obj3;
  self.ErrorUtils = obj3;
  // Metro registry: 20218 module registrations omitted (each __d(factory, id, deps) wires a module rendered above)
  __r(119);
  return __r(0);
}

function startsWith(str, arg1, arg2) {
  return str.startsWith(arg1, arg2);
}

function startsWith(arr, arg1, arg2) {
  return arr.slice(arg2, arg2 + arg1.length) === arg1;
}

function fromCodePoint() {
  let length;
  const items = [];
  let num = 0;
  if (0 < arguments.length) {
    do {
      items[num] = arguments[num];
      num = num + 1;
      length = arguments.length;
    } while (num < length);
  }
  let str = "";
  let num2 = 0;
  let str2 = "";
  if (items.length > 0) {
    while (items[num2] <= 1114111) {
      if (tmp < 65536) {
        let _String2 = String;
        let fromCharCodeResult = String.fromCharCode(tmp);
      } else {
        let _String = String;
        let diff = tmp - 65536;
        fromCharCodeResult = String.fromCharCode(55296 + (diff >> 10), (diff % 1024) + 56320);
      }
      num2 = num2 + 1;
      str = str + fromCharCodeResult;
      str2 = str;
    }
    const _RangeError = RangeError;
    throw RangeError(items[num2] + " is not a valid code point");
  }
  return str2;
}

function fromEntries(mapped) {
  let length;
  const obj = {};
  let num = 0;
  if (0 < mapped.length) {
    do {
      let tmp = mapped[num];
      obj[tmp[0]] = tmp[1];
      num = num + 1;
      length = mapped.length;
    } while (num < length);
  }
  return obj;
}

function codePointAt(str, arg1) {
  return str.codePointAt(arg1);
}

function codePointAt(str, arg1) {
  if (arg1 >= 0) {
    if (arg1 < length) {
      const charCodeAtResult = str.charCodeAt(arg1);
      let sum = charCodeAtResult;
      if (charCodeAtResult >= 55296) {
        sum = charCodeAtResult;
        if (charCodeAtResult <= 56319) {
          sum = charCodeAtResult;
          if (arg1 + 1 !== length) {
            const charCodeAtResult1 = str.charCodeAt(arg1 + 1);
            sum = charCodeAtResult;
            if (charCodeAtResult1 >= 56320) {
              sum = charCodeAtResult;
              if (charCodeAtResult1 <= 57343) {
                sum = charCodeAtResult1 - 56320 + ((charCodeAtResult - 55296) << 10) + 65536;
              }
            }
          }
        }
      }
      return sum;
    }
  }
}

function trimStart(str) {
  return str.trimStart();
}

function trimStart(str) {
  return str.replace(closure_1_2, "");
}

function trimEnd(str) {
  return str.trimEnd();
}

function trimEnd(str) {
  return str.replace(closure_1_3, "");
}

function matchIdentifierAtIndex(arg0, lastIndex) {
  closure_1_12.lastIndex = lastIndex;
  const tmp = closure_1_12.exec(arg0)[1];
  let str = "";
  if (null !== tmp) {
    str = "";
    if (undefined !== tmp) {
      str = tmp;
    }
  }
  return str;
}

function matchIdentifierAtIndex(arg0, arg1) {
  let sum = arg1;
  const items = [];
  const tmp2 = closure_1_9(arg0, sum);
  const tmp3 = tmp2;
  while (undefined !== tmp2) {
    let tmp852 = tmp3 >= 9;
    let tmp6 = tmp852;
    if (tmp852) {
      tmp6 = tmp3 <= 13;
    }
    if (!tmp6) {
      tmp6 = 32 === tmp3;
    }
    if (!tmp6) {
      tmp6 = 133 === tmp3;
    }
    if (!tmp6) {
      let tmp13 = tmp3 >= 8206;
      let tmp14 = tmp13;
      if (tmp13) {
        tmp14 = tmp3 <= 8207;
      }
      tmp6 = tmp14;
    }
    if (!tmp6) {
      tmp6 = 8232 === tmp3;
    }
    if (!tmp6) {
      tmp6 = 8233 === tmp3;
    }
    if (tmp6) {
      break;
    } else {
      let tmp23 = tmp3 >= 33;
      let tmp24 = tmp23;
      if (tmp23) {
        tmp24 = tmp3 <= 35;
      }
      if (!tmp24) {
        tmp24 = 36 === tmp3;
      }
      if (!tmp24) {
        let tmp31 = tmp3 >= 37;
        let tmp32 = tmp31;
        if (tmp31) {
          tmp32 = tmp3 <= 39;
        }
        tmp24 = tmp32;
      }
      if (!tmp24) {
        tmp24 = 40 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 41 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 42 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 43 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 44 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 45 === tmp3;
      }
      if (!tmp24) {
        let tmp49 = tmp3 >= 46;
        let tmp50 = tmp49;
        if (tmp49) {
          tmp50 = tmp3 <= 47;
        }
        tmp24 = tmp50;
      }
      if (!tmp24) {
        let tmp55 = tmp3 >= 58;
        let tmp56 = tmp55;
        if (tmp55) {
          tmp56 = tmp3 <= 59;
        }
        tmp24 = tmp56;
      }
      if (!tmp24) {
        let tmp61 = tmp3 >= 60;
        let tmp62 = tmp61;
        if (tmp61) {
          tmp62 = tmp3 <= 62;
        }
        tmp24 = tmp62;
      }
      if (!tmp24) {
        let tmp67 = tmp3 >= 63;
        let tmp68 = tmp67;
        if (tmp67) {
          tmp68 = tmp3 <= 64;
        }
        tmp24 = tmp68;
      }
      if (!tmp24) {
        tmp24 = 91 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 92 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 93 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 94 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 96 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 123 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 124 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 125 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 126 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 161 === tmp3;
      }
      if (!tmp24) {
        let tmp93 = tmp3 >= 162;
        let tmp94 = tmp93;
        if (tmp93) {
          tmp94 = tmp3 <= 165;
        }
        tmp24 = tmp94;
      }
      if (!tmp24) {
        tmp24 = 166 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 167 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 169 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 171 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 172 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 174 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 176 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 177 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 182 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 187 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 191 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 215 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 247 === tmp3;
      }
      if (!tmp24) {
        let tmp125 = tmp3 >= 8208;
        let tmp126 = tmp125;
        if (tmp125) {
          tmp126 = tmp3 <= 8213;
        }
        tmp24 = tmp126;
      }
      if (!tmp24) {
        let tmp131 = tmp3 >= 8214;
        let tmp132 = tmp131;
        if (tmp131) {
          tmp132 = tmp3 <= 8215;
        }
        tmp24 = tmp132;
      }
      if (!tmp24) {
        tmp24 = 8216 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 8217 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 8218 === tmp3;
      }
      if (!tmp24) {
        let tmp143 = tmp3 >= 8219;
        let tmp144 = tmp143;
        if (tmp143) {
          tmp144 = tmp3 <= 8220;
        }
        tmp24 = tmp144;
      }
      if (!tmp24) {
        tmp24 = 8221 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 8222 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 8223 === tmp3;
      }
      if (!tmp24) {
        let tmp155 = tmp3 >= 8224;
        let tmp156 = tmp155;
        if (tmp155) {
          tmp156 = tmp3 <= 8231;
        }
        tmp24 = tmp156;
      }
      if (!tmp24) {
        let tmp161 = tmp3 >= 8240;
        let tmp162 = tmp161;
        if (tmp161) {
          tmp162 = tmp3 <= 8248;
        }
        tmp24 = tmp162;
      }
      if (!tmp24) {
        tmp24 = 8249 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 8250 === tmp3;
      }
      if (!tmp24) {
        let tmp171 = tmp3 >= 8251;
        let tmp172 = tmp171;
        if (tmp171) {
          tmp172 = tmp3 <= 8254;
        }
        tmp24 = tmp172;
      }
      if (!tmp24) {
        let tmp177 = tmp3 >= 8257;
        let tmp178 = tmp177;
        if (tmp177) {
          tmp178 = tmp3 <= 8259;
        }
        tmp24 = tmp178;
      }
      if (!tmp24) {
        tmp24 = 8260 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 8261 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 8262 === tmp3;
      }
      if (!tmp24) {
        let tmp189 = tmp3 >= 8263;
        let tmp190 = tmp189;
        if (tmp189) {
          tmp190 = tmp3 <= 8273;
        }
        tmp24 = tmp190;
      }
      if (!tmp24) {
        tmp24 = 8274 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 8275 === tmp3;
      }
      if (!tmp24) {
        let tmp199 = tmp3 >= 8277;
        let tmp200 = tmp199;
        if (tmp199) {
          tmp200 = tmp3 <= 8286;
        }
        tmp24 = tmp200;
      }
      if (!tmp24) {
        let tmp205 = tmp3 >= 8592;
        let tmp206 = tmp205;
        if (tmp205) {
          tmp206 = tmp3 <= 8596;
        }
        tmp24 = tmp206;
      }
      if (!tmp24) {
        let tmp211 = tmp3 >= 8597;
        let tmp212 = tmp211;
        if (tmp211) {
          tmp212 = tmp3 <= 8601;
        }
        tmp24 = tmp212;
      }
      if (!tmp24) {
        let tmp217 = tmp3 >= 8602;
        let tmp218 = tmp217;
        if (tmp217) {
          tmp218 = tmp3 <= 8603;
        }
        tmp24 = tmp218;
      }
      if (!tmp24) {
        let tmp223 = tmp3 >= 8604;
        let tmp224 = tmp223;
        if (tmp223) {
          tmp224 = tmp3 <= 8607;
        }
        tmp24 = tmp224;
      }
      if (!tmp24) {
        tmp24 = 8608 === tmp3;
      }
      if (!tmp24) {
        let tmp231 = tmp3 >= 8609;
        let tmp232 = tmp231;
        if (tmp231) {
          tmp232 = tmp3 <= 8610;
        }
        tmp24 = tmp232;
      }
      if (!tmp24) {
        tmp24 = 8611 === tmp3;
      }
      if (!tmp24) {
        let tmp239 = tmp3 >= 8612;
        let tmp240 = tmp239;
        if (tmp239) {
          tmp240 = tmp3 <= 8613;
        }
        tmp24 = tmp240;
      }
      if (!tmp24) {
        tmp24 = 8614 === tmp3;
      }
      if (!tmp24) {
        let tmp247 = tmp3 >= 8615;
        let tmp248 = tmp247;
        if (tmp247) {
          tmp248 = tmp3 <= 8621;
        }
        tmp24 = tmp248;
      }
      if (!tmp24) {
        tmp24 = 8622 === tmp3;
      }
      if (!tmp24) {
        let tmp255 = tmp3 >= 8623;
        let tmp256 = tmp255;
        if (tmp255) {
          tmp256 = tmp3 <= 8653;
        }
        tmp24 = tmp256;
      }
      if (!tmp24) {
        let tmp261 = tmp3 >= 8654;
        let tmp262 = tmp261;
        if (tmp261) {
          tmp262 = tmp3 <= 8655;
        }
        tmp24 = tmp262;
      }
      if (!tmp24) {
        let tmp267 = tmp3 >= 8656;
        let tmp268 = tmp267;
        if (tmp267) {
          tmp268 = tmp3 <= 8657;
        }
        tmp24 = tmp268;
      }
      if (!tmp24) {
        tmp24 = 8658 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 8659 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 8660 === tmp3;
      }
      if (!tmp24) {
        let tmp279 = tmp3 >= 8661;
        let tmp280 = tmp279;
        if (tmp279) {
          tmp280 = tmp3 <= 8691;
        }
        tmp24 = tmp280;
      }
      if (!tmp24) {
        let tmp285 = tmp3 >= 8692;
        let tmp286 = tmp285;
        if (tmp285) {
          tmp286 = tmp3 <= 8959;
        }
        tmp24 = tmp286;
      }
      if (!tmp24) {
        let tmp291 = tmp3 >= 8960;
        let tmp292 = tmp291;
        if (tmp291) {
          tmp292 = tmp3 <= 8967;
        }
        tmp24 = tmp292;
      }
      if (!tmp24) {
        tmp24 = 8968 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 8969 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 8970 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 8971 === tmp3;
      }
      if (!tmp24) {
        let tmp305 = tmp3 >= 8972;
        let tmp306 = tmp305;
        if (tmp305) {
          tmp306 = tmp3 <= 8991;
        }
        tmp24 = tmp306;
      }
      if (!tmp24) {
        let tmp311 = tmp3 >= 8992;
        let tmp312 = tmp311;
        if (tmp311) {
          tmp312 = tmp3 <= 8993;
        }
        tmp24 = tmp312;
      }
      if (!tmp24) {
        let tmp317 = tmp3 >= 8994;
        let tmp318 = tmp317;
        if (tmp317) {
          tmp318 = tmp3 <= 9000;
        }
        tmp24 = tmp318;
      }
      if (!tmp24) {
        tmp24 = 9001 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 9002 === tmp3;
      }
      if (!tmp24) {
        let tmp327 = tmp3 >= 9003;
        let tmp328 = tmp327;
        if (tmp327) {
          tmp328 = tmp3 <= 9083;
        }
        tmp24 = tmp328;
      }
      if (!tmp24) {
        tmp24 = 9084 === tmp3;
      }
      if (!tmp24) {
        let tmp335 = tmp3 >= 9085;
        let tmp336 = tmp335;
        if (tmp335) {
          tmp336 = tmp3 <= 9114;
        }
        tmp24 = tmp336;
      }
      if (!tmp24) {
        let tmp341 = tmp3 >= 9115;
        let tmp342 = tmp341;
        if (tmp341) {
          tmp342 = tmp3 <= 9139;
        }
        tmp24 = tmp342;
      }
      if (!tmp24) {
        let tmp347 = tmp3 >= 9140;
        let tmp348 = tmp347;
        if (tmp347) {
          tmp348 = tmp3 <= 9179;
        }
        tmp24 = tmp348;
      }
      if (!tmp24) {
        let tmp353 = tmp3 >= 9180;
        let tmp354 = tmp353;
        if (tmp353) {
          tmp354 = tmp3 <= 9185;
        }
        tmp24 = tmp354;
      }
      if (!tmp24) {
        let tmp359 = tmp3 >= 9186;
        let tmp360 = tmp359;
        if (tmp359) {
          tmp360 = tmp3 <= 9254;
        }
        tmp24 = tmp360;
      }
      if (!tmp24) {
        let tmp365 = tmp3 >= 9255;
        let tmp366 = tmp365;
        if (tmp365) {
          tmp366 = tmp3 <= 9279;
        }
        tmp24 = tmp366;
      }
      if (!tmp24) {
        let tmp371 = tmp3 >= 9280;
        let tmp372 = tmp371;
        if (tmp371) {
          tmp372 = tmp3 <= 9290;
        }
        tmp24 = tmp372;
      }
      if (!tmp24) {
        let tmp377 = tmp3 >= 9291;
        let tmp378 = tmp377;
        if (tmp377) {
          tmp378 = tmp3 <= 9311;
        }
        tmp24 = tmp378;
      }
      if (!tmp24) {
        let tmp383 = tmp3 >= 9472;
        let tmp384 = tmp383;
        if (tmp383) {
          tmp384 = tmp3 <= 9654;
        }
        tmp24 = tmp384;
      }
      if (!tmp24) {
        tmp24 = 9655 === tmp3;
      }
      if (!tmp24) {
        let tmp391 = tmp3 >= 9656;
        let tmp392 = tmp391;
        if (tmp391) {
          tmp392 = tmp3 <= 9664;
        }
        tmp24 = tmp392;
      }
      if (!tmp24) {
        tmp24 = 9665 === tmp3;
      }
      if (!tmp24) {
        let tmp399 = tmp3 >= 9666;
        let tmp400 = tmp399;
        if (tmp399) {
          tmp400 = tmp3 <= 9719;
        }
        tmp24 = tmp400;
      }
      if (!tmp24) {
        let tmp405 = tmp3 >= 9720;
        let tmp406 = tmp405;
        if (tmp405) {
          tmp406 = tmp3 <= 9727;
        }
        tmp24 = tmp406;
      }
      if (!tmp24) {
        let tmp411 = tmp3 >= 9728;
        let tmp412 = tmp411;
        if (tmp411) {
          tmp412 = tmp3 <= 9838;
        }
        tmp24 = tmp412;
      }
      if (!tmp24) {
        tmp24 = 9839 === tmp3;
      }
      if (!tmp24) {
        let tmp419 = tmp3 >= 9840;
        let tmp420 = tmp419;
        if (tmp419) {
          tmp420 = tmp3 <= 10087;
        }
        tmp24 = tmp420;
      }
      if (!tmp24) {
        tmp24 = 10088 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10089 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10090 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10091 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10092 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10093 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10094 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10095 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10096 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10097 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10098 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10099 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10100 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10101 === tmp3;
      }
      if (!tmp24) {
        let tmp453 = tmp3 >= 10132;
        let tmp454 = tmp453;
        if (tmp453) {
          tmp454 = tmp3 <= 10175;
        }
        tmp24 = tmp454;
      }
      if (!tmp24) {
        let tmp459 = tmp3 >= 10176;
        let tmp460 = tmp459;
        if (tmp459) {
          tmp460 = tmp3 <= 10180;
        }
        tmp24 = tmp460;
      }
      if (!tmp24) {
        tmp24 = 10181 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10182 === tmp3;
      }
      if (!tmp24) {
        let tmp469 = tmp3 >= 10183;
        let tmp470 = tmp469;
        if (tmp469) {
          tmp470 = tmp3 <= 10213;
        }
        tmp24 = tmp470;
      }
      if (!tmp24) {
        tmp24 = 10214 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10215 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10216 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10217 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10218 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10219 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10220 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10221 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10222 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10223 === tmp3;
      }
      if (!tmp24) {
        let tmp495 = tmp3 >= 10224;
        let tmp496 = tmp495;
        if (tmp495) {
          tmp496 = tmp3 <= 10239;
        }
        tmp24 = tmp496;
      }
      if (!tmp24) {
        let tmp501 = tmp3 >= 10240;
        let tmp502 = tmp501;
        if (tmp501) {
          tmp502 = tmp3 <= 10495;
        }
        tmp24 = tmp502;
      }
      if (!tmp24) {
        let tmp507 = tmp3 >= 10496;
        let tmp508 = tmp507;
        if (tmp507) {
          tmp508 = tmp3 <= 10626;
        }
        tmp24 = tmp508;
      }
      if (!tmp24) {
        tmp24 = 10627 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10628 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10629 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10630 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10631 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10632 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10633 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10634 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10635 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10636 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10637 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10638 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10639 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10640 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10641 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10642 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10643 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10644 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10645 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10646 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10647 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10648 === tmp3;
      }
      if (!tmp24) {
        let tmp557 = tmp3 >= 10649;
        let tmp558 = tmp557;
        if (tmp557) {
          tmp558 = tmp3 <= 10711;
        }
        tmp24 = tmp558;
      }
      if (!tmp24) {
        tmp24 = 10712 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10713 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10714 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10715 === tmp3;
      }
      if (!tmp24) {
        let tmp571 = tmp3 >= 10716;
        let tmp572 = tmp571;
        if (tmp571) {
          tmp572 = tmp3 <= 10747;
        }
        tmp24 = tmp572;
      }
      if (!tmp24) {
        tmp24 = 10748 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 10749 === tmp3;
      }
      if (!tmp24) {
        let tmp581 = tmp3 >= 10750;
        let tmp582 = tmp581;
        if (tmp581) {
          tmp582 = tmp3 <= 11007;
        }
        tmp24 = tmp582;
      }
      if (!tmp24) {
        let tmp587 = tmp3 >= 11008;
        let tmp588 = tmp587;
        if (tmp587) {
          tmp588 = tmp3 <= 11055;
        }
        tmp24 = tmp588;
      }
      if (!tmp24) {
        let tmp593 = tmp3 >= 11056;
        let tmp594 = tmp593;
        if (tmp593) {
          tmp594 = tmp3 <= 11076;
        }
        tmp24 = tmp594;
      }
      if (!tmp24) {
        let tmp599 = tmp3 >= 11077;
        let tmp600 = tmp599;
        if (tmp599) {
          tmp600 = tmp3 <= 11078;
        }
        tmp24 = tmp600;
      }
      if (!tmp24) {
        let tmp605 = tmp3 >= 11079;
        let tmp606 = tmp605;
        if (tmp605) {
          tmp606 = tmp3 <= 11084;
        }
        tmp24 = tmp606;
      }
      if (!tmp24) {
        let tmp611 = tmp3 >= 11085;
        let tmp612 = tmp611;
        if (tmp611) {
          tmp612 = tmp3 <= 11123;
        }
        tmp24 = tmp612;
      }
      if (!tmp24) {
        let tmp617 = tmp3 >= 11124;
        let tmp618 = tmp617;
        if (tmp617) {
          tmp618 = tmp3 <= 11125;
        }
        tmp24 = tmp618;
      }
      if (!tmp24) {
        let tmp623 = tmp3 >= 11126;
        let tmp624 = tmp623;
        if (tmp623) {
          tmp624 = tmp3 <= 11157;
        }
        tmp24 = tmp624;
      }
      if (!tmp24) {
        tmp24 = 11158 === tmp3;
      }
      if (!tmp24) {
        let tmp631 = tmp3 >= 11159;
        let tmp632 = tmp631;
        if (tmp631) {
          tmp632 = tmp3 <= 11263;
        }
        tmp24 = tmp632;
      }
      if (!tmp24) {
        let tmp637 = tmp3 >= 11776;
        let tmp638 = tmp637;
        if (tmp637) {
          tmp638 = tmp3 <= 11777;
        }
        tmp24 = tmp638;
      }
      if (!tmp24) {
        tmp24 = 11778 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11779 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11780 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11781 === tmp3;
      }
      if (!tmp24) {
        let tmp651 = tmp3 >= 11782;
        let tmp652 = tmp651;
        if (tmp651) {
          tmp652 = tmp3 <= 11784;
        }
        tmp24 = tmp652;
      }
      if (!tmp24) {
        tmp24 = 11785 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11786 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11787 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11788 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11789 === tmp3;
      }
      if (!tmp24) {
        let tmp667 = tmp3 >= 11790;
        let tmp668 = tmp667;
        if (tmp667) {
          tmp668 = tmp3 <= 11798;
        }
        tmp24 = tmp668;
      }
      if (!tmp24) {
        tmp24 = 11799 === tmp3;
      }
      if (!tmp24) {
        let tmp675 = tmp3 >= 11800;
        let tmp676 = tmp675;
        if (tmp675) {
          tmp676 = tmp3 <= 11801;
        }
        tmp24 = tmp676;
      }
      if (!tmp24) {
        tmp24 = 11802 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11803 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11804 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11805 === tmp3;
      }
      if (!tmp24) {
        let tmp689 = tmp3 >= 11806;
        let tmp690 = tmp689;
        if (tmp689) {
          tmp690 = tmp3 <= 11807;
        }
        tmp24 = tmp690;
      }
      if (!tmp24) {
        tmp24 = 11808 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11809 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11810 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11811 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11812 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11813 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11814 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11815 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11816 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11817 === tmp3;
      }
      if (!tmp24) {
        let tmp715 = tmp3 >= 11818;
        let tmp716 = tmp715;
        if (tmp715) {
          tmp716 = tmp3 <= 11822;
        }
        tmp24 = tmp716;
      }
      if (!tmp24) {
        tmp24 = 11823 === tmp3;
      }
      if (!tmp24) {
        let tmp723 = tmp3 >= 11824;
        let tmp724 = tmp723;
        if (tmp723) {
          tmp724 = tmp3 <= 11833;
        }
        tmp24 = tmp724;
      }
      if (!tmp24) {
        let tmp729 = tmp3 >= 11834;
        let tmp730 = tmp729;
        if (tmp729) {
          tmp730 = tmp3 <= 11835;
        }
        tmp24 = tmp730;
      }
      if (!tmp24) {
        let tmp735 = tmp3 >= 11836;
        let tmp736 = tmp735;
        if (tmp735) {
          tmp736 = tmp3 <= 11839;
        }
        tmp24 = tmp736;
      }
      if (!tmp24) {
        tmp24 = 11840 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11841 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 11842 === tmp3;
      }
      if (!tmp24) {
        let tmp747 = tmp3 >= 11843;
        let tmp748 = tmp747;
        if (tmp747) {
          tmp748 = tmp3 <= 11855;
        }
        tmp24 = tmp748;
      }
      if (!tmp24) {
        let tmp753 = tmp3 >= 11856;
        let tmp754 = tmp753;
        if (tmp753) {
          tmp754 = tmp3 <= 11857;
        }
        tmp24 = tmp754;
      }
      if (!tmp24) {
        tmp24 = 11858 === tmp3;
      }
      if (!tmp24) {
        let tmp761 = tmp3 >= 11859;
        let tmp762 = tmp761;
        if (tmp761) {
          tmp762 = tmp3 <= 11903;
        }
        tmp24 = tmp762;
      }
      if (!tmp24) {
        let tmp767 = tmp3 >= 12289;
        let tmp768 = tmp767;
        if (tmp767) {
          tmp768 = tmp3 <= 12291;
        }
        tmp24 = tmp768;
      }
      if (!tmp24) {
        tmp24 = 12296 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12297 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12298 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12299 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12300 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12301 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12302 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12303 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12304 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12305 === tmp3;
      }
      if (!tmp24) {
        let tmp793 = tmp3 >= 12306;
        let tmp794 = tmp793;
        if (tmp793) {
          tmp794 = tmp3 <= 12307;
        }
        tmp24 = tmp794;
      }
      if (!tmp24) {
        tmp24 = 12308 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12309 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12310 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12311 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12312 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12313 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12314 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12315 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12316 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12317 === tmp3;
      }
      if (!tmp24) {
        let tmp819 = tmp3 >= 12318;
        let tmp820 = tmp819;
        if (tmp819) {
          tmp820 = tmp3 <= 12319;
        }
        tmp24 = tmp820;
      }
      if (!tmp24) {
        tmp24 = 12320 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 12336 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 64830 === tmp3;
      }
      if (!tmp24) {
        tmp24 = 64831 === tmp3;
      }
      if (!tmp24) {
        let tmp833 = tmp3 >= 65093;
        let tmp834 = tmp833;
        if (tmp833) {
          tmp834 = tmp3 <= 65094;
        }
        tmp24 = tmp834;
      }
      if (tmp24) {
        break;
      } else {
        let push = arr2.push;
        let arr = push(tmp3);
        let tmp845 = tmp3 >= 65536;
        let num88 = 1;
        if (tmp845) {
          num88 = 2;
        }
        sum = tmp4 + num88;
        continue;
      }
    }
  }
  return closure_1_7.apply(undefined, items);
}

function f24631() {
  class Parser {
    constructor(arg0, arg1) {
      obj = {};
      obj1 = arg1;
      if (undefined === arg1) {
        obj1 = {};
      }
      obj.message = arg0;
      obj.position = { offset: 0, line: 1, column: 1 };
      obj.ignoreTag = obj1.ignoreTag;
      obj.locale = obj1.locale;
      obj.requiresOtherClause = obj1.requiresOtherClause;
      obj.shouldParseSkeletons = obj1.shouldParseSkeletons;
      return;
    }
    parse() {
      self = this;
      if (0 !== this.offset()) {
        tmp = globalThis;
        _Error = Error;
        str2 = "parser can only be used once";
        throw Error("parser can only be used once");
      } else {
        flag = false;
        str = "";
        return self.parseMessage(0, "", false);
      }
    }
    parseMessage(arg0, arg1, arg2) {
      self = this;
      items = [];
      if (!this.isEOF()) {
        while (true) {
          charResult = self.char();
          if (123 === charResult) {
            parseArgumentResult = self.parseArgument(arg0, arg2);
            if (parseArgumentResult.err) {
              return parseArgumentResult;
            } else {
              arr1 = items.push(parseArgumentResult.val);
              if (self.isEOF()) {
                break;
              }
            }
          } else {
            if (125 !== charResult) {
              if (35 !== charResult) {
                tmp7 = 60 === charResult;
                if (tmp7) {
                  if (!self.ignoreTag) {
                    if (47 === self.peek()) {
                      if (arg2) {
                        break;
                      } else {
                        tmp8 = closure_1_0;
                        tmp9 = closure_1_1;
                        clonePositionResult = self.clonePosition();
                        obj1 = { start: null, end: null };
                        obj1.start = clonePositionResult;
                        obj1.end = self.clonePosition();
                        return self.error(closure_1_0(closure_1_1[2]).ErrorKind.UNMATCHED_CLOSING_TAG, obj1);
                      }
                    }
                    break;
                  }
                }
                if (tmp7) {
                  if (!self.ignoreTag) {
                    tmp11 = self.peek() || 0;
                    tmp12 = tmp11 >= 97 && tmp11 <= 122;
                    if (!tmp12) {
                      tmp13 = tmp11 >= 65 && tmp11 <= 90;
                      tmp12 = tmp13;
                    }
                    if (tmp12) {
                      parseTagResult = self.parseTag(arg0, arg1);
                      if (parseTagResult.err) {
                        return parseTagResult;
                      } else {
                        arr2 = items.push(parseTagResult.val);
                      }
                    }
                  }
                }
                parseLiteralResult = self.parseLiteral(arg0, arg1);
                if (parseLiteralResult.err) {
                  return parseLiteralResult;
                } else {
                  arr3 = items.push(parseLiteralResult.val);
                }
              } else if ("plural" === arg1) {
                clonePositionResult1 = self.clonePosition();
                bumpResult = self.bump();
                obj = { type: null, location: null };
                tmp4 = closure_1_0;
                tmp5 = closure_1_1;
                obj.type = closure_1_0(closure_1_1[1]).TYPE.pound;
                obj4 = { start: null, end: null };
                obj4.start = clonePositionResult1;
                obj4.end = self.clonePosition();
                obj.location = obj4;
                arr4 = items.push(obj);
              }
            } else if (arg0 > 0) {
              break;
            }
            break;
          }
          break;
        }
      }
      return { val: items, err: null };
    }
    parseTag(arg0, arg1) {
      self = this;
      clonePositionResult = this.clonePosition();
      bumpResult = this.bump();
      parseTagNameResult = this.parseTagName();
      bumpSpaceResult = this.bumpSpace();
      if (this.bumpIf("/>")) {
        obj1 = { val: null, err: null };
        obj12 = { type: null, value: null, location: null };
        tmp26 = closure_1_0;
        tmp27 = closure_1_1;
        obj12.type = closure_1_0(closure_1_1[1]).TYPE.literal;
        str3 = "<";
        concat = "<".concat;
        obj12.value = "<".concat(parseTagNameResult, "/>");
        obj13 = { start: null, end: null };
        obj13.start = clonePositionResult;
        obj13.end = self.clonePosition();
        obj12.location = obj13;
        obj1.val = obj12;
        return obj1;
      } else {
        str = ">";
        if (self.bumpIf(">")) {
          tmp7 = arg0;
          tmp8 = arg1;
          num = 1;
          flag = true;
          parseMessageResult = self.parseMessage(arg0 + 1, arg1, true);
          if (parseMessageResult.err) {
            return parseMessageResult;
          } else {
            clonePositionResult1 = self.clonePosition();
            str2 = "</";
            if (self.bumpIf("</")) {
              if (!self.isEOF()) {
                charResult = self.char();
                num2 = 97;
                tmp14 = charResult >= 97;
                if (tmp14) {
                  num3 = 122;
                  tmp14 = charResult <= 122;
                }
                if (!tmp14) {
                  num4 = 65;
                  tmp15 = charResult >= 65;
                  if (tmp15) {
                    num5 = 90;
                    tmp15 = charResult <= 90;
                  }
                  tmp14 = tmp15;
                }
                if (tmp14) {
                  clonePositionResult2 = self.clonePosition();
                  if (parseTagNameResult !== self.parseTagName()) {
                    tmp22 = closure_1_0;
                    tmp23 = closure_1_1;
                    obj14 = { start: null, end: null };
                    obj14.start = clonePositionResult2;
                    obj14.end = self.clonePosition();
                    errorResult = self.error(closure_1_0(closure_1_1[2]).ErrorKind.UNMATCHED_CLOSING_TAG, obj14);
                  } else {
                    bumpSpaceResult1 = self.bumpSpace();
                    if (self.bumpIf(">")) {
                      obj15 = { val: null, err: null };
                      element = { type: null, value: null, children: null, location: null };
                      tmp20 = closure_1_0;
                      tmp21 = closure_1_1;
                      element.type = closure_1_0(closure_1_1[1]).TYPE.tag;
                      element.value = parseTagNameResult;
                      element.children = parseMessageResult.val;
                      obj16 = { start: null, end: null };
                      obj16.start = clonePositionResult;
                      obj16.end = self.clonePosition();
                      element.location = obj16;
                      obj15.val = element;
                      errorResult = obj15;
                    } else {
                      tmp17 = closure_1_0;
                      tmp18 = closure_1_1;
                      obj17 = { start: null, end: null };
                      obj17.start = clonePositionResult1;
                      obj17.end = self.clonePosition();
                      errorResult = self.error(closure_1_0(closure_1_1[2]).ErrorKind.INVALID_TAG, obj17);
                    }
                  }
                  return errorResult;
                }
              }
              tmp24 = closure_1_0;
              tmp25 = closure_1_1;
              obj18 = { start: null, end: null };
              obj18.start = clonePositionResult1;
              obj18.end = self.clonePosition();
              return self.error(closure_1_0(closure_1_1[2]).ErrorKind.INVALID_TAG, obj18);
            } else {
              tmp11 = closure_1_0;
              tmp12 = closure_1_1;
              obj19 = { start: null, end: null };
              obj19.start = clonePositionResult;
              obj19.end = self.clonePosition();
              return self.error(closure_1_0(closure_1_1[2]).ErrorKind.UNCLOSED_TAG, obj19);
            }
          }
        } else {
          tmp5 = closure_1_0;
          tmp6 = closure_1_1;
          obj = { start: null, end: null };
          obj.start = clonePositionResult;
          obj.end = self.clonePosition();
          return self.error(closure_1_0(closure_1_1[2]).ErrorKind.INVALID_TAG, obj);
        }
      }
    }
    parseTagName() {
      self = this;
      offsetResult = this.offset();
      bumpResult = this.bump();
      if (!this.isEOF()) {
        charResult = self.char();
        num = 45;
        if (45 !== charResult) {
          num63 = 46;
          if (46 !== charResult) {
            num64 = 48;
            if (charResult < 48) {
              num3 = 95;
              if (95 !== charResult) {
                num65 = 97;
                if (charResult < 97) {
                  num5 = 65;
                  if (charResult < 65) {
                    num7 = 183;
                    if (183 != charResult) {
                      num66 = 192;
                      if (charResult < 192) {
                        num9 = 216;
                        if (charResult < 216) {
                          num11 = 248;
                          if (charResult < 248) {
                            num13 = 895;
                            if (charResult < 895) {
                              num15 = 8204;
                              if (charResult < 8204) {
                                num17 = 8255;
                                if (charResult < 8255) {
                                  num19 = 8304;
                                  if (charResult < 8304) {
                                    num21 = 11264;
                                    if (charResult < 11264) {
                                      num23 = 12289;
                                      if (charResult < 12289) {
                                        num25 = 63744;
                                        if (charResult < 63744) {
                                          num27 = 65008;
                                          if (charResult < 65008) {
                                            num29 = 65536;
                                            if (charResult >= 65536) {
                                              num67 = 983039;
                                            }
                                          } else {
                                            num28 = 65533;
                                          }
                                        } else {
                                          num26 = 64975;
                                        }
                                      } else {
                                        num24 = 55295;
                                      }
                                    } else {
                                      num22 = 12271;
                                    }
                                  } else {
                                    num20 = 8591;
                                  }
                                } else {
                                  num18 = 8256;
                                }
                              } else {
                                num16 = 8205;
                              }
                            } else {
                              num14 = 8191;
                            }
                          } else {
                            num12 = 893;
                          }
                        } else {
                          num10 = 246;
                        }
                      } else {
                        num8 = 214;
                      }
                    }
                  } else {
                    num6 = 90;
                  }
                } else {
                  num4 = 122;
                }
              }
            } else {
              num2 = 57;
            }
          }
        }
        bumpResult1 = self.bump();
        while (!self.isEOF()) {
          charResult1 = self.char();
          if (45 === charResult1) {
            continue;
          } else {
            num30 = 46;
            if (46 === charResult1) {
              continue;
            } else {
              num31 = 48;
              if (charResult1 < 48) {
                num33 = 95;
                if (95 === charResult1) {
                  continue;
                } else {
                  num34 = 97;
                  if (charResult1 < 97) {
                    num36 = 65;
                    if (charResult1 < 65) {
                      num38 = 183;
                      if (183 == charResult1) {
                        continue;
                      } else {
                        num39 = 192;
                        if (charResult1 < 192) {
                          num41 = 216;
                          if (charResult1 < 216) {
                            num43 = 248;
                            if (charResult1 < 248) {
                              num45 = 895;
                              if (charResult1 < 895) {
                                num47 = 8204;
                                if (charResult1 < 8204) {
                                  num49 = 8255;
                                  if (charResult1 < 8255) {
                                    num51 = 8304;
                                    if (charResult1 < 8304) {
                                      num53 = 11264;
                                      if (charResult1 < 11264) {
                                        num55 = 12289;
                                        if (charResult1 < 12289) {
                                          num57 = 63744;
                                          if (charResult1 < 63744) {
                                            num59 = 65008;
                                            if (charResult1 < 65008) {
                                              num61 = 65536;
                                              if (charResult1 < 65536) {
                                                break;
                                              } else {
                                                num62 = 983039;
                                                if (charResult1 > 983039) {
                                                  break;
                                                }
                                              }
                                            } else {
                                              num60 = 65533;
                                            }
                                            continue;
                                          } else {
                                            num58 = 64975;
                                          }
                                          continue;
                                        } else {
                                          num56 = 55295;
                                        }
                                        continue;
                                      } else {
                                        num54 = 12271;
                                      }
                                      continue;
                                    } else {
                                      num52 = 8591;
                                    }
                                    continue;
                                  } else {
                                    num50 = 8256;
                                  }
                                  continue;
                                } else {
                                  num48 = 8205;
                                }
                                continue;
                              } else {
                                num46 = 8191;
                              }
                              continue;
                            } else {
                              num44 = 893;
                            }
                            continue;
                          } else {
                            num42 = 246;
                          }
                          continue;
                        } else {
                          num40 = 214;
                        }
                        continue;
                      }
                      continue;
                    } else {
                      num37 = 90;
                    }
                    continue;
                  } else {
                    num35 = 122;
                  }
                  continue;
                }
                continue;
              } else {
                num32 = 57;
              }
              continue;
            }
            continue;
          }
          continue;
        }
      }
      message = self.message;
      return message.slice(offsetResult, self.offset());
    }
    parseLiteral(arg0, arg1) {
      self = this;
      str = "";
      clonePositionResult = this.clonePosition();
      while (true) {
        tryParseQuoteResult = self.tryParseQuote(arg1);
        tmp3 = str;
        if (tryParseQuoteResult) {
          str = `${tmp2}`;
          continue;
        } else {
          tryParseUnquotedResult = self.tryParseUnquoted(arg0, arg1);
          if (tryParseUnquotedResult) {
            str = `${tmp2}${tmp4}`;
            continue;
          } else {
            result = self.tryParseLeftAngleBracket();
            if (!result) {
              break;
            } else {
              str = `${tmp2}${tmp4}${tmp5}`;
              continue;
            }
          }
          continue;
        }
      }
      obj = { start: clonePositionResult, end: self.clonePosition() };
      obj1 = { val: null, err: null };
      obj4 = { type: closure_1_0(closure_1_1[1]).TYPE.literal, value: str, location: obj };
      obj1.val = obj4;
      return obj1;
    }
    tryParseLeftAngleBracket() {
      self = this;
      tmp = null;
      if (!this.isEOF()) {
        num = 60;
        tmp = null;
        if (60 === self.char()) {
          if (self.ignoreTag) {
            bumpResult = self.bump();
            tmp = "<";
          } else {
            tmp2 = self.peek() || 0;
            num2 = 97;
            tmp3 = tmp2 >= 97;
            if (tmp3) {
              num3 = 122;
              tmp3 = tmp2 <= 122;
            }
            if (!tmp3) {
              num4 = 65;
              tmp4 = tmp2 >= 65;
              if (tmp4) {
                num5 = 90;
                tmp4 = tmp2 <= 90;
              }
              tmp3 = tmp4;
            }
            if (!tmp3) {
              num6 = 47;
              tmp3 = 47 === tmp2;
            }
            tmp = null;
          }
        }
      }
      return tmp;
    }
    tryParseQuote(arg0) {
      self = this;
      if (!this.isEOF()) {
        num = 39;
        if (39 === self.char()) {
          peekResult = self.peek();
          if (39 === peekResult) {
            bumpResult = self.bump();
            bumpResult1 = self.bump();
            str3 = "'";
            return "'";
          } else {
            num6 = 123;
            if (123 !== peekResult) {
              num2 = 60;
              if (60 !== peekResult) {
                num3 = 62;
                if (62 !== peekResult) {
                  num4 = 125;
                  if (125 !== peekResult) {
                    num5 = 35;
                    if (35 === peekResult) {
                      tmp2 = arg0;
                      str = "plural";
                      if ("plural" !== arg0) {
                        str2 = "selectordinal";
                        if ("selectordinal" !== arg0) {
                          tmp3 = null;
                          return null;
                        }
                      }
                    } else {
                      tmp = null;
                      return null;
                    }
                  }
                }
              }
            }
            bumpResult2 = self.bump();
            items = [];
            items[0] = self.char();
            bumpResult3 = self.bump();
            if (!self.isEOF()) {
              while (true) {
                charResult = self.char();
                if (39 === charResult) {
                  if (39 !== self.peek()) {
                    break;
                  } else {
                    arr1 = items.push(39);
                    bumpResult4 = self.bump();
                  }
                } else {
                  arr2 = items.push(charResult);
                }
                bumpResult5 = self.bump();
              }
              bumpResult6 = self.bump();
            }
            tmp12 = closure_1_7;
            return closure_1_7.apply(undefined, items);
          }
        }
      }
      return null;
    }
    tryParseUnquoted(arg0, arg1) {
      self = this;
      if (this.isEOF()) {
        return null;
      } else {
        charResult = self.char();
        num = 60;
        tmp2 = null;
        if (60 !== charResult) {
          num2 = 123;
          tmp2 = null;
          if (123 !== charResult) {
            num3 = 35;
            if (35 !== charResult) {
              num4 = 125;
              if (125 !== charResult) {
                bumpResult = self.bump();
                tmp6 = closure_1_7;
                tmp2 = closure_1_7(charResult);
              } else {
                tmp4 = arg0;
                num5 = 0;
                tmp2 = null;
              }
            } else {
              tmp3 = arg1;
              str = "plural";
              tmp2 = null;
              if ("plural" !== arg1) {
                str2 = "selectordinal";
                tmp2 = null;
              }
            }
          }
        }
        return tmp2;
      }
    }
    parseArgument(arg0, arg1) {
      self = this;
      clonePositionResult = this.clonePosition();
      bumpResult = this.bump();
      bumpSpaceResult = this.bumpSpace();
      if (this.isEOF()) {
        tmp28 = closure_1_0;
        tmp29 = closure_1_1;
        obj1 = { start: null, end: null };
        obj1.start = clonePositionResult;
        obj1.end = self.clonePosition();
        return self.error(closure_1_0(closure_1_1[2]).ErrorKind.EXPECT_ARGUMENT_CLOSING_BRACE, obj1);
      } else {
        num = 125;
        if (125 === self.char()) {
          bumpResult1 = self.bump();
          tmp26 = closure_1_0;
          tmp27 = closure_1_1;
          obj10 = { start: null, end: null };
          obj10.start = clonePositionResult;
          obj10.end = self.clonePosition();
          return self.error(closure_1_0(closure_1_1[2]).ErrorKind.EMPTY_ARGUMENT, obj10);
        } else {
          value = self.parseIdentifierIfPossible().value;
          if (value) {
            bumpSpaceResult1 = self.bumpSpace();
            if (self.isEOF()) {
              tmp23 = closure_1_0;
              tmp24 = closure_1_1;
              obj11 = { start: null, end: null };
              obj11.start = clonePositionResult;
              obj11.end = self.clonePosition();
              return self.error(closure_1_0(closure_1_1[2]).ErrorKind.EXPECT_ARGUMENT_CLOSING_BRACE, obj11);
            } else {
              charResult = self.char();
              if (125 === charResult) {
                bumpResult2 = self.bump();
                obj12 = { val: null, err: null };
                obj13 = { type: null, value: null, location: null };
                tmp21 = closure_1_0;
                tmp22 = closure_1_1;
                obj13.type = closure_1_0(closure_1_1[1]).TYPE.argument;
                obj13.value = value;
                obj14 = { start: null, end: null };
                obj14.start = clonePositionResult;
                obj14.end = self.clonePosition();
                obj13.location = obj14;
                obj12.val = obj13;
                return obj12;
              } else {
                num2 = 44;
                if (44 === charResult) {
                  bumpResult3 = self.bump();
                  bumpSpaceResult2 = self.bumpSpace();
                  if (self.isEOF()) {
                    tmp18 = closure_1_0;
                    tmp19 = closure_1_1;
                    obj15 = { start: null, end: null };
                    obj15.start = clonePositionResult;
                    obj15.end = self.clonePosition();
                    errorResult = self.error(
                      closure_1_0(closure_1_1[2]).ErrorKind.EXPECT_ARGUMENT_CLOSING_BRACE,
                      obj15,
                    );
                  } else {
                    tmp12 = arg0;
                    tmp13 = arg1;
                    tmp14 = self;
                    tmp15 = value;
                    tmp16 = clonePositionResult;
                    errorResult = self.parseArgumentOptions(arg0, arg1, value, clonePositionResult);
                  }
                  return errorResult;
                } else {
                  tmp8 = closure_1_0;
                  tmp9 = closure_1_1;
                  obj16 = { start: null, end: null };
                  obj16.start = clonePositionResult;
                  obj16.end = self.clonePosition();
                  return self.error(closure_1_0(closure_1_1[2]).ErrorKind.MALFORMED_ARGUMENT, obj16);
                }
              }
            }
          } else {
            tmp4 = closure_1_0;
            tmp5 = closure_1_1;
            obj = { start: null, end: null };
            obj.start = clonePositionResult;
            obj.end = self.clonePosition();
            return self.error(closure_1_0(closure_1_1[2]).ErrorKind.MALFORMED_ARGUMENT, obj);
          }
        }
      }
    }
    parseIdentifierIfPossible() {
      clonePositionResult = this.clonePosition();
      offsetResult = this.offset();
      arr = closure_1_5(this.message, offsetResult);
      bumpToResult = this.bumpTo(offsetResult + arr.length);
      obj = { value: arr, location: null };
      obj1 = { start: clonePositionResult, end: this.clonePosition() };
      obj.location = obj1;
      return obj;
    }
    parseArgumentOptions(arg0, arg1, arg2, arg3) {
      self = this;
      clonePositionResult = this.clonePosition();
      value = this.parseIdentifierIfPossible().value;
      clonePositionResult1 = this.clonePosition();
      if ("" === value) {
        tmp65 = closure_1_0;
        tmp66 = closure_1_1;
        obj1 = { start: null, end: null };
        obj1.start = clonePositionResult;
        obj1.end = clonePositionResult1;
        return self.error(closure_1_0(closure_1_1[2]).ErrorKind.EXPECT_ARGUMENT_TYPE, obj1);
      } else {
        tmp67 = arg2;
        tmp68 = arg3;
        str10 = "number";
        if ("number" !== value) {
          str11 = "date";
          if ("date" !== value) {
            str12 = "time";
            if ("time" !== value) {
              str13 = "plural";
              if ("plural" !== value) {
                str = "selectordinal";
                if ("selectordinal" !== value) {
                  str2 = "select";
                  if ("select" !== value) {
                    tmp3 = closure_1_0;
                    tmp4 = closure_1_1;
                    obj = { start: null, end: null };
                    obj.start = clonePositionResult;
                    obj.end = clonePositionResult1;
                    return self.error(closure_1_0(closure_1_1[2]).ErrorKind.INVALID_ARGUMENT_TYPE, obj);
                  }
                }
              }
              clonePositionResult2 = self.clonePosition();
              bumpSpaceResult = self.bumpSpace();
              str3 = ",";
              if (self.bumpIf(",")) {
                bumpSpaceResult1 = self.bumpSpace();
                iter = self.parseIdentifierIfPossible();
                str4 = "select";
                num = 0;
                result1 = iter;
                if ("select" !== value) {
                  str14 = "offset";
                  num = 0;
                  result1 = iter;
                  if ("offset" === iter.value) {
                    str15 = ":";
                    if (self.bumpIf(":")) {
                      bumpSpaceResult2 = self.bumpSpace();
                      tmp15 = closure_1_0;
                      tmp16 = closure_1_1;
                      result = self.tryParseDecimalInteger(
                        closure_1_0(closure_1_1[2]).ErrorKind.EXPECT_PLURAL_ARGUMENT_OFFSET_VALUE,
                        closure_1_0(closure_1_1[2]).ErrorKind.INVALID_PLURAL_ARGUMENT_OFFSET_VALUE,
                      );
                      if (result.err) {
                        return result;
                      } else {
                        bumpSpaceResult3 = self.bumpSpace();
                        result1 = self.parseIdentifierIfPossible();
                        num = result.val;
                      }
                    } else {
                      tmp11 = closure_1_0;
                      tmp12 = closure_1_1;
                      clonePositionResult3 = self.clonePosition();
                      obj23 = { start: null, end: null };
                      obj23.start = clonePositionResult3;
                      obj23.end = self.clonePosition();
                      return self.error(
                        closure_1_0(closure_1_1[2]).ErrorKind.EXPECT_PLURAL_ARGUMENT_OFFSET_VALUE,
                        obj23,
                      );
                    }
                  }
                }
                tmp19 = arg0;
                tmp20 = arg1;
                tmp21 = self;
                tmp22 = value;
                tmp23 = result1;
                result2 = self.tryParsePluralOrSelectOptions(arg0, value, arg1, result1);
                if (result2.err) {
                  return result2;
                } else {
                  result3 = self.tryParseArgumentClose(arg3);
                  if (result3.err) {
                    return result3;
                  } else {
                    obj24 = { start: null, end: null };
                    obj24.start = arg3;
                    obj24.end = self.clonePosition();
                    if ("select" === value) {
                      obj25 = { val: null, err: null };
                      obj26 = { type: null, value: null, options: null, location: null };
                      tmp29 = closure_1_0;
                      tmp30 = closure_1_1;
                      obj26.type = closure_1_0(closure_1_1[1]).TYPE.select;
                      obj26.value = arg2;
                      tmp31 = closure_1_8;
                      obj26.options = closure_1_8(result2.val);
                      obj26.location = obj24;
                      obj25.val = obj26;
                      obj28 = obj25;
                    } else {
                      obj27 = {
                        type: null,
                        value: null,
                        options: null,
                        offset: null,
                        pluralType: null,
                        location: null,
                      };
                      tmp26 = closure_1_0;
                      tmp27 = closure_1_1;
                      obj27.type = closure_1_0(closure_1_1[1]).TYPE.plural;
                      obj27.value = arg2;
                      tmp28 = closure_1_8;
                      obj27.options = closure_1_8(result2.val);
                      obj27.offset = num;
                      str5 = "ordinal";
                      if ("plural" === value) {
                        str5 = "cardinal";
                      }
                      obj28 = { val: null, err: null };
                      obj27.pluralType = str5;
                      obj27.location = obj24;
                      obj28.val = obj27;
                    }
                    return obj28;
                  }
                }
              } else {
                tmp7 = closure_1_0;
                tmp8 = closure_1_1;
                obj2 = closure_1_0(closure_1_1[5]);
                obj29 = { start: null, end: null };
                obj29.start = clonePositionResult2;
                obj29.end = obj2.__assign({}, clonePositionResult2);
                return self.error(closure_1_0(closure_1_1[2]).ErrorKind.EXPECT_SELECT_ARGUMENT_OPTIONS, obj29);
              }
            }
          }
        }
        bumpSpaceResult4 = self.bumpSpace();
        str6 = ",";
        tmp33 = null;
        tmp34 = null;
        if (self.bumpIf(",")) {
          bumpSpaceResult5 = self.bumpSpace();
          clonePositionResult4 = self.clonePosition();
          result4 = self.parseSimpleArgStyleIfPossible();
          if (result4.err) {
            return result4;
          } else {
            tmp38 = closure_1_11;
            arr = closure_1_11(result4.val);
            num2 = 0;
            if (0 === arr.length) {
              tmp62 = closure_1_0;
              tmp63 = closure_1_1;
              clonePositionResult5 = self.clonePosition();
              obj30 = { start: null, end: null };
              obj30.start = clonePositionResult5;
              obj30.end = self.clonePosition();
              return self.error(closure_1_0(closure_1_1[2]).ErrorKind.EXPECT_ARGUMENT_STYLE, obj30);
            } else {
              obj31 = { style: null, styleLocation: null };
              obj31.style = arr;
              obj32 = { start: null, end: null };
              obj32.start = clonePositionResult4;
              obj32.end = self.clonePosition();
              obj31.styleLocation = obj32;
              tmp34 = obj31;
            }
          }
        }
        result5 = self.tryParseArgumentClose(arg3);
        if (result5.err) {
          return result5;
        } else {
          obj33 = { start: null, end: null };
          obj33.start = arg3;
          obj33.end = self.clonePosition();
          if (tmp34) {
            style = undefined;
            tmp40 = closure_1_6;
            if (null != tmp34) {
              style = tmp34.style;
            }
            num3 = 0;
            str7 = "::";
            if (tmp40(style, "::", 0)) {
              tmp50 = closure_1_10;
              style1 = tmp34.style;
              num4 = 2;
              arr3 = closure_1_10(style1.slice(2));
              if ("number" === value) {
                result6 = self.parseNumberSkeletonFromString(arr3, tmp34.styleLocation);
                tmp59 = result6;
                if (!result6.err) {
                  obj34 = { val: null, err: null };
                  obj35 = { type: null, value: null, location: null, style: null };
                  tmp60 = closure_1_0;
                  tmp61 = closure_1_1;
                  obj35.type = closure_1_0(closure_1_1[1]).TYPE.number;
                  obj35.value = arg2;
                  obj35.location = obj33;
                  obj35.style = result6.val;
                  obj34.val = obj35;
                  tmp59 = obj34;
                }
                return tmp59;
              } else if (0 === arr3.length) {
                tmp56 = closure_1_0;
                tmp57 = closure_1_1;
                return self.error(closure_1_0(closure_1_1[2]).ErrorKind.EXPECT_DATE_TIME_SKELETON, obj33);
              } else {
                bestPattern = arr3;
                if (self.locale) {
                  tmp51 = closure_1_0;
                  tmp52 = closure_1_1;
                  bestPattern = closure_1_0(closure_1_1[3]).getBestPattern(arr3, self.locale);
                }
                obj36 = { type: null, pattern: null, location: null, parsedOptions: null };
                tmp54 = closure_1_0;
                tmp55 = closure_1_1;
                obj36.type = closure_1_0(closure_1_1[1]).SKELETON_TYPE.dateTime;
                obj36.pattern = bestPattern;
                obj36.location = tmp34.styleLocation;
                if (self.shouldParseSkeletons) {
                  result7 = tmp54(tmp55[4]).parseDateTimeSkeleton(bestPattern);
                } else {
                  result7 = {};
                }
                obj36.parsedOptions = result7;
                str9 = "date";
                if ("date" === value) {
                  time2 = tmp54(tmp55[1]).TYPE.date;
                } else {
                  time2 = tmp54(tmp55[1]).TYPE.time;
                }
                obj37 = { val: null, err: null };
                obj38 = { type: null, value: null, location: null, style: null };
                obj38.type = time2;
                obj38.value = arg2;
                obj38.location = obj33;
                obj38.style = obj36;
                obj37.val = obj38;
                return obj37;
              }
            }
          }
          if ("number" === value) {
            tmp46 = closure_1_0;
            tmp47 = closure_1_1;
            time = closure_1_0(closure_1_1[1]).TYPE.number;
          } else {
            str8 = "date";
            if ("date" === value) {
              tmp44 = closure_1_0;
              tmp45 = closure_1_1;
              time = closure_1_0(closure_1_1[1]).TYPE.date;
            } else {
              tmp42 = closure_1_0;
              tmp43 = closure_1_1;
              time = closure_1_0(closure_1_1[1]).TYPE.time;
            }
          }
          obj39 = { type: null, value: null, location: null, style: null };
          obj39.type = time;
          obj39.value = arg2;
          obj39.location = obj33;
          style2 = undefined;
          if (null != tmp34) {
            style2 = tmp34.style;
          }
          tmp49 = null;
          if (null !== style2) {
            tmp49 = null;
            if (undefined !== style2) {
              tmp49 = style2;
            }
          }
          obj40 = { val: null, err: null };
          obj39.style = tmp49;
          obj40.val = obj39;
          return obj40;
        }
      }
    }
    tryParseArgumentClose(arg0) {
      self = this;
      if (!this.isEOF()) {
        num = 125;
        if (125 === self.char()) {
          bumpResult = self.bump();
          errorResult = { val: true, err: null };
        }
        return errorResult;
      }
      obj1 = { start: arg0, end: self.clonePosition() };
      errorResult = self.error(closure_1_0(closure_1_1[2]).ErrorKind.EXPECT_ARGUMENT_CLOSING_BRACE, obj1);
      return;
    }
    parseSimpleArgStyleIfPossible() {
      self = this;
      clonePositionResult = this.clonePosition();
      num = 0;
      if (!this.isEOF()) {
        while (true) {
          charResult = self.char();
          tmp3 = num;
          if (39 === charResult) {
            bumpResult = self.bump();
            clonePositionResult1 = self.clonePosition();
            if (self.bumpUntil("'")) {
              bumpResult1 = self.bump();
              sum = num;
            } else {
              tmp9 = closure_1_0;
              tmp10 = closure_1_1;
              obj1 = { start: null, end: null };
              obj1.start = clonePositionResult1;
              obj1.end = self.clonePosition();
              return self.error(closure_1_0(closure_1_1[2]).ErrorKind.UNCLOSED_QUOTE_IN_ARGUMENT_STYLE, obj1);
            }
          } else if (123 === charResult) {
            sum = num + 1;
            bumpResult2 = self.bump();
          } else if (125 === charResult) {
            if (0 >= num) {
              break;
            } else {
              sum = num - 1;
            }
          } else {
            bumpResult3 = self.bump();
            sum = num;
          }
          num = sum;
        }
        obj = { val: null, err: null };
        message = self.message;
        obj.val = message.slice(clonePositionResult.offset, self.offset());
        return obj;
      }
      obj4 = { val: null, err: null };
      message1 = self.message;
      obj4.val = message1.slice(clonePositionResult.offset, self.offset());
      return obj4;
    }
    parseNumberSkeletonFromString(arg0, arg1) {
      try {
        tmp3 = arg0;
        tmp4 = closure_1_0;
        tmp5 = closure_1_0;
        tmp6 = closure_1_1;
        tmp7 = closure_1_1;
        result = closure_1_0(closure_1_1[4]).parseNumberSkeletonFromString(arg0);
        obj1 = { type: null, tokens: null, location: null, parsedOptions: null };
        tmp9 = tmp4;
        tmp10 = tmp6;
        obj1.type = tmp5(tmp7[1]).SKELETON_TYPE.number;
        tmp11 = result;
        obj1.tokens = result;
        obj1.location = arg1;
        if (tmp2.shouldParseSkeletons) {
          tmp12 = tmp4;
          tmp13 = tmp6;
          parseNumberSkeletonResult = tmp5(tmp7[4]).parseNumberSkeleton(result);
        } else {
          parseNumberSkeletonResult = {};
        }
        obj5 = { val: null, err: null };
        obj1.parsedOptions = parseNumberSkeletonResult;
        obj5.val = obj1;
        return obj5;
      } catch (err) {
        tmp14 = closure_1_0;
        tmp15 = closure_1_0;
        tmp16 = closure_1_1;
        tmp17 = closure_1_1;
        return obj.error(closure_1_0(closure_1_1[2]).ErrorKind.INVALID_NUMBER_SKELETON, tmp);
      }
      return;
    }
    tryParsePluralOrSelectOptions(arg0, arg1, arg2, arg3) {
      self = this;
      items = [];
      set = new Set();
      ({ value, location: _location } = arg3);
      tmp = "select" === arg1;
      flag = false;
      while (true) {
        tmp2 = _location;
        substr = value;
        tmp4 = flag;
        if (0 === value.length) {
          clonePositionResult = self.clonePosition();
          if (tmp) {
            break;
          } else if (!self.bumpIf("=")) {
            break;
          } else {
            tmp5 = closure_1_0;
            tmp6 = closure_1_1;
            result = self.tryParseDecimalInteger(
              closure_1_0(closure_1_1[2]).ErrorKind.EXPECT_PLURAL_ARGUMENT_SELECTOR,
              closure_1_0(closure_1_1[2]).ErrorKind.INVALID_PLURAL_ARGUMENT_SELECTOR,
            );
            if (result.err) {
              return result;
            } else {
              obj1 = { start: null, end: null };
              obj1.start = clonePositionResult;
              obj1.end = self.clonePosition();
              message = self.message;
              substr = message.slice(clonePositionResult.offset, self.offset());
              tmp2 = obj1;
            }
          }
        }
        tmp8 = substr;
        if (set.has(substr)) {
          if ("select" === arg1) {
            tmp24 = closure_1_0;
            tmp25 = closure_1_1;
            DUPLICATE_PLURAL_ARGUMENT_SELECTOR = closure_1_0(closure_1_1[2]).ErrorKind
              .DUPLICATE_SELECT_ARGUMENT_SELECTOR;
          } else {
            tmp22 = closure_1_0;
            tmp23 = closure_1_1;
            DUPLICATE_PLURAL_ARGUMENT_SELECTOR = closure_1_0(closure_1_1[2]).ErrorKind
              .DUPLICATE_PLURAL_ARGUMENT_SELECTOR;
          }
          return self.error(DUPLICATE_PLURAL_ARGUMENT_SELECTOR, tmp2);
        } else {
          flag2 = flag;
          if ("other" === substr) {
            flag2 = true;
          }
          bumpSpaceResult = self.bumpSpace();
          clonePositionResult1 = self.clonePosition();
          if (self.bumpIf("{")) {
            parseMessageResult = self.parseMessage(arg0 + 1, arg1, arg2);
            if (parseMessageResult.err) {
              return parseMessageResult;
            } else {
              result1 = self.tryParseArgumentClose(clonePositionResult1);
              if (result1.err) {
                return result1;
              } else {
                items1 = [,];
                items1[0] = substr;
                obj9 = { value: null, location: null };
                obj9.value = parseMessageResult.val;
                obj10 = { start: null, end: null };
                obj10.start = clonePositionResult1;
                obj10.end = self.clonePosition();
                obj9.location = obj10;
                items1[1] = obj9;
                arr1 = items.push(items1);
                addResult = set.add(substr);
                bumpSpaceResult1 = self.bumpSpace();
                result2 = self.parseIdentifierIfPossible();
                ({ value, location: _location } = result2);
                flag = flag2;
                continue;
              }
            }
          } else {
            if ("select" === arg1) {
              tmp13 = closure_1_0;
              tmp14 = closure_1_1;
              EXPECT_PLURAL_ARGUMENT_SELECTOR_FRAGMENT = closure_1_0(closure_1_1[2]).ErrorKind
                .EXPECT_SELECT_ARGUMENT_SELECTOR_FRAGMENT;
            } else {
              tmp11 = closure_1_0;
              tmp12 = closure_1_1;
              EXPECT_PLURAL_ARGUMENT_SELECTOR_FRAGMENT = closure_1_0(closure_1_1[2]).ErrorKind
                .EXPECT_PLURAL_ARGUMENT_SELECTOR_FRAGMENT;
            }
            clonePositionResult2 = self.clonePosition();
            obj11 = { start: null, end: null };
            obj11.start = clonePositionResult2;
            obj11.end = self.clonePosition();
            return self.error(EXPECT_PLURAL_ARGUMENT_SELECTOR_FRAGMENT, obj11);
          }
        }
      }
      if (0 === items.length) {
        tmp30 = closure_1_0;
        tmp31 = closure_1_1;
        ErrorKind = closure_1_0(closure_1_1[2]).ErrorKind;
        tmp32 = tmp ? ErrorKind.EXPECT_SELECT_ARGUMENT_SELECTOR : ErrorKind.EXPECT_PLURAL_ARGUMENT_SELECTOR;
        clonePositionResult3 = self.clonePosition();
        ErrorKind = { start: null, end: null };
        ErrorKind.start = clonePositionResult3;
        ErrorKind.end = self.clonePosition();
        errorResult = self.error(tmp32, ErrorKind);
      } else {
        if (self.requiresOtherClause) {
          if (!flag) {
            tmp26 = closure_1_0;
            tmp27 = closure_1_1;
            clonePositionResult4 = self.clonePosition();
            obj12 = { start: null, end: null };
            obj12.start = clonePositionResult4;
            obj12.end = self.clonePosition();
            errorResult1 = self.error(closure_1_0(closure_1_1[2]).ErrorKind.MISSING_OTHER_CLAUSE, obj12);
          }
          return errorResult1;
        }
        obj13 = { val: null, err: null };
        obj13.val = items;
        errorResult1 = obj13;
      }
      return;
    }
    tryParseDecimalInteger(arg0, arg1) {
      self = this;
      clonePositionResult = this.clonePosition();
      num = 1;
      num2 = 1;
      if (!this.bumpIf("+")) {
        str = "-";
        if (self.bumpIf("-")) {
          num = -1;
        }
        num2 = num;
      }
      num3 = 0;
      flag = false;
      num4 = 0;
      flag2 = false;
      if (!self.isEOF()) {
        charResult = self.char();
        tmp3 = num3;
        tmp4 = flag;
        num4 = num3;
        flag2 = flag;
        while (charResult >= 48) {
          num4 = num3;
          flag2 = flag;
          if (charResult > 57) {
            break;
          } else {
            num3 = 10 * num3 + (charResult - 48);
            bumpResult = self.bump();
            flag = true;
            num4 = num3;
            flag2 = true;
            if (self.isEOF()) {
              break;
            }
          }
        }
      }
      obj = { start: clonePositionResult, end: self.clonePosition() };
      if (flag2) {
        tmp7 = closure_1_4;
        result = num4 * num2;
        if (closure_1_4(result)) {
          obj1 = { val: null, err: null };
          obj1.val = result;
          errorResult = obj1;
        } else {
          tmp9 = arg1;
          errorResult = self.error(arg1, obj);
        }
        tmp11 = errorResult;
      } else {
        tmp6 = arg0;
        return self.error(arg0, obj);
      }
      return;
    }
    offset() {
      return this.position.offset;
    }
    isEOF() {
      return this.offset() === this.message.length;
    }
    clonePosition() {
      obj = { offset: this.position.offset, line: this.position.line, column: this.position.column };
      return obj;
    }
    char() {
      offset = this.position.offset;
      if (offset >= this.message.length) {
        tmp5 = globalThis;
        _Error2 = Error;
        str3 = "out of bound";
        throw Error("out of bound");
      } else {
        tmp2 = closure_1_9;
        tmp3 = closure_1_9(tmp.message, offset);
        if (undefined === tmp3) {
          tmp4 = globalThis;
          _Error = Error;
          str = "Offset ";
          concat = "Offset ".concat;
          str2 = " is at invalid UTF-16 code unit boundary";
          throw Error("Offset ".concat(offset, " is at invalid UTF-16 code unit boundary"));
        } else {
          return tmp3;
        }
      }
    }
    error(arg0, arg1) {
      obj = { val: null, err: null };
      obj1 = { kind: arg0, message: this.message, location: arg1 };
      obj.err = obj1;
      return obj;
    }
    bump() {
      self = this;
      if (!this.isEOF()) {
        charResult = self.char();
        num = 10;
        if (10 === charResult) {
          position3 = self.position;
          num5 = 1;
          position3.line = position3.line + 1;
          self.position.column = 1;
          position4 = self.position;
          position4.offset = position4.offset + 1;
        } else {
          position = self.position;
          num2 = 1;
          position.column = position.column + 1;
          position2 = self.position;
          num3 = 65536;
          num4 = 2;
          if (charResult < 65536) {
            num4 = 1;
          }
          position2.offset = position2.offset + num4;
        }
      }
      return;
    }
    bumpIf(arg0) {
      self = this;
      if (closure_1_6(this.message, arg0, this.offset())) {
        num = 0;
        num2 = 1;
        if (0 < arg0.length) {
          do {
            bumpResult = self.bump();
            num = num + 1;
            length = arg0.length;
          } while (num < length);
        }
        flag2 = true;
        return true;
      } else {
        flag = false;
        return false;
      }
    }
    bumpUntil(arg0) {
      self = this;
      message = this.message;
      index = message.indexOf(arg0, this.offset());
      if (index >= 0) {
        bumpToResult = self.bumpTo(index);
        flag = true;
      } else {
        bumpToResult1 = self.bumpTo(self.message.length);
        flag = false;
      }
      return flag;
    }
    bumpTo(arg0) {
      self = this;
      if (this.offset() > arg0) {
        tmp5 = globalThis;
        _Error2 = Error;
        str3 = "targetOffset ";
        concat2 = "targetOffset ".concat;
        str4 = " must be greater than or equal to the current offset ";
        combined = "targetOffset ".concat(arg0, " must be greater than or equal to the current offset ");
        throw Error(combined.concat(self.offset()));
      } else {
        tmp = globalThis;
        _Math = Math;
        bound = Math.min(arg0, self.message.length);
        offsetResult = self.offset();
        while (offsetResult !== bound) {
          if (offsetResult > bound) {
            _Error = Error;
            str = "targetOffset ";
            concat = "targetOffset ".concat;
            str2 = " is at invalid UTF-16 code unit boundary";
            throw Error("targetOffset ".concat(bound, " is at invalid UTF-16 code unit boundary"));
          } else {
            bumpResult = self.bump();
            if (self.isEOF()) {
              break;
            }
          }
        }
        return;
      }
    }
    bumpSpace() {
      self = this;
      if (!this.isEOF()) {
        charResult = self.char();
        num = 9;
        tmp2 = charResult >= 9;
        if (tmp2) {
          num2 = 13;
          tmp2 = charResult <= 13;
        }
        if (!tmp2) {
          num3 = 32;
          tmp2 = 32 === charResult;
        }
        if (!tmp2) {
          num4 = 133;
          tmp2 = 133 === charResult;
        }
        if (!tmp2) {
          num5 = 8206;
          tmp3 = charResult >= 8206;
          if (tmp3) {
            num6 = 8207;
            tmp3 = charResult <= 8207;
          }
          tmp2 = tmp3;
        }
        if (!tmp2) {
          num7 = 8232;
          tmp2 = 8232 === charResult;
        }
        if (!tmp2) {
          num8 = 8233;
          tmp2 = 8233 === charResult;
        }
        num9 = 8233;
        num10 = 8232;
        num11 = 8207;
        num12 = 8206;
        num13 = 133;
        num14 = 32;
        num15 = 13;
        if (tmp2) {
          bumpResult = self.bump();
          while (!self.isEOF()) {
            charResult1 = self.char();
            tmp6 = (charResult1 >= 9 && charResult1 <= 13) || 32 === charResult1 || 133 === charResult1;
            if (!tmp6) {
              tmp7 = charResult1 >= 8206 && charResult1 <= 8207;
              tmp6 = tmp7;
            }
            if (!tmp6) {
              tmp6 = 8232 === charResult1;
            }
            if (!tmp6) {
              tmp6 = 8233 === charResult1;
            }
            if (!tmp6) {
              break;
            }
          }
        }
      }
      return;
    }
    peek() {
      self = this;
      if (this.isEOF()) {
        tmp5 = null;
        return null;
      } else {
        charResult = self.char();
        message = self.message;
        num = 65536;
        num2 = 1;
        offsetResult = self.offset();
        if (charResult >= 65536) {
          num2 = 2;
        }
        charCodeAtResult = message.charCodeAt(offsetResult + num2);
        tmp4 = null;
        if (null != charCodeAtResult) {
          tmp4 = charCodeAtResult;
        }
        return tmp4;
      }
    }
  }
  return Parser;
}

function crc32(arg0, arg1, arg2, arg3) {
  let sum1 = arg3;
  const sum = arg3 + arg2;
  let tmp4 = arg0 ^ -1;
  let tmp5 = tmp4;
  if (arg3 < sum) {
    do {
      tmp4 = (tmp4 >>> 8) ^ tmp2[255 & (tmp4 ^ arg1[sum1])];
      sum1 = sum1 + 1;
      tmp5 = tmp4;
    } while (sum1 < sum);
  }
  return ~tmp5;
}

function f63871(str) {
  let sum4;
  let num = 0;
  let num2 = 0;
  let num3 = 0;
  if (0 < str.length) {
    do {
      let charCodeAtResult = str.charCodeAt(num2);
      let tmp4 = 64512 & charCodeAtResult;
      let tmp5 = 55296 === tmp4;
      let tmp8 = tmp;
      if (55296 === tmp4) {
        tmp5 = num2 + 1 < length;
      }
      if (tmp5) {
        let charCodeAtResult1 = str.charCodeAt(num2 + 1);
        tmp5 = 56320 === (64512 & charCodeAtResult1);
        tmp8 = charCodeAtResult1;
      }
      let sum1 = num2;
      let sum = charCodeAtResult;
      if (tmp5) {
        sum = 65536 + ((charCodeAtResult - 55296) << 10) + (tmp8 - 56320);
        sum1 = num2 + 1;
      }
      let num4 = 1;
      if (sum >= 128) {
        let num5 = 2;
        if (sum >= 2048) {
          let num6 = 4;
          if (sum < 65536) {
            num6 = 3;
          }
          num5 = num6;
        }
        num4 = num5;
      }
      num = num + num4;
      num2 = sum1 + 1;
      tmp = tmp8;
      num3 = num;
      let tmp2 = tmp8;
    } while (num2 < length);
  }
  const buf8 = new closure_1_0(closure_1_1[0]).Buf8(num3);
  let num7 = 0;
  let num8 = 0;
  if (0 < num3) {
    do {
      let charCodeAtResult2 = str.charCodeAt(num8);
      let tmp14 = 64512 & charCodeAtResult2;
      let tmp15 = 55296 === tmp14;
      let tmp18 = tmp2;
      if (55296 === tmp14) {
        tmp15 = num8 + 1 < length;
      }
      if (tmp15) {
        let charCodeAtResult3 = str.charCodeAt(num8 + 1);
        tmp15 = 56320 === (64512 & charCodeAtResult3);
        tmp18 = charCodeAtResult3;
      }
      let sum3 = num8;
      let sum2 = charCodeAtResult2;
      if (tmp15) {
        sum2 = 65536 + ((charCodeAtResult2 - 55296) << 10) + (tmp18 - 56320);
        sum3 = num8 + 1;
      }
      if (sum2 < 128) {
        sum4 = num7 + 1;
        buf8[num7] = sum2;
      } else if (sum2 < 2048) {
        let sum5 = num7 + 1;
        buf8[num7] = 192 | (sum2 >>> 6);
        sum4 = sum5 + 1;
        buf8[sum5] = 128 | (63 & sum2);
      } else if (sum2 < 65536) {
        let sum6 = num7 + 1;
        buf8[num7] = 224 | (sum2 >>> 12);
        let sum7 = sum6 + 1;
        buf8[sum6] = 128 | ((sum2 >>> 6) & 63);
        sum4 = sum7 + 1;
        buf8[sum7] = 128 | (63 & sum2);
      } else {
        let sum8 = num7 + 1;
        buf8[num7] = 240 | (sum2 >>> 18);
        let sum9 = sum8 + 1;
        buf8[sum8] = 128 | ((sum2 >>> 12) & 63);
        let sum10 = sum9 + 1;
        buf8[sum9] = 128 | ((sum2 >>> 6) & 63);
        sum4 = sum10 + 1;
        buf8[sum10] = 128 | (63 & sum2);
      }
      num8 = sum3 + 1;
      num7 = sum4;
      tmp2 = tmp18;
    } while (sum4 < num3);
  }
  return buf8;
}

function f63872(subarray) {
  if (subarray.length < 65534) {
    if (!subarray.subarray) {
      if (!subarray.subarray) {
        return str2;
      }
    }
    const _String2 = String;
    str2 = fromCharCode.apply(null, closure_1_0(closure_1_1[0]).shrinkBuf(subarray, length));
    const obj = closure_1_0(closure_1_1[0]);
  }
  let num = 0;
  let str = "";
  str2 = "";
  if (0 < subarray.length) {
    do {
      let _String = String;
      str = `${String.fromCharCode(subarray[num])}`;
      num = num + 1;
      str2 = str;
    } while (num < length);
  }
}

function f63873(str) {
  const buf8 = new closure_1_0(closure_1_1[0]).Buf8(str.length);
  for (let num = 0; num < length; num = num + 1) {
    buf8[num] = str.charCodeAt(num);
  }
  return buf8;
}

function f63874(arg0, arg1) {
  let sum2;
  let length = arg1;
  if (!arg1) {
    length = arg0.length;
  }
  const array = new Array(2 * length);
  let num = 0;
  let num2 = 0;
  let num3 = 0;
  if (0 < length) {
    do {
      let sum = num2 + 1;
      let tmp3 = arg0[num2];
      if (tmp3 < 128) {
        let sum1 = num + 1;
        array[num] = tmp3;
        sum2 = sum;
      } else {
        let tmp25 = closure_1_4[tmp3];
        if (tmp25 > 4) {
          sum1 = num + 1;
          array[num] = 65533;
          sum2 = sum + (tmp25 - 1);
        } else {
          let num5 = 31;
          if (2 !== tmp25) {
            let num4 = 7;
            if (3 === tmp25) {
              num4 = 15;
            }
            num5 = num4;
          }
          let tmp5 = tmp3 & num5;
          let tmp6 = tmp25;
          let tmp7 = tmp5;
          let tmp8 = sum;
          if (tmp25 > 1) {
            let tmp9 = tmp25;
            let tmp10 = tmp5;
            let tmp11 = sum;
            tmp6 = tmp25;
            tmp7 = tmp5;
            tmp8 = sum;
            if (sum < length) {
              let sum3 = tmp11 + 1;
              let tmp13 = (tmp10 << 6) | (63 & arg0[tmp11]);
              let diff = tmp9 - 1;
              tmp6 = diff;
              tmp7 = tmp13;
              tmp8 = sum3;
              while (diff > 1) {
                tmp9 = diff;
                tmp10 = tmp13;
                tmp11 = sum3;
                tmp6 = diff;
                tmp7 = tmp13;
                tmp8 = sum3;
                if (sum3 >= length) {
                  break;
                }
              }
            }
          }
          if (tmp6 > 1) {
            sum1 = num + 1;
            array[num] = 65533;
            sum2 = tmp8;
          } else if (tmp7 < 65536) {
            sum1 = num + 1;
            array[num] = tmp7;
            sum2 = tmp8;
          } else {
            let diff1 = tmp7 - 65536;
            let sum4 = num + 1;
            array[num] = 55296 | ((diff1 >> 10) & 1023);
            sum1 = sum4 + 1;
            array[sum4] = 56320 | (1023 & diff1);
            sum2 = tmp8;
          }
        }
      }
      num = sum1;
      num2 = sum2;
      num3 = sum1;
    } while (sum2 < length);
  }
  if (num3 < 65534) {
    if (!array.subarray) {
      if (!array.subarray) {
        return str2;
      }
    }
    const _String2 = String;
    str2 = fromCharCode.apply(null, closure_1_0(closure_1_1[0]).shrinkBuf(array, num3));
    const obj = closure_1_0(closure_1_1[0]);
  }
  let str = "";
  str2 = "";
  let num6 = 0;
  if (0 < num3) {
    do {
      let _String = String;
      str = `${String.fromCharCode(tmp[num6])}`;
      num6 = num6 + 1;
      str2 = str;
    } while (num6 < num3);
  }
}

function f63875(arg0, arg1) {
  let length = arg1;
  if (!arg1) {
    length = arg0.length;
  }
  if (length > arg0.length) {
    length = arg0.length;
  }
  const diff = length - 1;
  let tmp2 = diff;
  if (0 <= diff) {
    let tmp3 = diff;
    tmp2 = diff;
    if (128 === (192 & arg0[diff])) {
      const diff1 = tmp3 - 1;
      tmp2 = diff1;
      while (0 <= diff1) {
        tmp3 = diff1;
        tmp2 = diff1;
        if (128 !== (192 & arg0[diff1])) {
          break;
        }
      }
    }
  }
  let tmp5 = length;
  if (tmp2 >= 0) {
    tmp5 = length;
    if (0 !== tmp2) {
      let tmp7 = length;
      if (tmp2 + closure_1_4[arg0[tmp2]] > length) {
        tmp7 = tmp2;
      }
      tmp5 = tmp7;
    }
  }
  return tmp5;
}

function isoStop(blobReachResult, bellReachResult) {
  let num = 1;
  if (bellReachResult > 0) {
    const _Math = Math;
    const _Math2 = Math;
    num = Math.min(1, Math.max(0, blobReachResult / bellReachResult));
  }
  return num;
}

function then() {}

function fetch(arg0, arg1) {
  let DOMException = arg0;
  let _Headers = arg1;
  return new Promise((arg0, fn) => {
    DOMException = arg0;
    _Headers = fn;
    function abortXhr() {
      xMLHttpRequest.abort();
    }
    const request = Object.create(closure_1_12.prototype);
    closure_1_12(DOMException, _Headers);
    if (request.signal) {
      if (request.signal.aborted) {
        let dOMException = new DOMException.DOMException("Aborted", "AbortError");
        return fn(dOMException);
      }
    }
    const xMLHttpRequest = new XMLHttpRequest();
    xMLHttpRequest.onload = () => {
      let obj = { statusText: xMLHttpRequest.statusText, headers: null };
      Object.create(closure_2_9.prototype);
      let obj2;
      obj2.map = {};
      if (undefined instanceof closure_2_9) {
        const item = undefined.forEach(function (item, index) {
          this.append(index, item);
        }, obj2);
      } else {
        const _Array = Array;
        if (Array.isArray(undefined)) {
          const item1 = undefined.forEach(function (item) {
            if (2 != item.length) {
              const _TypeError = TypeError;
              const typeError = new TypeError(
                "Headers constructor: expected name/value pair to be length 2, found" + item.length,
              );
              throw typeError;
            } else {
              const self = this;
              this.append(item[0], item[1]);
            }
          }, obj2);
        }
      }
      let str = xMLHttpRequest.getAllResponseHeaders() || "";
      let parts = xMLHttpRequest.getAllResponseHeaders() || "".replace(/\r?\n[\t ]+/g, " ").split("\r");
      const mapped = parts.map((arr) => {
        let substr = arr;
        if (0 === arr.indexOf("\n")) {
          substr = arr.substr(1, arr.length);
        }
        return substr;
      });
      const item2 = mapped.forEach((item) => {
        const parts = item.split(":");
        const trimmed = parts.shift().trim();
        if (trimmed) {
          const joined = parts.join(":");
          try {
            obj2.append(trimmed, tmp3);
          } catch (tmp6) {
            const _console = console;
            console.warn("Response " + tmp6.message);
          }
        }
        const str = parts.shift();
      });
      obj.headers = obj2;
      url = url.url;
      if (0 !== url.indexOf("file://")) {
        obj.status = xMLHttpRequest.status;
      } else {
        obj.status = 200;
      }
      if ("responseURL" in xMLHttpRequest) {
        let responseURL = xMLHttpRequest.responseURL;
      } else {
        const headers = obj.headers;
        responseURL = headers.get("X-Request-URL");
      }
      obj.url = responseURL;
      url = "response" in xMLHttpRequest ? xMLHttpRequest.response : xMLHttpRequest.responseText;
      const timerId = setTimeout(() => {
        obj = Object.create(closure_4_14.prototype);
        closure_4_14(closure_2, obj);
        obj2(obj);
      }, 0);
      const str2 = xMLHttpRequest.getAllResponseHeaders() || "".replace(/\r?\n[\t ]+/g, " ");
    };
    xMLHttpRequest.onerror = () => {
      const timerId = setTimeout(() => {
        const typeError = new TypeError("Network request failed");
        config(typeError);
      }, 0);
    };
    xMLHttpRequest.ontimeout = () => {
      const timerId = setTimeout(() => {
        const typeError = new TypeError("Network request timed out");
        config(typeError);
      }, 0);
    };
    xMLHttpRequest.onabort = () => {
      const timerId = setTimeout(() => {
        const dOMException = new DOMException.DOMException("Aborted", "AbortError");
        config(dOMException);
      }, 0);
    };
    xMLHttpRequest.open(
      request.method,
      (function fixUrl(url) {
        try {
          let href = url;
          if ("" === url) {
            href = url;
            if (config.location.href) {
              href = config.location.href;
            }
          }
          return href;
        } catch (err) {
          return tmp;
        }
      })(request.url),
      true,
    );
    if ("include" === request.credentials) {
      xMLHttpRequest.withCredentials = true;
    } else if ("omit" === request.credentials) {
      xMLHttpRequest.withCredentials = false;
    }
    if ("responseType" in xMLHttpRequest) {
      if (closure_1_4) {
        xMLHttpRequest.responseType = "blob";
      } else if (closure_1_6) {
        xMLHttpRequest.responseType = "arraybuffer";
      }
    }
    if (_Headers) {
      if (typeof tmp.headers === "object") {
        if (!(tmp.headers instanceof closure_1_9)) {
          closure_5 = [];
          const _Object = Object;
          const ownPropertyNames = Object.getOwnPropertyNames(tmp.headers);
          let item = ownPropertyNames.forEach((item) => {
            let str = item;
            if (typeof item !== "string") {
              const _String = String;
              str = String(item);
            }
            if (!obj.test(str)) {
              if ("" !== str) {
                closure_5.push(str.toLowerCase());
                let StringResult = tmp5;
                if (typeof config.headers[item] !== "string") {
                  const _String2 = String;
                  StringResult = String(tmp5);
                }
                xMLHttpRequest.setRequestHeader(item, StringResult);
              }
            }
            const typeError = new TypeError('Invalid character in header field name: "' + str + '"');
            throw typeError;
          });
          let headers = request.headers;
          let item1 = headers.forEach((item, index) => {
            if (-1 === closure_5.indexOf(index)) {
              xMLHttpRequest.setRequestHeader(index, item);
            }
          });
        }
        if (request.signal) {
          let signal = request.signal;
          const listener = signal.addEventListener("abort", abortXhr);
          xMLHttpRequest.onreadystatechange = () => {
            if (4 === xMLHttpRequest.readyState) {
              const signal = request.signal;
              const removed = signal.removeEventListener("abort", abortXhr);
            }
          };
        }
        let _bodyInit = null;
        if (undefined !== request._bodyInit) {
          _bodyInit = request._bodyInit;
        }
        xMLHttpRequest.send(_bodyInit);
      }
    }
    const headers1 = request.headers;
    let item2 = headers1.forEach((item, index) => {
      xMLHttpRequest.setRequestHeader(index, item);
    });
  });
}

function f77866(message, name) {
  const error = { message, name, stack: Error(message).stack };
}

function get() {
  return closure_1_0.name;
}

function f80220(arg0, arg1, str) {
  if (typeof str !== "string") {
    ({ length: closure_1_0.minimumSignificantDigits, length: closure_1_0.maximumSignificantDigits } = arg1);
  } else if ("+" === str) {
    closure_1_0.minimumSignificantDigits = arg1.length;
  } else if ("#" === arg1[0]) {
    closure_1_0.maximumSignificantDigits = arg1.length;
  } else {
    closure_1_0.minimumSignificantDigits = arg1.length;
    let num = 0;
    if (typeof str === "string") {
      num = str.length;
    }
    closure_1_0.maximumSignificantDigits = arg1.length + num;
  }
  return "";
}

function f80222(arg0, arg1) {
  closure_1_0(closure_1_1[1]);
  closure_1_0(closure_1_1[1]);
  switch (arg1) {
    case "sign-auto":
      let obj = { signDisplay: "auto" };
      if (!obj) {
        obj = {};
      }
      return tmp2(tmp4, obj);
    case "sign-accounting":
      obj = { currencySign: "accounting" };
      break;
    case "()":
      obj = { currencySign: "accounting" };
      break;
    case "sign-always":
      obj = { signDisplay: "always" };
      break;
    case "+!":
      obj = { signDisplay: "always" };
      break;
    case "sign-accounting-always":
      obj = { signDisplay: "always", currencySign: "accounting" };
      break;
    case "()!":
      obj = { signDisplay: "always", currencySign: "accounting" };
      break;
    case "sign-except-zero":
      obj = { signDisplay: "exceptZero" };
      break;
    case "+?":
      obj = { signDisplay: "exceptZero" };
      break;
    case "sign-accounting-except-zero":
      obj = { signDisplay: "exceptZero", currencySign: "accounting" };
      break;
    case "()?":
      obj = { signDisplay: "exceptZero", currencySign: "accounting" };
      break;
    case "sign-never":
      obj = { signDisplay: "never" };
      break;
    case "+_":
      obj = { signDisplay: "never" };
      break;
  }
}

function f80223(arg0, arg1) {
  closure_1_0(closure_1_1[1]);
  closure_1_0(closure_1_1[1]);
  switch (arg1) {
    case "sign-auto":
      let obj = { signDisplay: "auto" };
      if (!obj) {
        obj = {};
      }
      return tmp2(tmp4, obj);
    case "sign-accounting":
      obj = { currencySign: "accounting" };
      break;
    case "()":
      obj = { currencySign: "accounting" };
      break;
    case "sign-always":
      obj = { signDisplay: "always" };
      break;
    case "+!":
      obj = { signDisplay: "always" };
      break;
    case "sign-accounting-always":
      obj = { signDisplay: "always", currencySign: "accounting" };
      break;
    case "()!":
      obj = { signDisplay: "always", currencySign: "accounting" };
      break;
    case "sign-except-zero":
      obj = { signDisplay: "exceptZero" };
      break;
    case "+?":
      obj = { signDisplay: "exceptZero" };
      break;
    case "sign-accounting-except-zero":
      obj = { signDisplay: "exceptZero", currencySign: "accounting" };
      break;
    case "()?":
      obj = { signDisplay: "exceptZero", currencySign: "accounting" };
      break;
    case "sign-never":
      obj = { signDisplay: "never" };
      break;
    case "+_":
      obj = { signDisplay: "never" };
      break;
  }
}

function f80224(arg0, arg1, arg2, arg3, arg4, arg5) {
  if (arg1) {
    closure_1_0.minimumIntegerDigits = arg2.length;
  } else {
    if (arg3) {
      if (arg4) {
        const _Error2 = Error;
        const error = new Error("We currently do not support maximum integer digits");
        throw error;
      }
    }
    if (arg5) {
      const _Error = Error;
      const error1 = new Error("We currently do not support exact integer digits");
      throw error1;
    }
  }
  return "";
}

function f80225(arg0, arg1, arg2, arg3, arg4, arg5) {
  if ("*" === arg2) {
    closure_1_0.minimumFractionDigits = arg1.length;
  } else {
    if (arg3) {
      if ("#" === arg3[0]) {
        closure_1_0.maximumFractionDigits = arg3.length;
      }
    }
    if (arg4) {
      if (arg5) {
        closure_1_0.minimumFractionDigits = arg4.length;
        closure_1_0.maximumFractionDigits = arg4.length + arg5.length;
      }
    }
    ({ length: closure_1_0.minimumFractionDigits, length: closure_1_0.maximumFractionDigits } = arg1);
  }
  return "";
}

function f80308(arg0, arg1, str) {
  if (typeof str !== "string") {
    ({ length: closure_1_0.minimumSignificantDigits, length: closure_1_0.maximumSignificantDigits } = arg1);
  } else if ("+" === str) {
    closure_1_0.minimumSignificantDigits = arg1.length;
  } else if ("#" === arg1[0]) {
    closure_1_0.maximumSignificantDigits = arg1.length;
  } else {
    closure_1_0.minimumSignificantDigits = arg1.length;
    let num = 0;
    if (typeof str === "string") {
      num = str.length;
    }
    closure_1_0.maximumSignificantDigits = arg1.length + num;
  }
  return "";
}

function f80310(arg0, arg1) {
  closure_1_0(closure_1_1[1]);
  closure_1_0(closure_1_1[1]);
  switch (arg1) {
    case "sign-auto":
      let obj = { signDisplay: "auto" };
      if (!obj) {
        obj = {};
      }
      return tmp2(tmp4, obj);
    case "sign-accounting":
      obj = { currencySign: "accounting" };
      break;
    case "()":
      obj = { currencySign: "accounting" };
      break;
    case "sign-always":
      obj = { signDisplay: "always" };
      break;
    case "+!":
      obj = { signDisplay: "always" };
      break;
    case "sign-accounting-always":
      obj = { signDisplay: "always", currencySign: "accounting" };
      break;
    case "()!":
      obj = { signDisplay: "always", currencySign: "accounting" };
      break;
    case "sign-except-zero":
      obj = { signDisplay: "exceptZero" };
      break;
    case "+?":
      obj = { signDisplay: "exceptZero" };
      break;
    case "sign-accounting-except-zero":
      obj = { signDisplay: "exceptZero", currencySign: "accounting" };
      break;
    case "()?":
      obj = { signDisplay: "exceptZero", currencySign: "accounting" };
      break;
    case "sign-never":
      obj = { signDisplay: "never" };
      break;
    case "+_":
      obj = { signDisplay: "never" };
      break;
  }
}

function f80311(arg0, arg1) {
  closure_1_0(closure_1_1[1]);
  closure_1_0(closure_1_1[1]);
  switch (arg1) {
    case "sign-auto":
      let obj = { signDisplay: "auto" };
      if (!obj) {
        obj = {};
      }
      return tmp2(tmp4, obj);
    case "sign-accounting":
      obj = { currencySign: "accounting" };
      break;
    case "()":
      obj = { currencySign: "accounting" };
      break;
    case "sign-always":
      obj = { signDisplay: "always" };
      break;
    case "+!":
      obj = { signDisplay: "always" };
      break;
    case "sign-accounting-always":
      obj = { signDisplay: "always", currencySign: "accounting" };
      break;
    case "()!":
      obj = { signDisplay: "always", currencySign: "accounting" };
      break;
    case "sign-except-zero":
      obj = { signDisplay: "exceptZero" };
      break;
    case "+?":
      obj = { signDisplay: "exceptZero" };
      break;
    case "sign-accounting-except-zero":
      obj = { signDisplay: "exceptZero", currencySign: "accounting" };
      break;
    case "()?":
      obj = { signDisplay: "exceptZero", currencySign: "accounting" };
      break;
    case "sign-never":
      obj = { signDisplay: "never" };
      break;
    case "+_":
      obj = { signDisplay: "never" };
      break;
  }
}

function f80312(arg0, arg1, arg2, arg3, arg4, arg5) {
  if (arg1) {
    closure_1_0.minimumIntegerDigits = arg2.length;
  } else {
    if (arg3) {
      if (arg4) {
        const _Error2 = Error;
        const error = new Error("We currently do not support maximum integer digits");
        throw error;
      }
    }
    if (arg5) {
      const _Error = Error;
      const error1 = new Error("We currently do not support exact integer digits");
      throw error1;
    }
  }
  return "";
}

function f80313(arg0, arg1, arg2, arg3, arg4, arg5) {
  if ("*" === arg2) {
    closure_1_0.minimumFractionDigits = arg1.length;
  } else {
    if (arg3) {
      if ("#" === arg3[0]) {
        closure_1_0.maximumFractionDigits = arg3.length;
      }
    }
    if (arg4) {
      if (arg5) {
        closure_1_0.minimumFractionDigits = arg4.length;
        closure_1_0.maximumFractionDigits = arg4.length + arg5.length;
      }
    }
    ({ length: closure_1_0.minimumFractionDigits, length: closure_1_0.maximumFractionDigits } = arg1);
  }
  return "";
}

function f81299(key10009) {
  const call = hasOwnProperty.call;
  let tmp2 = typeof call === "unknown" ? hasOwnProperty(key10009) : call(closure_1_0, key10009);
  if (tmp2) {
    tmp2 = typeof closure_1_1[key10009] === "string";
  }
  if (tmp2) {
    closure_1_1[key10009] = closure_1_0[key10009](closure_1_1[key10009]);
  }
}

function update(arg0, arg1, arg2, arg3) {
  let tmp = arg0;
  let num = 0;
  let tmp2 = arg0;
  if (0 < arg3) {
    do {
      tmp = closure_1_3.table[255 & (tmp ^ arg1[arg2 + num])] ^ (tmp >>> 8);
      num = num + 1;
      tmp2 = tmp;
    } while (num < arg3);
  }
  return tmp2;
}

function crc(uint8Array, sum48, sum13) {
  return 4294967295 ^ closure_1_3.update(4294967295, uint8Array, sum48, sum13);
}

function multVec(items, items1) {
  items = [, , ,];
  items[0] = items[0] * items1[0] + items[1] * items1[1] + items[2] * items1[2] + items[3] * items1[3];
  items[1] = items[4] * items1[0] + items[5] * items1[1] + items[6] * items1[2] + items[7] * items1[3];
  items[2] = items[8] * items1[0] + items[9] * items1[1] + items[10] * items1[2] + items[11] * items1[3];
  items[3] = items[12] * items1[0] + items[13] * items1[1] + items[14] * items1[2] + items[15] * items1[3];
  return items;
}

function dot(multVecResult, items2) {
  return (
    multVecResult[0] * items2[0] +
    multVecResult[1] * items2[1] +
    multVecResult[2] * items2[2] +
    multVecResult[3] * items2[3]
  );
}

function sml(arg0, items2) {
  const items = [arg0 * items2[0], arg0 * items2[1], arg0 * items2[2], arg0 * items2[3]];
  return items;
}

function encode(arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
  let num = arg3;
  if (null == arg3) {
    num = 0;
  }
  let flag = arg6;
  if (null == arg6) {
    flag = false;
  }
  const items = [false, false, false, 0, flag, false];
  const tmp = closure_1_7(arg0, arg1, arg2, num, items);
  closure_1_6(tmp, -1);
  return closure_1_5(tmp, arg1, arg2, arg4, arg5);
}

function encodeLL(arg0, width, height, arg3, arg4, depth, arg6, arg7) {
  let length;
  let num = 2;
  if (1 == arg3) {
    num = 0;
  }
  let num2 = 4;
  if (0 == arg4) {
    num2 = 0;
  }
  const obj = { ctype: num + num2, depth, frames: [] };
  const timestamp = Date.now();
  const result = (arg3 + arg4) * depth;
  let num3 = 0;
  if (0 < arg0.length) {
    do {
      let frames = obj.frames;
      let obj2 = { rect: null, img: null, blend: 0, dispose: 1, bpp: null, bpl: null };
      let size = { x: 0, y: 0, width, height };
      obj2.rect = size;
      let _Uint8Array = Uint8Array;
      let tmp4 = new.target;
      let tmp5 = new.target;
      let uint8Array = new Uint8Array(arg0[num3]);
      obj2.img = uint8Array;
      let _Math = Math;
      obj2.bpp = Math.ceil(result / 8);
      let _Math2 = Math;
      obj2.bpl = Math.ceil(tmp3 / 8);
      let arr = frames.push(obj2);
      num3 = num3 + 1;
      length = arg0.length;
    } while (num3 < length);
  }
  closure_1_6(obj, 0, true);
  return closure_1_5(obj, width, height, arg6, arg7);
}

function f82944(arg0, arg1) {
  return arg1[1].lastUsedMs - arg0[1].lastUsedMs;
}

function f93178(timestamp_ms, timestamp_ms2) {
  return timestamp_ms.timestamp_ms - timestamp_ms2.timestamp_ms;
}

function createListState(channelId, sortOrder, tagFilter, tagSetting) {
  return {
    loading: false,
    isInitialLoad: true,
    hasMore: false,
    failed: false,
    threads: [],
    nextOffset: 0,
    channelId,
    sortOrder,
    tagFilter,
    tagSetting,
  };
}

function touchList(arg0, arg1) {
  closure_1_8.delete(arg0);
  const result = closure_1_8.set(arg0, arg1);
  if (closure_1_8.size > 50) {
    const obj = closure_1_8[Symbol.iterator]();
    while (obj !== undefined) {
      let tmp11 = closure_1_3(tmp8, 2);
      [tmp12, tmp13] = tmp11;
      if (closure_1_8.size <= 50) {
        obj.return();
        break;
      } else {
        if (!tmp13.loading) {
          let deleteResult1 = closure_1_8.delete(tmp12);
        }
        continue;
      }
      break;
    }
  }
}

function f97156() {
  closure_1_0._zod.run = closure_1_0._zod.parse;
}

function runChecks(memo, arg1, arg2) {
  closure_0 = memo;
  let async = arg2;
  if (memo.memo) {
    return memo;
  } else {
    closure_3 = closure_1_10.aborted(memo);
    function _loop() {
      if (_zod._zod.def.when) {
        if (closure_2_10.explicitlyAborted(length)) {
          return 0;
        } else {
          const def = tmp._zod.def;
          if (!def.when(tmp4)) {
            return 0;
          }
        }
        tmp4 = length;
      } else if (closure_3) {
        return 0;
      }
      _zod = tmp._zod;
      const checkResult = _zod.check(length.issues.length);
      async = checkResult;
      if (checkResult instanceof Promise) {
        async = undefined;
        if (async != null) {
          async = async.async;
        }
        if (false === async) {
          const ZodAsyncError = new closure_2_8.$ZodAsyncError();
          throw ZodAsyncError;
        }
      }
      if (!closure_2) {
        if (!(checkResult instanceof Promise)) {
          if (tmp5.issues.length === length) {
            return 0;
          } else {
            closure_2_10.attachSchema(tmp5.issues, length, closure_0);
            if (!closure_3) {
              closure_3 = closure_2_10.aborted(tmp5, length);
            }
          }
        }
      }
      let resolved = closure_2;
      if (closure_2 == null) {
        resolved = Promise.resolve();
      }
      closure_2 = resolved.then(
        closure_2_4(function* () {
          if (c2 === 2) {
            c2 = 3;
            throw new TypeError("Generator functions may not be called on executing generators");
          } else if (tmp3 === 3) {
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
              c2 = 2;
              if (0 === c1) {
                if (arg0 === 1) {
                  c2 = 3;
                  throw value;
                } else if (arg0 === 2) {
                  c2 = 3;
                  const obj3 = { value, done: true };
                  return obj3;
                } else {
                  c1 = 1;
                  c2 = 1;
                  const obj4 = { value: checkResult, done: false };
                  return obj4;
                }
              } else if (arg0 === 1) {
                c2 = 3;
                throw value;
              } else if (arg0 === 2) {
                c2 = 3;
                const obj = { value, done: true };
                return obj;
              } else {
                if (tmp4.issues.length !== closure_128_0) {
                  closure_3_10.attachSchema(tmp4.issues, closure_128_0, length);
                  if (!closure_3) {
                    closure_3 = closure_3_10.aborted(tmp4, closure_128_0);
                  }
                }
                c2 = 3;
                return { value: "HermesInternal", done: null };
              }
            } catch (tmp16) {
              c2 = tmp;
              throw tmp16;
            }
          }
        }),
      );
    }
    const iter = arg1[Symbol.iterator]();
    while (iter !== undefined) {
      let _zod = iter.next();
      let _loopResult = _loop();
      continue;
    }
    let nextPromise = memo;
    if (closure_2) {
      nextPromise = closure_2.then(() => closure_0);
    }
    return nextPromise;
  }
}

function handleCanaryResult(arg0, arg1, arg2) {}

function f97159(value, skipChecks) {
  closure_0 = value;
  if (skipChecks.skipChecks) {
    const _zod4 = closure_0._zod;
    return _zod4.parse(value, skipChecks);
  } else if ("backward" === skipChecks.direction) {
    const _zod2 = closure_0._zod;
    const obj = { value: value.value, issues: [] };
    const obj2 = {};
    const merged = Object.assign(skipChecks);
    obj2.skipChecks = true;
    const parsed = _zod2.parse(obj, obj2);
    if (parsed instanceof Promise) {
      let nextPromise = parsed.then((result) => {
        if (typeof closure_2_3 === "function") {
          let _zod = skipChecks;
          if (closure_1_10.aborted(result)) {
            result.aborted = true;
            let nextPromise = result;
          } else {
            const promise = closure_2_2(tmp, closure_2_1, skipChecks);
            if (promise instanceof Promise) {
              if (false === skipChecks.async) {
                ZodAsyncError = new ZodAsyncError.$ZodAsyncError();
                throw ZodAsyncError;
              } else {
                nextPromise = promise.then((result) => {
                  _zod = _zod._zod;
                  return _zod.parse(result, _zod);
                });
              }
            } else {
              _zod = closure_2_0._zod;
              nextPromise = _zod.parse(promise, skipChecks);
            }
          }
          return nextPromise;
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      });
    } else if (typeof closure_3 === "function") {
      closure_129_0 = skipChecks;
      if (closure_1_10.aborted(parsed)) {
        parsed.aborted = true;
        nextPromise = parsed;
      } else {
        const promise3 = closure_2(value, skipChecks, skipChecks);
        if (promise3 instanceof Promise) {
          if (false === skipChecks.async) {
            let ZodAsyncError = new closure_1_8.$ZodAsyncError();
            throw ZodAsyncError;
          } else {
            nextPromise = promise3.then((result) => {
              _zod = _zod._zod;
              return _zod.parse(result, _zod);
            });
          }
        } else {
          const _zod3 = closure_0._zod;
          nextPromise = _zod3.parse(promise3, skipChecks);
        }
      }
    } else {
      throw new TypeError("Trying to call a non-function");
    }
    return nextPromise;
  } else {
    let _zod = closure_0._zod;
    const parsed1 = _zod.parse(value, skipChecks);
    if (parsed1 instanceof Promise) {
      if (false === skipChecks.async) {
        const ZodAsyncError1 = new closure_1_8.$ZodAsyncError();
        throw ZodAsyncError1;
      } else {
        return parsed1.then((result) => closure_2_2(result, closure_2_1, closure_1));
      }
    } else {
      return closure_2(parsed1, skipChecks, skipChecks);
    }
  }
}

function f101552() {
  let hasFlagResult = "Preview" === closure_1_7;
  if (!hasFlagResult) {
    hasFlagResult = closure_2_0(closure_2_2[33]).hasFlag(closure_1_3.flags, closure_2_20.EPHEMERAL);
    const obj = closure_2_0(closure_2_2[33]);
  }
  const items = [];
  if (hasFlagResult) {
    items.push(
      closure_1_18,
      closure_1_19,
      closure_1_25,
      closure_1_26,
      closure_1_27,
      closure_1_28,
      closure_1_33,
      closure_1_39,
      closure_1_14,
      closure_1_15,
      closure_1_31,
      closure_1_30,
      closure_1_29,
      closure_1_34,
    );
  }
  let tmp22 = closure_1_13;
  if (closure_1_13) {
    tmp22 = !closure_2_0(closure_2_2[33]).hasFlag(closure_1_3.flags, closure_2_20.EPHEMERAL);
    const obj2 = closure_2_0(closure_2_2[33]);
  }
  if (!tmp22) {
    items.push(
      closure_1_14,
      closure_1_15,
      closure_1_16,
      closure_1_33,
      closure_1_35,
      closure_1_39,
      closure_1_20,
      closure_1_21,
      closure_1_22,
      closure_1_23,
      closure_1_24,
      closure_1_19,
      closure_1_31,
      closure_1_29,
      closure_1_34,
    );
  }
  if (obj3.hasFlag(closure_1_3.flags, closure_2_20.EPHEMERAL)) {
    items.push(closure_1_17, closure_1_16, closure_1_32, closure_1_36, closure_1_37, closure_1_38);
  }
  obj3 = closure_2_0(closure_2_2[33]);
  return new Set(items);
}

function f101553(id) {
  return id.id === closure_1_6.source.id;
}

function f101554(arg0) {
  return !closure_1_40.has(arg0);
}

function f101555(arr) {
  return arr.filter((item) => set.has(item));
}

function f101556(arg0) {
  return arg0.length > 0;
}

function _loop4() {
  if (" " === closure_1[1][0]) {
    return 1;
  } else {
    let str = "";
    const parts = obj.trimEnd().split(/\s/);
    const mapped = parts.map((item) => {
      str = `${str}${item} `;
      return str.trim();
    });
    const reversed = mapped.reverse();
    str = reversed.find((item) => null != closure_1_2[item]);
    if (str == null) {
      str = "";
    }
    if (null != closure_2[str]) {
      const obj3 = { location: closure_1.index, length: str.length + 1, data: null };
      const obj4 = { type: closure_1_0(closure_1_2[9]).ChatInputParseResultDataType.ROLE_HIGHLIGHT, color: null };
      let colorString = null;
      if ("username" === closure_1_5.roleStyle) {
        colorString = tmp3.colorString;
      }
      obj4.color = colorString;
      obj3.data = obj4;
      str.push(obj3);
    } else if (obj2.startsWith(closure_1_22)) {
      const obj5 = { location: closure_1.index, length: 9, data: null };
      const obj6 = {
        type: closure_1_0(closure_1_2[9]).ChatInputParseResultDataType.ROLE_HIGHLIGHT,
        color: closure_1_1(closure_1_2[18]).unsafe_rawColors.BRAND_500,
      };
      obj5.data = obj6;
      str.push(obj5);
    } else if (obj2.startsWith(closure_1_23)) {
      const obj7 = { location: closure_1.index, length: 5, data: null };
      const obj8 = {
        type: closure_1_0(closure_1_2[9]).ChatInputParseResultDataType.ROLE_HIGHLIGHT,
        color: closure_1_1(closure_1_2[18]).unsafe_rawColors.BRAND_500,
      };
      obj7.data = obj8;
      str.push(obj7);
    }
    const str3 = obj.trimEnd();
  }
}

function InsertUnicodeExtensionAndCanonicalize(arr, arg1, arg2) {
  closure_1_4(-1 === arr.indexOf("-u-"), "Expected locale to not have a Unicode locale extension");
  let str = "-u";
  while (tmp2 !== undefined) {
    let _HermesInternal = HermesInternal;
    str = `-u${"-" + tmp3}`;
    continue;
  }
  const iter = arg2[Symbol.iterator]();
  const iter2 = iter.next();
  while (iter !== undefined) {
    let value = iter2.value;
    let _HermesInternal2 = HermesInternal;
    let tmp5 = value;
    str = str + "-" + iter2.key;
    if ("" !== value) {
      let _HermesInternal3 = HermesInternal;
      str = str + "-" + tmp5;
    }
    continue;
  }
  if ("-u" === str) {
    return closure_1_13(arr);
  } else {
    const index = arr.indexOf("-x-");
    if (-1 === index) {
      let sum = arr + str;
    } else {
      const sum1 = arr.slice(0, index) + str;
      sum = sum1 + arr.slice(index);
    }
    return closure_1_13(sum);
  }
  tmp2 = arg1[Symbol.iterator]();
}

function bellReach(bound) {
  let num = 0;
  if (bound < closure_1_0) {
    const _Math = Math;
    num = 1 - Math.sqrt(bound / tmp);
  }
  return num;
}

function blobReach(peak, radius, bound) {
  let num = 0;
  if (bound < peak) {
    const _Math = Math;
    const _Math2 = Math;
    num = radius * Math.sqrt(Math.log(peak / bound));
  }
  return num;
}

function f121465() {
  return {
    navigate(arg0, arg1) {
      const navigation = closure_1_0.navigation;
      const obj = {};
      const CommonActions = closure_2_0(closure_2_1[3]).CommonActions;
      const merged = Object.assign(CommonActions.navigate(arg0, arg1));
      obj.target = closure_1_0.state.key;
      navigation.dispatch(obj);
    },
    back() {
      const navigation = closure_1_0.navigation;
      navigation.goBack();
    },
  };
}

function f121466() {
  return { emit: closure_1_0.navigation.emit };
}

function f121577(offset) {
  let tmp2 = closure_1_0;
  if (closure_1_0) {
    const obj2 = { timingFunction: tmp };
    tmp2 = obj2;
  }
  const merged = Object.assign(tmp2);
  return { offset, props: closure_1_1 };
}

function _loop() {
  let attrs = closure_1_8.attrs;
  let str = "";
  if (null != attrs) {
    let _Object = Object;
    let entries = Object.entries(attrs);
    let mapped = entries.map((item) => {
      [tmp, tmp2] = item;
      combined = tmp;
      if (true !== tmp2) {
        const _String = String;
        const _HermesInternal = HermesInternal;
        combined = "" + tmp + "=" + String(tmp2);
      }
      return combined;
    });
    str = mapped.join(" ");
  }
  let combined = "" + closure_1_8.name + " " + str;
  let hasItem = closure_2_1.has(closure_1_8.id);
  let tmp5 = !hasItem;
  if (!hasItem) {
    tmp5 = null != closure_1_8.end;
  }
  if (tmp5) {
    tmp5 = null == closure_1_8.error;
  }
  if (tmp5) {
    let num = closure_1_6.get(combined);
    if (num == null) {
      num = 0;
    }
    if (num >= 4) {
      if (closure_1_7.has(combined)) {
        return 1;
      } else {
        closure_1_7.add(combined);
        const children = closure_1_5.children;
        children.push(
          closure_2_4(
            closure_1_4.filter((id) => {
              const hasItem = closure_2_1.has(id.id);
              let tmp2 = !hasItem;
              if (!hasItem) {
                tmp2 = null != id.end;
              }
              if (tmp2) {
                tmp2 = null == id.error;
              }
              if (tmp2) {
                const attrs = id.attrs;
                let str2 = "";
                if (null != attrs) {
                  const _Object = Object;
                  const entries = Object.entries(attrs);
                  const mapped = entries.map((item) => {
                    [tmp, tmp2] = item;
                    combined = tmp;
                    if (true !== tmp2) {
                      const _String = String;
                      const _HermesInternal = HermesInternal;
                      combined = "" + tmp + "=" + String(tmp2);
                    }
                    return combined;
                  });
                  str2 = mapped.join(" ");
                }
                let _HermesInternal = HermesInternal;
                tmp2 = "" + id.name + " " + str2 === combined;
              }
              return tmp2;
            }),
            closure_1_1 + 1,
            closure_1_4,
            closure_1_3,
          ),
        );
      }
    }
  }
  const children1 = closure_1_5.children;
  const items = [closure_1_8];
  children1.push(closure_2_4(items, closure_1_1 + 1, closure_1_4, closure_1_3));
}

function f123921(arg0, count) {
  return arg0 + count.count + count.descendants;
}

function ot(str) {
  const parts = str.split("\n");
  const mapped = parts.map(closure_1_53);
  const found = mapped.filter((item) => null !== item);
  return closure_1_38(closure_1_38({}, found[0]), found[1]);
}

function f126519() {
  return closure_1_0;
}

function f131025(soft_deleted) {
  return !soft_deleted.soft_deleted;
}

function f131026(soft_deleted) {
  return soft_deleted.soft_deleted;
}

function f135465(arg0) {
  closure_1_8.setRequestHeader(arg0, closure_1_6[arg0]);
}

function f136359(arr) {
  return arr.every((item) => {
    const ipV6 = regex.regex.ipV6;
    return ipV6.test(item);
  });
}

function f136360(arg0, tldBlacklist) {
  const _Array = Array;
  if (tldBlacklist.tldBlacklist) {
    if (!isArray(tldBlacklist.tldBlacklist)) {
      const hasOwn2 = closure_1_2.hasOwn;
      const call3 = hasOwn2.call;
      tldBlacklist = tldBlacklist.tldBlacklist;
      !(typeof call3 === "unknown" ? hasOwn2(arg0) : call3(tldBlacklist, arg0));
    }
    const indexOf2 = closure_1_2.indexOf;
    const call4 = indexOf2.call;
    const tldBlacklist2 = tldBlacklist.tldBlacklist;
    -1 === (typeof call4 === "unknown" ? indexOf2(arg0) : call4(tldBlacklist2, arg0));
  } else if (isArray(tldBlacklist.tldWhitelist)) {
    const indexOf = closure_1_2.indexOf;
    const call2 = indexOf.call;
    const tldWhitelist2 = tldBlacklist.tldWhitelist;
    -1 !== (typeof call2 === "unknown" ? indexOf(arg0) : call2(tldWhitelist2, arg0));
  } else {
    const hasOwn = closure_1_2.hasOwn;
    const call = hasOwn.call;
    const tldWhitelist = tldBlacklist.tldWhitelist;
    return typeof call === "unknown" ? hasOwn(arg0) : call(tldWhitelist, arg0);
  }
}

function f136361(arg0, arg1, arg2) {
  let tmp14;
  let obj = arg1;
  let tmp = arg2;
  let domain = arg2;
  if (!arg1) {
    obj = {};
  }
  let obj2 = obj;
  if (typeof obj === "function") {
    domain = obj;
    obj2 = {};
    tmp = obj;
  }
  if (typeof tmp !== "function") {
    if (obj2.checkDNS) {
      const _TypeError4 = TypeError;
      const typeError = new TypeError("expected callback function for checkDNS option");
      throw typeError;
    } else {
      domain = null;
    }
  }
  let errorLevel;
  let valid;
  if (typeof obj2.errorLevel === "number") {
    errorLevel = true;
    valid = obj2.errorLevel;
  } else {
    errorLevel = obj2.errorLevel;
    valid = valid.diagnoses.valid;
  }
  if (obj2.tldWhitelist) {
    if (typeof obj2.tldWhitelist === "string") {
      const items = [obj2.tldWhitelist];
      obj2.tldWhitelist = items;
    } else if ("object" !== domain(obj2.tldWhitelist)) {
      const _TypeError = TypeError;
      const typeError1 = new TypeError("expected array or object tldWhitelist");
      throw typeError1;
    }
  }
  if (obj2.tldBlacklist) {
    if (typeof obj2.tldBlacklist === "string") {
      const items1 = [obj2.tldBlacklist];
      obj2.tldBlacklist = items1;
    } else if ("object" !== domain(obj2.tldBlacklist)) {
      const _TypeError2 = TypeError;
      const typeError2 = new TypeError("expected array or object tldBlacklist");
      throw typeError2;
    }
  }
  if (obj2.minDomainAtoms) {
    const _TypeError3 = TypeError;
    const typeError3 = new TypeError("expected positive integer minDomainAtoms");
    throw typeError3;
  }
  let obj3 = valid;
  let dnsWarnNoRecord = valid.diagnoses.valid;
  function updateResult(arg0) {}
  let localpart = valid.components.localpart;
  let localpart2 = valid.components.localpart;
  const items2 = [valid.components.localpart];
  const obj5 = { local: "", domain: "" };
  const obj10 = { locals: [""], domains: [""] };
  let sum2 = 0;
  let num = 0;
  let num2 = 0;
  let flag2 = false;
  let flag3 = false;
  let num3 = 0;
  let num4 = 0;
  let num5 = 0;
  let str3 = "";
  let tmp13 = localpart;
  let num6 = 0;
  let flag4 = false;
  let num7 = 0;
  let num8 = 0;
  if (0 < arg0.length) {
    while (true) {
      let obj6 = arg0[num2];
      let obj7 = valid;
      if (valid.components.localpart === localpart) {
        if ("(" === obj6) {
          if (0 === num4) {
            if (0 === num5) {
              let deprecatedComment2 = obj7.diagnoses.cfwsComment;
            } else {
              deprecatedComment2 = obj7.diagnoses.deprecatedComment;
            }
            let flag10 = flag2;
            if (deprecatedComment2 > dnsWarnNoRecord) {
              dnsWarnNoRecord = deprecatedComment2;
              flag10 = flag2;
            }
          } else {
            let cfwsComment2 = obj7.diagnoses.cfwsComment;
            flag10 = true;
            if (cfwsComment2 > dnsWarnNoRecord) {
              dnsWarnNoRecord = cfwsComment2;
              flag10 = true;
            }
          }
          let arr = items2.push(localpart);
          let contextComment = obj7.components.contextComment;
        } else if ("." === obj6) {
          if (0 === num4) {
            if (0 === num5) {
              let errConsecutiveDots2 = obj7.diagnoses.errDotStart;
            } else {
              errConsecutiveDots2 = obj7.diagnoses.errConsecutiveDots;
            }
            let tmp32 = localpart2;
            let contextFWS = localpart;
            let num11 = num;
            let diff = num2;
            let tmp38 = obj6;
            let flag5 = flag2;
            let tmp39 = flag3;
            let num10 = num3;
            let num12 = num4;
            let num13 = num5;
            let tmp40 = str3;
            if (errConsecutiveDots2 > dnsWarnNoRecord) {
              dnsWarnNoRecord = errConsecutiveDots2;
              tmp32 = localpart2;
              contextFWS = localpart;
              num11 = num;
              diff = num2;
              tmp38 = obj6;
              flag5 = flag2;
              tmp39 = flag3;
              num10 = num3;
              num12 = num4;
              num13 = num5;
              tmp40 = str3;
            }
          } else {
            if (flag2) {
              let deprecatedLocalPart2 = obj7.diagnoses.deprecatedLocalPart;
              if (deprecatedLocalPart2 > dnsWarnNoRecord) {
                dnsWarnNoRecord = deprecatedLocalPart2;
              }
            }
            let sum = num5 + 1;
            sum2 = sum;
            obj5.local = obj5.local + obj6;
            obj10.locals[sum] = "";
            tmp32 = localpart2;
            contextFWS = localpart;
            num11 = sum;
            diff = num2;
            tmp38 = obj6;
            flag5 = false;
            tmp39 = flag3;
            num10 = num3;
            num12 = 0;
            num13 = sum;
            tmp40 = str3;
          }
        } else if ('"' === obj6) {
          if (0 === num4) {
            if (0 === num5) {
              let deprecatedLocalPart = obj7.diagnoses.rfc5321QuotedString;
            } else {
              deprecatedLocalPart = obj7.diagnoses.deprecatedLocalPart;
            }
            if (deprecatedLocalPart > dnsWarnNoRecord) {
              dnsWarnNoRecord = deprecatedLocalPart;
            }
            obj5.local = obj5.local + obj6;
            let locals6 = obj10.locals;
            locals6[num5] = locals6[num5] + obj6;
            num12 = num4 + 1;
            let arr2 = items2.push(localpart);
            contextFWS = obj7.components.contextQuotedString;
            tmp32 = localpart2;
            num11 = num;
            diff = num2;
            tmp38 = obj6;
            flag5 = true;
            tmp39 = flag3;
            num10 = num3;
            num13 = num5;
            tmp40 = str3;
          } else {
            let errExpectingATEXT4 = obj7.diagnoses.errExpectingATEXT;
            tmp32 = localpart2;
            contextFWS = localpart;
            num11 = num;
            diff = num2;
            tmp38 = obj6;
            flag5 = flag2;
            tmp39 = flag3;
            num10 = num3;
            num12 = num4;
            num13 = num5;
            tmp40 = str3;
            if (errExpectingATEXT4 > dnsWarnNoRecord) {
              dnsWarnNoRecord = errExpectingATEXT4;
              tmp32 = localpart2;
              contextFWS = localpart;
              num11 = num;
              diff = num2;
              tmp38 = obj6;
              flag5 = flag2;
              tmp39 = flag3;
              num10 = num3;
              num12 = num4;
              num13 = num5;
              tmp40 = str3;
            }
          }
        } else {
          if ("\r" === obj6) {
            let sum1 = num2 + 1;
            if (length !== sum1) {
              let tmp205 = sum1;
            }
            let errCRNoLF6 = obj7.diagnoses.errCRNoLF;
            tmp32 = localpart2;
            contextFWS = localpart;
            num11 = num;
            diff = sum1;
            tmp38 = obj6;
            flag5 = flag2;
            tmp39 = flag3;
            num10 = num3;
            num12 = num4;
            num13 = num5;
            tmp40 = str3;
            if (errCRNoLF6 > dnsWarnNoRecord) {
              dnsWarnNoRecord = errCRNoLF6;
              tmp32 = localpart2;
              contextFWS = localpart;
              num11 = num;
              diff = sum1;
              tmp38 = obj6;
              flag5 = flag2;
              tmp39 = flag3;
              num10 = num3;
              num12 = num4;
              num13 = num5;
              tmp40 = str3;
            }
          } else {
            tmp205 = num2;
            if (" " !== obj6) {
              tmp205 = num2;
              if ("\t" !== obj6) {
                if ("@" === obj6) {
                  if (1 !== items2.length) {
                    let tmp139 = globalThis;
                    let _Error5 = Error;
                    let tmp140 = new.target;
                    let str8 = "unexpected item on context stack";
                    let tmp141 = new.target;
                    let error = new Error("unexpected item on context stack");
                    throw error;
                  } else {
                    if (0 === obj5.local.length) {
                      let errNoLocalPart = obj7.diagnoses.errNoLocalPart;
                      if (errNoLocalPart > dnsWarnNoRecord) {
                        dnsWarnNoRecord = errNoLocalPart;
                      }
                    } else if (0 === num4) {
                      let errDotEnd = obj7.diagnoses.errDotEnd;
                      if (errDotEnd > dnsWarnNoRecord) {
                        dnsWarnNoRecord = errDotEnd;
                      }
                    } else if (obj5.local.length > 64) {
                      let rfc5322LocalTooLong = obj7.diagnoses.rfc5322LocalTooLong;
                      if (rfc5322LocalTooLong > dnsWarnNoRecord) {
                        dnsWarnNoRecord = rfc5322LocalTooLong;
                      }
                    } else {
                      let tmp134 =
                        localpart2 !== obj7.components.contextComment && localpart2 !== obj7.components.contextFWS;
                      if (!tmp134) {
                        let deprecatedCFWSNearAt = obj7.diagnoses.deprecatedCFWSNearAt;
                        if (deprecatedCFWSNearAt > dnsWarnNoRecord) {
                          dnsWarnNoRecord = deprecatedCFWSNearAt;
                        }
                      }
                    }
                    contextFWS = obj7.components.domain;
                    items2[0] = obj7.components.domain;
                    sum2 = 0;
                    tmp32 = localpart2;
                    num11 = 0;
                    diff = num2;
                    tmp38 = obj6;
                    flag5 = false;
                    tmp39 = flag3;
                    num10 = num3;
                    num12 = 0;
                    num13 = 0;
                    tmp40 = str3;
                  }
                } else if (flag2) {
                  if (obj7.components.contextComment !== localpart2) {
                    if (obj7.components.contextFWS !== localpart2) {
                      if (obj7.components.contextQuotedString === localpart2) {
                        let errATEXTAfterQS = obj7.diagnoses.errATEXTAfterQS;
                        tmp32 = localpart2;
                        contextFWS = localpart;
                        num11 = num;
                        diff = num2;
                        tmp38 = obj6;
                        flag5 = flag2;
                        tmp39 = flag3;
                        num10 = num3;
                        num12 = num4;
                        num13 = num5;
                        tmp40 = str3;
                        if (errATEXTAfterQS > dnsWarnNoRecord) {
                          dnsWarnNoRecord = errATEXTAfterQS;
                          tmp32 = localpart2;
                          contextFWS = localpart;
                          num11 = num;
                          diff = num2;
                          tmp38 = obj6;
                          flag5 = flag2;
                          tmp39 = flag3;
                          num10 = num3;
                          num12 = num4;
                          num13 = num5;
                          tmp40 = str3;
                        }
                      } else {
                        let tmp127 = globalThis;
                        let _Error4 = Error;
                        let str7 = "more atext found where none is allowed, but unrecognized prev context: ";
                        let tmp128 = new.target;
                        let tmp129 = new.target;
                        let error1 = new Error(
                          "more atext found where none is allowed, but unrecognized prev context: " + localpart2,
                        );
                        throw error1;
                      }
                    }
                  }
                  let errATEXTAfterCFWS2 = obj7.diagnoses.errATEXTAfterCFWS;
                  tmp32 = localpart2;
                  contextFWS = localpart;
                  num11 = num;
                  diff = num2;
                  tmp38 = obj6;
                  flag5 = flag2;
                  tmp39 = flag3;
                  num10 = num3;
                  num12 = num4;
                  num13 = num5;
                  tmp40 = str3;
                  if (errATEXTAfterCFWS2 > dnsWarnNoRecord) {
                    dnsWarnNoRecord = errATEXTAfterCFWS2;
                    tmp32 = localpart2;
                    contextFWS = localpart;
                    num11 = num;
                    diff = num2;
                    tmp38 = obj6;
                    flag5 = flag2;
                    tmp39 = flag3;
                    num10 = num3;
                    num12 = num4;
                    num13 = num5;
                    tmp40 = str3;
                  }
                } else {
                  let charCodeAtResult = obj6.charCodeAt(0);
                  let tmp125 = charCodeAtResult < 33 || charCodeAtResult > 126 || obj7.specials(charCodeAtResult);
                  if (tmp125) {
                    let errExpectingATEXT3 = obj7.diagnoses.errExpectingATEXT;
                    if (errExpectingATEXT3 > dnsWarnNoRecord) {
                      dnsWarnNoRecord = errExpectingATEXT3;
                    }
                  }
                  obj5.local = obj5.local + obj6;
                  let locals5 = obj10.locals;
                  locals5[num5] = locals5[num5] + obj6;
                  num12 = num4 + 1;
                  tmp32 = localpart;
                  contextFWS = localpart;
                  num11 = num;
                  diff = num2;
                  tmp38 = obj6;
                  flag5 = flag2;
                  tmp39 = flag3;
                  num10 = num3;
                  num13 = num5;
                  tmp40 = str3;
                }
              }
            }
          }
          let flag9 = true;
          if (0 === num4) {
            if (0 === num5) {
              let deprecatedFWS3 = obj7.diagnoses.cfwsFWS;
            } else {
              deprecatedFWS3 = obj7.diagnoses.deprecatedFWS;
            }
            flag9 = flag2;
            if (deprecatedFWS3 > dnsWarnNoRecord) {
              dnsWarnNoRecord = deprecatedFWS3;
              flag9 = flag2;
            }
          }
          let arr3 = items2.push(localpart);
          contextFWS = obj7.components.contextFWS;
          flag5 = flag9;
          tmp32 = localpart2;
          num11 = num;
          diff = tmp205;
          tmp38 = obj6;
          tmp39 = flag3;
          num10 = num3;
          num12 = num4;
          num13 = num5;
          tmp40 = obj6;
        }
      } else if (obj7.components.domain === localpart) {
        if ("(" === obj6) {
          if (0 === num4) {
            if (0 === num5) {
              let deprecatedComment = obj7.diagnoses.deprecatedCFWSNearAt;
            } else {
              deprecatedComment = obj7.diagnoses.deprecatedComment;
            }
            let flag8 = flag2;
            if (deprecatedComment > dnsWarnNoRecord) {
              dnsWarnNoRecord = deprecatedComment;
              flag8 = flag2;
            }
          } else {
            let cfwsComment = obj7.diagnoses.cfwsComment;
            flag8 = true;
            if (cfwsComment > dnsWarnNoRecord) {
              dnsWarnNoRecord = cfwsComment;
              flag8 = true;
            }
          }
          let arr5 = items2.push(localpart);
          contextFWS = obj7.components.contextComment;
          flag5 = flag8;
          tmp32 = localpart2;
          num11 = num;
          diff = num2;
          tmp38 = obj6;
          tmp39 = flag3;
          num10 = num3;
          num12 = num4;
          num13 = num5;
          tmp40 = str3;
        } else if ("." === obj6) {
          if (0 === num4) {
            if (0 === num5) {
              let errConsecutiveDots = obj7.diagnoses.errDotStart;
            } else {
              errConsecutiveDots = obj7.diagnoses.errConsecutiveDots;
            }
            if (errConsecutiveDots > dnsWarnNoRecord) {
              dnsWarnNoRecord = errConsecutiveDots;
            }
          } else if (flag3) {
            let errDomainHyphenEnd = obj7.diagnoses.errDomainHyphenEnd;
            if (errDomainHyphenEnd > dnsWarnNoRecord) {
              dnsWarnNoRecord = errDomainHyphenEnd;
            }
          } else if (63 < num4) {
            let rfc5322LabelTooLong = obj7.diagnoses.rfc5322LabelTooLong;
            if (rfc5322LabelTooLong > dnsWarnNoRecord) {
              dnsWarnNoRecord = rfc5322LabelTooLong;
            }
          }
          sum2 = num5 + 1;
          obj10.domains[sum2] = "";
          obj5.domain = obj5.domain + obj6;
          tmp32 = localpart2;
          contextFWS = localpart;
          num11 = sum2;
          diff = num2;
          tmp38 = obj6;
          flag5 = false;
          tmp39 = flag3;
          num10 = num3;
          num12 = 0;
          num13 = sum2;
          tmp40 = str3;
        } else if ("[" === obj6) {
          if (0 === obj5.domain.length) {
            num12 = num4 + 1;
            let arr6 = items2.push(localpart);
            contextFWS = obj7.components.literal;
            obj5.domain = obj5.domain + obj6;
            let domains5 = obj10.domains;
            domains5[num5] = domains5[num5] + obj6;
            obj5.literal = "";
            tmp32 = localpart2;
            num11 = num;
            diff = num2;
            tmp38 = obj6;
            flag5 = true;
            tmp39 = flag3;
            num10 = num3;
            num13 = num5;
            tmp40 = str3;
          } else {
            let errExpectingATEXT2 = obj7.diagnoses.errExpectingATEXT;
            tmp32 = localpart2;
            contextFWS = localpart;
            num11 = num;
            diff = num2;
            tmp38 = obj6;
            flag5 = flag2;
            tmp39 = flag3;
            num10 = num3;
            num12 = num4;
            num13 = num5;
            tmp40 = str3;
            if (errExpectingATEXT2 > dnsWarnNoRecord) {
              dnsWarnNoRecord = errExpectingATEXT2;
              tmp32 = localpart2;
              contextFWS = localpart;
              num11 = num;
              diff = num2;
              tmp38 = obj6;
              flag5 = flag2;
              tmp39 = flag3;
              num10 = num3;
              num12 = num4;
              num13 = num5;
              tmp40 = str3;
            }
          }
        } else {
          if ("\r" === obj6) {
            let sum3 = num2 + 1;
            if (length !== sum3) {
              let tmp204 = sum3;
            }
            let errCRNoLF5 = obj7.diagnoses.errCRNoLF;
            tmp32 = localpart2;
            contextFWS = localpart;
            num11 = num;
            diff = sum3;
            tmp38 = obj6;
            flag5 = flag2;
            tmp39 = flag3;
            num10 = num3;
            num12 = num4;
            num13 = num5;
            tmp40 = str3;
            if (errCRNoLF5 > dnsWarnNoRecord) {
              dnsWarnNoRecord = errCRNoLF5;
              tmp32 = localpart2;
              contextFWS = localpart;
              num11 = num;
              diff = sum3;
              tmp38 = obj6;
              flag5 = flag2;
              tmp39 = flag3;
              num10 = num3;
              num12 = num4;
              num13 = num5;
              tmp40 = str3;
            }
          } else {
            tmp204 = num2;
            if (" " !== obj6) {
              tmp204 = num2;
              if ("\t" !== obj6) {
                if (flag2) {
                  if (obj7.components.contextComment !== localpart2) {
                    if (obj7.components.contextFWS !== localpart2) {
                      if (obj7.components.literal !== localpart2) {
                        break;
                      } else {
                        let errATEXTAfterDomainLiteral = obj7.diagnoses.errATEXTAfterDomainLiteral;
                        if (errATEXTAfterDomainLiteral > dnsWarnNoRecord) {
                          dnsWarnNoRecord = errATEXTAfterDomainLiteral;
                        }
                      }
                    }
                  }
                  let errATEXTAfterCFWS = obj7.diagnoses.errATEXTAfterCFWS;
                  if (errATEXTAfterCFWS > dnsWarnNoRecord) {
                    dnsWarnNoRecord = errATEXTAfterCFWS;
                  }
                }
                let charCodeAtResult1 = obj6.charCodeAt(0);
                if (charCodeAtResult1 >= 33) {
                  if (charCodeAtResult1 <= 126) {
                    if (!obj7.specials(charCodeAtResult1)) {
                      if ("-" === obj6) {
                        let flag6 = true;
                        if (0 === num4) {
                          let errDomainHyphenStart = obj7.diagnoses.errDomainHyphenStart;
                          flag6 = true;
                          if (errDomainHyphenStart > dnsWarnNoRecord) {
                            dnsWarnNoRecord = errDomainHyphenStart;
                            flag6 = true;
                          }
                        }
                      } else {
                        let tmp104 = charCodeAtResult1 < 48 || charCodeAtResult1 > 122;
                        if (!tmp104) {
                          let tmp105 = charCodeAtResult1 > 57 && charCodeAtResult1 < 65;
                          tmp104 = tmp105;
                        }
                        if (!tmp104) {
                          let tmp106 = charCodeAtResult1 > 90 && charCodeAtResult1 < 97;
                          tmp104 = tmp106;
                        }
                        flag6 = false;
                        if (tmp104) {
                          let rfc5322Domain = obj7.diagnoses.rfc5322Domain;
                          flag6 = false;
                          if (rfc5322Domain > dnsWarnNoRecord) {
                            dnsWarnNoRecord = rfc5322Domain;
                            flag6 = false;
                          }
                        }
                      }
                    }
                    obj5.domain = obj5.domain + obj6;
                    let domains4 = obj10.domains;
                    domains4[num5] = domains4[num5] + obj6;
                    num12 = num4 + 1;
                    tmp39 = flag6;
                    tmp32 = localpart2;
                    contextFWS = localpart;
                    num11 = num;
                    diff = num2;
                    tmp38 = obj6;
                    flag5 = flag2;
                    num10 = num3;
                    num13 = num5;
                    tmp40 = str3;
                  }
                }
                let errExpectingATEXT = obj7.diagnoses.errExpectingATEXT;
                flag6 = false;
                if (errExpectingATEXT > dnsWarnNoRecord) {
                  dnsWarnNoRecord = errExpectingATEXT;
                  flag6 = false;
                }
              }
            }
          }
          if (0 === num4) {
            if (0 === num5) {
              let deprecatedFWS2 = obj7.diagnoses.deprecatedCFWSNearAt;
            } else {
              deprecatedFWS2 = obj7.diagnoses.deprecatedFWS;
            }
            let flag7 = flag2;
            if (deprecatedFWS2 > dnsWarnNoRecord) {
              dnsWarnNoRecord = deprecatedFWS2;
              flag7 = flag2;
            }
          } else {
            let cfwsFWS4 = obj7.diagnoses.cfwsFWS;
            flag7 = true;
            if (cfwsFWS4 > dnsWarnNoRecord) {
              dnsWarnNoRecord = cfwsFWS4;
              flag7 = true;
            }
          }
          let arr20 = items2.push(localpart);
          contextFWS = obj7.components.contextFWS;
          flag5 = flag7;
          tmp32 = localpart2;
          num11 = num;
          diff = tmp204;
          tmp38 = obj6;
          tmp39 = flag3;
          num10 = num3;
          num12 = num4;
          num13 = num5;
          tmp40 = obj6;
        }
      } else if (obj7.components.literal === localpart) {
        if ("]" === obj6) {
          if (dnsWarnNoRecord < obj7.categories.deprecated) {
            let literal = obj5.literal;
            let ipV4 = obj7.regex.ipV4;
            let match = ipV4.exec(literal);
            let num14 = -1;
            let tmp84 = match;
            if (match) {
              let index = match.index;
              tmp84 = 0 !== index;
              num14 = index;
            }
            let text = literal;
            if (tmp84) {
              text = `${arr4.slice(0, num14)}0:0`;
            }
            if (0 === num14) {
              let rfc5321AddressLiteral2 = obj7.diagnoses.rfc5321AddressLiteral;
              if (rfc5321AddressLiteral2 > dnsWarnNoRecord) {
                dnsWarnNoRecord = rfc5321AddressLiteral2;
              }
            } else {
              let str13 = text.slice(0, 5);
              if ("ipv6:" !== str13.toLowerCase()) {
                let rfc5322DomainLiteral2 = obj7.diagnoses.rfc5322DomainLiteral;
                if (rfc5322DomainLiteral2 > dnsWarnNoRecord) {
                  dnsWarnNoRecord = rfc5322DomainLiteral2;
                }
              } else {
                let str14 = text.slice(5);
                let maxIPv6Groups = obj7.maxIPv6Groups;
                let parts = str14.split(":");
                let index1 = str14.indexOf("::");
                if (~index1) {
                  if (index1 !== str14.lastIndexOf("::")) {
                    let rfc5322IPv62x2xColon = obj7.diagnoses.rfc5322IPv62x2xColon;
                    if (rfc5322IPv62x2xColon > dnsWarnNoRecord) {
                      dnsWarnNoRecord = rfc5322IPv62x2xColon;
                    }
                  } else {
                    let tmp85 = 0 !== index1 && index1 !== str14.length - 2;
                    let sum4 = maxIPv6Groups;
                    if (!tmp85) {
                      sum4 = maxIPv6Groups + 1;
                    }
                    if (parts.length > sum4) {
                      let rfc5322IPv6MaxGroups = obj7.diagnoses.rfc5322IPv6MaxGroups;
                      if (rfc5322IPv6MaxGroups > dnsWarnNoRecord) {
                        dnsWarnNoRecord = rfc5322IPv6MaxGroups;
                      }
                    } else if (parts.length === sum4) {
                      let deprecatedIPv6 = obj7.diagnoses.deprecatedIPv6;
                      if (deprecatedIPv6 > dnsWarnNoRecord) {
                        dnsWarnNoRecord = deprecatedIPv6;
                      }
                    }
                  }
                } else if (parts.length !== maxIPv6Groups) {
                  let rfc5322IPv6GroupCount = obj7.diagnoses.rfc5322IPv6GroupCount;
                  if (rfc5322IPv6GroupCount > dnsWarnNoRecord) {
                    dnsWarnNoRecord = rfc5322IPv6GroupCount;
                  }
                }
                if (":" === str14[0]) {
                  if (":" !== str14[1]) {
                    let rfc5322IPv6ColonStart = obj7.diagnoses.rfc5322IPv6ColonStart;
                    if (rfc5322IPv6ColonStart > dnsWarnNoRecord) {
                      dnsWarnNoRecord = rfc5322IPv6ColonStart;
                    }
                  }
                }
                if (":" === str14[str14.length - 1]) {
                  if (":" !== str14[str14.length - 2]) {
                    let rfc5322IPv6ColonEnd = obj7.diagnoses.rfc5322IPv6ColonEnd;
                    if (rfc5322IPv6ColonEnd > dnsWarnNoRecord) {
                      dnsWarnNoRecord = rfc5322IPv6ColonEnd;
                    }
                  }
                }
                let diagnoses = obj7.diagnoses;
                if (obj7.checkIpV6(parts)) {
                  let rfc5321AddressLiteral = diagnoses.rfc5321AddressLiteral;
                  if (rfc5321AddressLiteral > dnsWarnNoRecord) {
                    dnsWarnNoRecord = rfc5321AddressLiteral;
                  }
                } else {
                  let rfc5322IPv6BadCharacter = diagnoses.rfc5322IPv6BadCharacter;
                  if (rfc5322IPv6BadCharacter > dnsWarnNoRecord) {
                    dnsWarnNoRecord = rfc5322IPv6BadCharacter;
                  }
                }
              }
            }
          } else {
            let rfc5322DomainLiteral = obj7.diagnoses.rfc5322DomainLiteral;
            if (rfc5322DomainLiteral > dnsWarnNoRecord) {
              dnsWarnNoRecord = rfc5322DomainLiteral;
            }
          }
          obj5.domain = obj5.domain + obj6;
          let domains3 = obj10.domains;
          domains3[num5] = domains3[num5] + obj6;
          num12 = num4 + 1;
          contextFWS = items2.pop();
          tmp32 = localpart;
          num11 = num;
          diff = num2;
          tmp38 = obj6;
          flag5 = flag2;
          tmp39 = flag3;
          num10 = num3;
          num13 = num5;
          tmp40 = str3;
        } else if ("\\" === obj6) {
          let rfc5322DomainLiteralOBSDText2 = obj7.diagnoses.rfc5322DomainLiteralOBSDText;
          if (rfc5322DomainLiteralOBSDText2 > dnsWarnNoRecord) {
            dnsWarnNoRecord = rfc5322DomainLiteralOBSDText2;
          }
          let arr21 = items2.push(localpart);
          contextFWS = obj7.components.contextQuotedPair;
          tmp32 = localpart2;
          num11 = num;
          diff = num2;
          tmp38 = obj6;
          flag5 = flag2;
          tmp39 = flag3;
          num10 = num3;
          num12 = num4;
          num13 = num5;
          tmp40 = str3;
        } else {
          if ("\r" === obj6) {
            let sum5 = num2 + 1;
            if (length !== sum5) {
              let tmp75 = sum5;
            }
            let errCRNoLF4 = obj7.diagnoses.errCRNoLF;
            tmp32 = localpart2;
            contextFWS = localpart;
            num11 = num;
            diff = sum5;
            tmp38 = obj6;
            flag5 = flag2;
            tmp39 = flag3;
            num10 = num3;
            num12 = num4;
            num13 = num5;
            tmp40 = str3;
            if (errCRNoLF4 > dnsWarnNoRecord) {
              dnsWarnNoRecord = errCRNoLF4;
              tmp32 = localpart2;
              contextFWS = localpart;
              num11 = num;
              diff = sum5;
              tmp38 = obj6;
              flag5 = flag2;
              tmp39 = flag3;
              num10 = num3;
              num12 = num4;
              num13 = num5;
              tmp40 = str3;
            }
          } else {
            tmp75 = num2;
            if (" " !== obj6) {
              tmp75 = num2;
              if ("\t" !== obj6) {
                let charCodeAtResult2 = obj6.charCodeAt(0);
                if (charCodeAtResult2 <= 127) {
                  if (0 !== charCodeAtResult2) {
                    if ("[" !== obj6) {
                      let tmp71 = charCodeAtResult2 < 33 || 127 === charCodeAtResult2;
                      if (tmp71) {
                        let rfc5322DomainLiteralOBSDText = obj7.diagnoses.rfc5322DomainLiteralOBSDText;
                        if (rfc5322DomainLiteralOBSDText > dnsWarnNoRecord) {
                          dnsWarnNoRecord = rfc5322DomainLiteralOBSDText;
                        }
                      }
                      obj5.literal = obj5.literal + obj6;
                      obj5.domain = obj5.domain + obj6;
                      let domains2 = obj10.domains;
                      domains2[num5] = domains2[num5] + obj6;
                      num12 = num4 + 1;
                      tmp32 = localpart2;
                      contextFWS = localpart;
                      num11 = num;
                      diff = num2;
                      tmp38 = obj6;
                      flag5 = flag2;
                      tmp39 = flag3;
                      num10 = num3;
                      num13 = num5;
                      tmp40 = str3;
                    }
                  }
                }
                let errExpectingDTEXT = obj7.diagnoses.errExpectingDTEXT;
                tmp32 = localpart2;
                contextFWS = localpart;
                num11 = num;
                diff = num2;
                tmp38 = obj6;
                flag5 = flag2;
                tmp39 = flag3;
                num10 = num3;
                num12 = num4;
                num13 = num5;
                tmp40 = str3;
                if (errExpectingDTEXT > dnsWarnNoRecord) {
                  dnsWarnNoRecord = errExpectingDTEXT;
                  tmp32 = localpart2;
                  contextFWS = localpart;
                  num11 = num;
                  diff = num2;
                  tmp38 = obj6;
                  flag5 = flag2;
                  tmp39 = flag3;
                  num10 = num3;
                  num12 = num4;
                  num13 = num5;
                  tmp40 = str3;
                }
              }
            }
          }
          let cfwsFWS3 = obj7.diagnoses.cfwsFWS;
          if (cfwsFWS3 > dnsWarnNoRecord) {
            dnsWarnNoRecord = cfwsFWS3;
          }
          let arr22 = items2.push(localpart);
          contextFWS = obj7.components.contextFWS;
          tmp32 = localpart2;
          num11 = num;
          diff = tmp75;
          tmp38 = obj6;
          flag5 = flag2;
          tmp39 = flag3;
          num10 = num3;
          num12 = num4;
          num13 = num5;
          tmp40 = obj6;
        }
      } else if (obj7.components.contextQuotedString === localpart) {
        if ("\\" === obj6) {
          let arr23 = items2.push(localpart);
          contextFWS = obj7.components.contextQuotedPair;
          tmp32 = localpart2;
          num11 = num;
          diff = num2;
          tmp38 = obj6;
          flag5 = flag2;
          tmp39 = flag3;
          num10 = num3;
          num12 = num4;
          num13 = num5;
          tmp40 = str3;
        } else {
          if ("\r" === obj6) {
            let sum6 = num2 + 1;
            if (length !== sum6) {
              let tmp199 = sum6;
            }
            let errCRNoLF3 = obj7.diagnoses.errCRNoLF;
            tmp32 = localpart2;
            contextFWS = localpart;
            num11 = num;
            diff = sum6;
            tmp38 = obj6;
            flag5 = flag2;
            tmp39 = flag3;
            num10 = num3;
            num12 = num4;
            num13 = num5;
            tmp40 = str3;
            if (errCRNoLF3 > dnsWarnNoRecord) {
              dnsWarnNoRecord = errCRNoLF3;
              tmp32 = localpart2;
              contextFWS = localpart;
              num11 = num;
              diff = sum6;
              tmp38 = obj6;
              flag5 = flag2;
              tmp39 = flag3;
              num10 = num3;
              num12 = num4;
              num13 = num5;
              tmp40 = str3;
            }
          } else {
            tmp199 = num2;
            if ("\t" !== obj6) {
              if ('"' === obj6) {
                obj5.local = obj5.local + obj6;
                let locals3 = obj10.locals;
                locals3[num5] = locals3[num5] + obj6;
                num12 = num4 + 1;
                contextFWS = items2.pop();
                tmp32 = localpart;
                num11 = num;
                diff = num2;
                tmp38 = obj6;
                flag5 = flag2;
                tmp39 = flag3;
                num10 = num3;
                num13 = num5;
                tmp40 = str3;
              } else {
                let charCodeAtResult3 = obj6.charCodeAt(0);
                if (charCodeAtResult3 <= 127) {
                  if (0 !== charCodeAtResult3) {
                    if (10 !== charCodeAtResult3) {
                      let tmp63 = charCodeAtResult3 < 32 || 127 === charCodeAtResult3;
                      if (tmp63) {
                        let deprecatedQTEXT = obj7.diagnoses.deprecatedQTEXT;
                        if (deprecatedQTEXT > dnsWarnNoRecord) {
                          dnsWarnNoRecord = deprecatedQTEXT;
                        }
                      }
                    }
                    obj5.local = obj5.local + obj6;
                    let locals2 = obj10.locals;
                    locals2[num5] = locals2[num5] + obj6;
                    num12 = num4 + 1;
                    tmp32 = localpart2;
                    contextFWS = localpart;
                    num11 = num;
                    diff = num2;
                    tmp38 = obj6;
                    flag5 = flag2;
                    tmp39 = flag3;
                    num10 = num3;
                    num13 = num5;
                    tmp40 = str3;
                  }
                }
                let errExpectingQTEXT = obj7.diagnoses.errExpectingQTEXT;
                if (errExpectingQTEXT > dnsWarnNoRecord) {
                  dnsWarnNoRecord = errExpectingQTEXT;
                }
              }
            }
          }
          obj5.local = `${obj4.local} `;
          let locals4 = obj10.locals;
          locals4[num5] = `${locals4[num5]} `;
          let cfwsFWS2 = obj7.diagnoses.cfwsFWS;
          if (cfwsFWS2 > dnsWarnNoRecord) {
            dnsWarnNoRecord = cfwsFWS2;
          }
          num12 = num4 + 1;
          let arr24 = items2.push(localpart);
          contextFWS = obj7.components.contextFWS;
          tmp32 = localpart2;
          num11 = num;
          diff = tmp199;
          tmp38 = obj6;
          flag5 = flag2;
          tmp39 = flag3;
          num10 = num3;
          num13 = num5;
          tmp40 = obj6;
        }
      } else if (obj7.components.contextQuotedPair === localpart) {
        let charCodeAtResult4 = obj6.charCodeAt(0);
        if (charCodeAtResult4 > 127) {
          let errExpectingQPair = obj7.diagnoses.errExpectingQPair;
          if (errExpectingQPair > dnsWarnNoRecord) {
            dnsWarnNoRecord = errExpectingQPair;
          }
        } else {
          let tmp53 = (charCodeAtResult4 < 31 && 9 !== charCodeAtResult4) || 127 === charCodeAtResult4;
          if (tmp53) {
            let deprecatedQP = obj7.diagnoses.deprecatedQP;
            if (deprecatedQP > dnsWarnNoRecord) {
              dnsWarnNoRecord = deprecatedQP;
            }
          }
        }
        let arr25 = items2.pop();
        let text1 = `\\${obj6}`;
        tmp32 = localpart;
        contextFWS = arr25;
        num11 = num;
        diff = num2;
        tmp38 = text1;
        flag5 = flag2;
        tmp39 = flag3;
        num10 = num3;
        num12 = num4;
        num13 = num5;
        tmp40 = str3;
        if (obj7.components.contextComment !== arr25) {
          if (obj7.components.contextQuotedString === arr25) {
            obj5.local = obj5.local + `\\${obj6}`;
            let locals = obj10.locals;
            locals[num5] = locals[num5] + `\\${obj6}`;
            num12 = num4 + 2;
            tmp32 = localpart;
            contextFWS = arr25;
            num11 = num;
            diff = num2;
            tmp38 = text1;
            flag5 = flag2;
            tmp39 = flag3;
            num10 = num3;
            num13 = num5;
            tmp40 = str3;
          } else if (obj7.components.literal === arr25) {
            obj5.domain = obj5.domain + `\\${obj6}`;
            let domains = obj10.domains;
            domains[num5] = domains[num5] + `\\${obj6}`;
            num12 = num4 + 2;
            tmp32 = localpart;
            contextFWS = arr25;
            num11 = num;
            diff = num2;
            tmp38 = text1;
            flag5 = flag2;
            tmp39 = flag3;
            num10 = num3;
            num13 = num5;
            tmp40 = str3;
          } else {
            let tmp58 = globalThis;
            let _Error2 = Error;
            let str5 = "quoted pair logic invoked in an invalid context: ";
            let tmp59 = new.target;
            let tmp60 = new.target;
            let error2 = new Error("quoted pair logic invoked in an invalid context: " + arr25);
            throw error2;
          }
        }
      } else if (obj7.components.contextComment === localpart) {
        if ("(" === obj6) {
          let arr26 = items2.push(localpart);
          contextFWS = obj7.components.contextComment;
          tmp32 = localpart2;
          num11 = num;
          diff = num2;
          tmp38 = obj6;
          flag5 = flag2;
          tmp39 = flag3;
          num10 = num3;
          num12 = num4;
          num13 = num5;
          tmp40 = str3;
        } else if (")" === obj6) {
          contextFWS = items2.pop();
          tmp32 = localpart;
          num11 = num;
          diff = num2;
          tmp38 = obj6;
          flag5 = flag2;
          tmp39 = flag3;
          num10 = num3;
          num12 = num4;
          num13 = num5;
          tmp40 = str3;
        } else if ("\\" === obj6) {
          let arr27 = items2.push(localpart);
          contextFWS = obj7.components.contextQuotedPair;
          tmp32 = localpart2;
          num11 = num;
          diff = num2;
          tmp38 = obj6;
          flag5 = flag2;
          tmp39 = flag3;
          num10 = num3;
          num12 = num4;
          num13 = num5;
          tmp40 = str3;
        } else {
          if ("\r" === obj6) {
            let sum7 = num2 + 1;
            if (length !== sum7) {
              let tmp46 = sum7;
            }
            let errCRNoLF2 = obj7.diagnoses.errCRNoLF;
            tmp32 = localpart2;
            contextFWS = localpart;
            num11 = num;
            diff = sum7;
            tmp38 = obj6;
            flag5 = flag2;
            tmp39 = flag3;
            num10 = num3;
            num12 = num4;
            num13 = num5;
            tmp40 = str3;
            if (errCRNoLF2 > dnsWarnNoRecord) {
              dnsWarnNoRecord = errCRNoLF2;
              tmp32 = localpart2;
              contextFWS = localpart;
              num11 = num;
              diff = sum7;
              tmp38 = obj6;
              flag5 = flag2;
              tmp39 = flag3;
              num10 = num3;
              num12 = num4;
              num13 = num5;
              tmp40 = str3;
            }
          } else {
            tmp46 = num2;
            if (" " !== obj6) {
              tmp46 = num2;
              if ("\t" !== obj6) {
                let charCodeAtResult5 = obj6.charCodeAt(0);
                if (charCodeAtResult5 <= 127) {
                  if (0 !== charCodeAtResult5) {
                    if (10 !== charCodeAtResult5) {
                      let tmp42 = charCodeAtResult5 < 32 || 127 === charCodeAtResult5;
                      tmp32 = localpart2;
                      contextFWS = localpart;
                      num11 = num;
                      diff = num2;
                      tmp38 = obj6;
                      flag5 = flag2;
                      tmp39 = flag3;
                      num10 = num3;
                      num12 = num4;
                      num13 = num5;
                      tmp40 = str3;
                      if (tmp42) {
                        let deprecatedCTEXT = obj7.diagnoses.deprecatedCTEXT;
                        tmp32 = localpart2;
                        contextFWS = localpart;
                        num11 = num;
                        diff = num2;
                        tmp38 = obj6;
                        flag5 = flag2;
                        tmp39 = flag3;
                        num10 = num3;
                        num12 = num4;
                        num13 = num5;
                        tmp40 = str3;
                        if (deprecatedCTEXT > dnsWarnNoRecord) {
                          dnsWarnNoRecord = deprecatedCTEXT;
                          tmp32 = localpart2;
                          contextFWS = localpart;
                          num11 = num;
                          diff = num2;
                          tmp38 = obj6;
                          flag5 = flag2;
                          tmp39 = flag3;
                          num10 = num3;
                          num12 = num4;
                          num13 = num5;
                          tmp40 = str3;
                        }
                      }
                    }
                  }
                }
                let errExpectingCTEXT = obj7.diagnoses.errExpectingCTEXT;
                tmp32 = localpart2;
                contextFWS = localpart;
                num11 = num;
                diff = num2;
                tmp38 = obj6;
                flag5 = flag2;
                tmp39 = flag3;
                num10 = num3;
                num12 = num4;
                num13 = num5;
                tmp40 = str3;
                if (errExpectingCTEXT > dnsWarnNoRecord) {
                  dnsWarnNoRecord = errExpectingCTEXT;
                  tmp32 = localpart2;
                  contextFWS = localpart;
                  num11 = num;
                  diff = num2;
                  tmp38 = obj6;
                  flag5 = flag2;
                  tmp39 = flag3;
                  num10 = num3;
                  num12 = num4;
                  num13 = num5;
                  tmp40 = str3;
                }
              }
            }
          }
          let cfwsFWS = obj7.diagnoses.cfwsFWS;
          if (cfwsFWS > dnsWarnNoRecord) {
            dnsWarnNoRecord = cfwsFWS;
          }
          let arr28 = items2.push(localpart);
          contextFWS = obj7.components.contextFWS;
          tmp32 = localpart2;
          num11 = num;
          diff = tmp46;
          tmp38 = obj6;
          flag5 = flag2;
          tmp39 = flag3;
          num10 = num3;
          num12 = num4;
          num13 = num5;
          tmp40 = obj6;
        }
      } else if (obj7.components.contextFWS === localpart) {
        let tmp30 = "\r" === str3;
        let num9 = num3;
        if (!tmp30) {
          if ("\r" === obj6) {
            let sum8 = num2 + 1;
            let tmp36 = length !== sum8 && "\n" === arg0[sum8];
            tmp32 = localpart2;
            contextFWS = localpart;
            diff = sum8;
            num10 = num9;
            if (!tmp36) {
              let errCRNoLF = obj7.diagnoses.errCRNoLF;
              tmp32 = localpart2;
              contextFWS = localpart;
              diff = sum8;
              num10 = num9;
              if (errCRNoLF > dnsWarnNoRecord) {
                dnsWarnNoRecord = errCRNoLF;
                tmp32 = localpart2;
                contextFWS = localpart;
                diff = sum8;
                num10 = num9;
              }
            }
          } else {
            tmp32 = localpart2;
            contextFWS = localpart;
            diff = num2;
            num10 = num9;
            if (" " !== obj6) {
              tmp32 = localpart2;
              contextFWS = localpart;
              diff = num2;
              num10 = num9;
              if ("\t" !== obj6) {
                if (tmp30) {
                  let errFWSCRLFEnd = obj7.diagnoses.errFWSCRLFEnd;
                  if (errFWSCRLFEnd > dnsWarnNoRecord) {
                    dnsWarnNoRecord = errFWSCRLFEnd;
                  }
                }
                contextFWS = items2.pop();
                diff = num2 - 1;
                tmp32 = localpart;
                num10 = 0;
              }
            }
          }
          num11 = num;
          tmp38 = obj6;
          flag5 = flag2;
          tmp39 = flag3;
          num12 = num4;
          num13 = num5;
          tmp40 = obj6;
        } else if ("\r" === obj6) {
          let errFWSCRLFx2 = obj7.diagnoses.errFWSCRLFx2;
          tmp32 = localpart2;
          contextFWS = localpart;
          num11 = num;
          diff = num2;
          tmp38 = obj6;
          flag5 = flag2;
          tmp39 = flag3;
          num10 = num3;
          num12 = num4;
          num13 = num5;
          tmp40 = str3;
          if (errFWSCRLFx2 > dnsWarnNoRecord) {
            dnsWarnNoRecord = errFWSCRLFx2;
            tmp32 = localpart2;
            contextFWS = localpart;
            num11 = num;
            diff = num2;
            tmp38 = obj6;
            flag5 = flag2;
            tmp39 = flag3;
            num10 = num3;
            num12 = num4;
            num13 = num5;
            tmp40 = str3;
          }
        } else {
          let sum9 = num3 + 1;
          num9 = 1;
          if (1 < sum9) {
            let deprecatedFWS = obj7.diagnoses.deprecatedFWS;
            num9 = sum9;
            if (deprecatedFWS > dnsWarnNoRecord) {
              dnsWarnNoRecord = deprecatedFWS;
              num9 = sum9;
            }
          }
        }
      } else {
        let tmp25 = globalThis;
        let _Error = Error;
        let str4 = "unknown context: ";
        let tmp26 = new.target;
        let tmp27 = new.target;
        let error3 = new Error("unknown context: " + localpart);
        throw error3;
      }
      tmp13 = contextFWS;
      num6 = num11;
      tmp14 = tmp38;
      flag4 = tmp39;
      num7 = num12;
      num8 = num13;
      obj3 = obj7;
      if (dnsWarnNoRecord <= obj7.categories.rfc5322) {
        num2 = diff + 1;
        localpart2 = tmp32;
        localpart = contextFWS;
        num = num11;
        flag2 = flag5;
        flag3 = tmp39;
        num3 = num10;
        num4 = num12;
        num5 = num13;
        str3 = tmp40;
        tmp13 = contextFWS;
        num6 = num11;
        tmp14 = tmp38;
        flag4 = tmp39;
        num7 = num12;
        num8 = num13;
        obj3 = obj7;
      }
    }
    const _Error3 = Error;
    const error4 = new Error("more atext found where none is allowed, but unrecognized prev context: " + localpart2);
    throw error4;
  }
  if (dnsWarnNoRecord < obj3.categories.rfc5322) {
    if (tmp13 === obj3.components.contextQuotedString) {
      const errUnclosedQuotedString = obj3.diagnoses.errUnclosedQuotedString;
      if (errUnclosedQuotedString > dnsWarnNoRecord) {
        dnsWarnNoRecord = errUnclosedQuotedString;
      }
    } else if (tmp13 === obj3.components.contextQuotedPair) {
      const errBackslashEnd = obj3.diagnoses.errBackslashEnd;
      if (errBackslashEnd > dnsWarnNoRecord) {
        dnsWarnNoRecord = errBackslashEnd;
      }
    } else if (tmp13 === obj3.components.contextComment) {
      const errUnclosedComment = obj3.diagnoses.errUnclosedComment;
      if (errUnclosedComment > dnsWarnNoRecord) {
        dnsWarnNoRecord = errUnclosedComment;
      }
    } else if (tmp13 === obj3.components.literal) {
      const errUnclosedDomainLiteral = obj3.diagnoses.errUnclosedDomainLiteral;
      if (errUnclosedDomainLiteral > dnsWarnNoRecord) {
        dnsWarnNoRecord = errUnclosedDomainLiteral;
      }
    } else if ("\r" === tmp14) {
      const errFWSCRLFEnd2 = obj3.diagnoses.errFWSCRLFEnd;
      if (errFWSCRLFEnd2 > dnsWarnNoRecord) {
        dnsWarnNoRecord = errFWSCRLFEnd2;
      }
    } else if (0 === obj5.domain.length) {
      const errNoDomain = obj3.diagnoses.errNoDomain;
      if (errNoDomain > dnsWarnNoRecord) {
        dnsWarnNoRecord = errNoDomain;
      }
    } else if (0 === num7) {
      const errDotEnd2 = obj3.diagnoses.errDotEnd;
      if (errDotEnd2 > dnsWarnNoRecord) {
        dnsWarnNoRecord = errDotEnd2;
      }
    } else if (flag4) {
      const errDomainHyphenEnd2 = obj3.diagnoses.errDomainHyphenEnd;
      if (errDomainHyphenEnd2 > dnsWarnNoRecord) {
        dnsWarnNoRecord = errDomainHyphenEnd2;
      }
    } else if (obj5.domain.length > 255) {
      const rfc5322DomainTooLong = obj3.diagnoses.rfc5322DomainTooLong;
      if (rfc5322DomainTooLong > dnsWarnNoRecord) {
        dnsWarnNoRecord = rfc5322DomainTooLong;
      }
    } else if (obj5.local.length + obj5.domain.length + 1 > 254) {
      const rfc5322TooLong = obj3.diagnoses.rfc5322TooLong;
      if (rfc5322TooLong > dnsWarnNoRecord) {
        dnsWarnNoRecord = rfc5322TooLong;
      }
    } else if (63 < num7) {
      const rfc5322LabelTooLong2 = obj3.diagnoses.rfc5322LabelTooLong;
      if (rfc5322LabelTooLong2 > dnsWarnNoRecord) {
        dnsWarnNoRecord = rfc5322LabelTooLong2;
      }
    } else {
      if (obj2.minDomainAtoms) {
        if (obj10.domains.length < obj2.minDomainAtoms) {
          const errDomainTooShort = obj3.diagnoses.errDomainTooShort;
          if (errDomainTooShort > dnsWarnNoRecord) {
            dnsWarnNoRecord = errDomainTooShort;
          }
        }
      }
      if (obj2.tldWhitelist) {
        if (!obj3.validDomain(obj10.domains[num8], obj2)) {
          const errUnknownTLD = obj3.diagnoses.errUnknownTLD;
          if (errUnknownTLD > dnsWarnNoRecord) {
            dnsWarnNoRecord = errUnknownTLD;
          }
        }
      }
    }
  }
  c7 = false;
  c8 = false;
  function finish() {
    let tmp = !c7;
    if (!c7) {
      tmp = dnsWarnNoRecord < closure_2_2.categories.dnsWarn;
    }
    if (tmp) {
      if (obj.charCodeAt(0) <= 57) {
        const rfc5321TLDNumeric = closure_2_2.diagnoses.rfc5321TLDNumeric;
        if (typeof updateResult === "function") {
          if (rfc5321TLDNumeric > dnsWarnNoRecord) {
            dnsWarnNoRecord = rfc5321TLDNumeric;
          }
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      } else if (0 === sum2) {
        const rfc5321TLD = closure_2_2.diagnoses.rfc5321TLD;
        if (typeof updateResult === "function") {
          if (rfc5321TLD > dnsWarnNoRecord) {
            dnsWarnNoRecord = rfc5321TLD;
          }
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      }
      obj = obj10.domains[sum2];
    }
    if (dnsWarnNoRecord < valid) {
      dnsWarnNoRecord = closure_2_2.diagnoses.valid;
    }
    if (errorLevel) {
      let tmp15 = tmp13;
    } else {
      tmp15 = tmp13 < closure_2_2.defaultThreshold;
    }
    if (domain) {
      if (c8) {
        domain(tmp15);
      } else {
        closure_2_2.defer(domain.bind(null, tmp15));
      }
    }
    return tmp15;
  }
  if (obj2.checkDNS) {
    if (dnsWarnNoRecord < obj3.categories.dnsWarn) {
      if (0 === num6) {
        obj5.domain = `${obj4.domain}.`;
      }
      domain = obj5.domain;
      const mx = errorLevel.resolveMx(domain, (code, arg1) => {
        if (code) {
          if (code.code !== closure_3_1.NODATA) {
            dnsWarnNoRecord = closure_3_2.diagnoses.dnsWarnNoRecord;
            if (typeof closure_2_4 === "function") {
              if (dnsWarnNoRecord > closure_2_3) {
                closure_2_3 = dnsWarnNoRecord;
              }
              return closure_2_9();
            } else {
              throw new TypeError("Trying to call a non-function");
            }
          }
        }
        if (arg1) {
          if (arg1.length) {
            closure_2_7 = true;
            return closure_2_9();
          }
        }
        closure_0 = 3;
        c1 = false;
        const dnsWarnNoMXRecord = closure_3_2.diagnoses.dnsWarnNoMXRecord;
        if (typeof closure_2_4 === "function") {
          if (dnsWarnNoMXRecord > closure_2_3) {
            closure_2_3 = dnsWarnNoMXRecord;
          }
          function handleRecords(arg0, arg1) {
            if (!c1) {
              closure_0 = closure_0 - 1;
              if (arg1) {
                if (arg1.length) {
                  c1 = true;
                  return finish();
                }
              }
              if (0 === closure_0) {
                dnsWarnNoRecord = closure_3_2.diagnoses.dnsWarnNoRecord;
                if (typeof updateResult === "function") {
                  c1 = true;
                  finish();
                } else {
                  throw new TypeError("Trying to call a non-function");
                }
              }
            }
          }
          const cname = closure_3_1.resolveCname(domain, handleRecords);
          closure_3_1.resolve4(domain, handleRecords);
          closure_3_1.resolve6(domain, handleRecords);
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      });
      c8 = true;
    }
  }
  c8 = true;
  return finish();
}

function t(options) {
  this.options = options;
}

function f136688(lastyear, lastmonth) {
  let length;
  const self = this;
  const options = this.options;
  if (lastyear !== this.lastyear) {
    if (typeof closure_1_15 === "function") {
      const _Date = Date;
      const _Date2 = Date;
      const _Date3 = Date;
      const date = new Date(Date.UTC(lastyear, 0, 1, 0, 0, 0));
      if (typeof closure_1_20 === "function") {
        const result = lastyear % 4;
        let tmp7 = result === 0;
        if (result === 0) {
          tmp7 = lastyear % 100 !== 0;
        }
        if (!tmp7) {
          tmp7 = lastyear % 400 === 0;
        }
        let num11 = 365;
        let num12 = 365;
        if (tmp7) {
          num12 = 366;
        }
        const sum = lastyear + 1;
        if (typeof closure_1_20 === "function") {
          const result1 = sum % 4;
          let tmp10 = result1 === 0;
          if (result1 === 0) {
            tmp10 = sum % 100 !== 0;
          }
          if (!tmp10) {
            tmp10 = sum % 400 === 0;
          }
          let num15 = num11;
          if (tmp10) {
            num15 = 366;
          }
          if (typeof closure_1_24 === "function") {
            if (typeof closure_1_23 === "function") {
              const diff = tmp14 - 60 * date.getTimezoneOffset() * 1000;
              if (typeof tmp15 === "function") {
                const _Math = Math;
                if (typeof closure_1_27 === "function") {
                  const tmp21 = closure_1_19[date.getUTCDay(date)];
                  const obj = { yearlen: num12, nextyearlen: num15, yearordinal: tmp18, yearweekday: tmp21 };
                  if (typeof closure_1_20 === "function") {
                    const result2 = lastyear % 4;
                    let tmp24 = result2 === 0;
                    if (result2 === 0) {
                      tmp24 = lastyear % 100 !== 0;
                    }
                    if (!tmp24) {
                      tmp24 = lastyear % 400 === 0;
                    }
                    let num20 = num11;
                    if (tmp24) {
                      num20 = 366;
                    }
                    if (typeof tmp160 === "function") {
                      const _Date4 = Date;
                      const _Date5 = Date;
                      const _Date6 = Date;
                      const date1 = new Date(Date.UTC(lastyear, 0, 1, 0, 0, 0));
                      if (typeof tmp19 === "function") {
                        const tmp29 = closure_1_19[date1.getUTCDay(date1)];
                        if (num11 === num20) {
                          const obj2 = {
                            mmask: closure_1_58,
                            mdaymask: closure_1_61,
                            nmdaymask: closure_1_63,
                            wdaymask: closure_1_66.slice(tmp29),
                            mrange: closure_1_65,
                          };
                          let obj3 = obj2;
                        } else {
                          obj3 = {
                            mmask: closure_1_59,
                            mdaymask: closure_1_60,
                            nmdaymask: closure_1_62,
                            wdaymask: closure_1_66.slice(tmp29),
                            mrange: closure_1_64,
                          };
                        }
                        const tmp22Result = closure_1_38(closure_1_38(obj, obj3), { wnomask: null });
                        const byweekno = options.byweekno;
                        if (typeof closure_1_12 === "function") {
                          if (typeof closure_1_3 === "function") {
                            let tmp45 = !tmp44;
                            if (null != byweekno) {
                              tmp45 = 0 === byweekno.length;
                            }
                            if (!tmp45) {
                              if (typeof closure_1_8 === "function") {
                                const sum1 = num12 + 7;
                                const items = [];
                                if (closure_1_6(0)) {
                                  let num28 = 0;
                                  if (0 < sum1) {
                                    do {
                                      let items1 = [];
                                      items[num28] = items1.concat(0);
                                      num28 = num28 + 1;
                                    } while (num28 < sum1);
                                  }
                                } else {
                                  let num27 = 0;
                                  if (0 < sum1) {
                                    do {
                                      items[num27] = 0;
                                      num27 = num27 + 1;
                                    } while (num27 < sum1);
                                  }
                                }
                                tmp22Result.wnomask = items;
                                if (typeof closure_1_10 === "function") {
                                  const result3 = (7 - tmp21 + options.wkst) % 7;
                                  let sum2 = result3;
                                  if (result3 * 7 < 0) {
                                    sum2 = result3 + 7;
                                  }
                                  if (4 <= sum2) {
                                    if (typeof closure_1_10 === "function") {
                                      const result4 = (tmp21 - options.wkst) % 7;
                                      let sum3 = result4;
                                      if (result4 * 7 < 0) {
                                        sum3 = result4 + 7;
                                      }
                                      let sum4 = tmp53 + sum3;
                                      let num29 = 0;
                                    } else {
                                      throw new TypeError("Trying to call a non-function");
                                    }
                                  } else {
                                    sum4 = num12 - sum2;
                                    num29 = sum2;
                                  }
                                  const _Math2 = Math;
                                  if (typeof closure_1_10 === "function") {
                                    const result5 = sum4 % 7;
                                    let sum5 = result5;
                                    if (result5 * 7 < 0) {
                                      sum5 = result5 + 7;
                                    }
                                    const _Math3 = Math;
                                    const rounded = Math.floor(tmp56 + sum5 / 4);
                                    const diff1 = 7 - sum2;
                                    for (let num30 = 0; num30 < options.byweekno.length; num30 = num30 + 1) {
                                      let tmp61 = options.byweekno[num30];
                                      let sum6 = tmp61;
                                      if (tmp61 < 0) {
                                        sum6 = tmp61 + (rounded + 1);
                                      }
                                      if (sum6 > 0) {
                                        if (sum6 <= rounded) {
                                          let diff2 = num29;
                                          if (sum6 > 1) {
                                            let sum7 = num29 + 7 * (sum6 - 1);
                                            diff2 = sum7;
                                            if (num29 !== sum2) {
                                              diff2 = sum7 - diff1;
                                            }
                                          }
                                          tmp22Result.wnomask[diff2] = 1;
                                          let sum8 = diff2 + 1;
                                          let num31 = 0;
                                          if (tmp22Result.wdaymask[sum8] !== options.wkst) {
                                            let sum9 = num31 + 1;
                                            while (sum9 < 7) {
                                              tmp22Result.wnomask[sum8] = 1;
                                              let sum10 = sum8 + 1;
                                              num31 = sum9;
                                              sum8 = sum10;
                                              if (tmp22Result.wdaymask[sum10] === options.wkst) {
                                                break;
                                              }
                                            }
                                          }
                                        }
                                      }
                                    }
                                    const byweekno1 = options.byweekno;
                                    if (typeof closure_1_14 === "function") {
                                      if (typeof closure_1_13 === "function") {
                                        if (typeof closure_1_12 === "function") {
                                          if (typeof closure_1_3 === "function") {
                                            let tmp75 = !tmp74;
                                            if (null != byweekno1) {
                                              tmp75 = 0 === byweekno1.length;
                                            }
                                            let tmp76 = !tmp75;
                                            if (tmp76) {
                                              tmp76 = -1 !== byweekno1.indexOf(1);
                                            }
                                            if (tmp76) {
                                              const sum11 = num29 + 7 * rounded;
                                              let diff3 = sum11;
                                              if (num29 !== sum2) {
                                                diff3 = sum11 - diff1;
                                              }
                                              if (diff3 < num12) {
                                                tmp22Result.wnomask[diff3] = 1;
                                                let sum12 = diff3 + 1;
                                                let num33 = 0;
                                                if (tmp22Result.wdaymask[sum12] !== options.wkst) {
                                                  const sum13 = num33 + 1;
                                                  while (sum13 < 7) {
                                                    tmp22Result.wnomask[sum12] = 1;
                                                    let sum14 = sum12 + 1;
                                                    sum12 = sum14;
                                                    num33 = sum13;
                                                    if (tmp22Result.wdaymask[sum14] === options.wkst) {
                                                      break;
                                                    }
                                                  }
                                                }
                                              }
                                            }
                                            if (num29) {
                                              const byweekno2 = options.byweekno;
                                              if (typeof closure_1_14 === "function") {
                                                if (typeof closure_1_13 === "function") {
                                                  if (typeof closure_1_12 === "function") {
                                                    if (typeof closure_1_3 === "function") {
                                                      let tmp88 = !tmp87;
                                                      if (null != byweekno2) {
                                                        tmp88 = 0 === byweekno2.length;
                                                      }
                                                      let tmp89 = !tmp88;
                                                      if (tmp89) {
                                                        tmp89 = -1 !== byweekno2.indexOf(-1);
                                                      }
                                                      let num36 = -1;
                                                      if (!tmp89) {
                                                        const diff4 = lastyear - 1;
                                                        if (typeof closure_1_15 === "function") {
                                                          const _Date7 = Date;
                                                          const _Date8 = Date;
                                                          const _Date9 = Date;
                                                          const date2 = new Date(Date.UTC(diff4, 0, 1, 0, 0, 0));
                                                          if (typeof tmp90 === "function") {
                                                            const obj7 = closure_1_19[date2.getUTCDay(date2)];
                                                            if (typeof closure_1_10 === "function") {
                                                              const result6 = (7 - obj7.valueOf() + options.wkst) % 7;
                                                              let sum15 = result6;
                                                              if (result6 * 7 < 0) {
                                                                sum15 = result6 + 7;
                                                              }
                                                              const diff5 = lastyear - 1;
                                                              if (typeof closure_1_20 === "function") {
                                                                const result7 = diff5 % 4;
                                                                let tmp104 = result7 === 0;
                                                                if (result7 === 0) {
                                                                  tmp104 = diff5 % 100 !== 0;
                                                                }
                                                                if (!tmp104) {
                                                                  tmp104 = diff5 % 400 === 0;
                                                                }
                                                                if (tmp104) {
                                                                  num11 = 366;
                                                                }
                                                                if (4 <= sum15) {
                                                                  if (typeof closure_1_10 === "function") {
                                                                    const result8 = (obj7 - options.wkst) % 7;
                                                                    let sum16 = result8;
                                                                    if (result8 * 7 < 0) {
                                                                      sum16 = result8 + 7;
                                                                    }
                                                                    let sum17 = num11 + sum16;
                                                                  } else {
                                                                    throw new TypeError(
                                                                      "Trying to call a non-function",
                                                                    );
                                                                  }
                                                                } else {
                                                                  sum17 = num12 - num29;
                                                                }
                                                                if (typeof closure_1_10 === "function") {
                                                                  const result9 = sum17 % 7;
                                                                  let sum18 = result9;
                                                                  if (result9 * 7 < 0) {
                                                                    sum18 = result9 + 7;
                                                                  }
                                                                  num36 = tmp109(52 + sum18 / 4);
                                                                } else {
                                                                  throw new TypeError("Trying to call a non-function");
                                                                }
                                                              } else {
                                                                throw new TypeError("Trying to call a non-function");
                                                              }
                                                            } else {
                                                              throw new TypeError("Trying to call a non-function");
                                                            }
                                                          } else {
                                                            throw new TypeError("Trying to call a non-function");
                                                          }
                                                        } else {
                                                          throw new TypeError("Trying to call a non-function");
                                                        }
                                                      }
                                                      const byweekno3 = options.byweekno;
                                                      if (typeof tmp83 === "function") {
                                                        if (typeof tmp84 === "function") {
                                                          if (typeof tmp85 === "function") {
                                                            if (typeof tmp86 === "function") {
                                                              let tmp113 = !tmp112;
                                                              if (null != byweekno3) {
                                                                tmp113 = 0 === byweekno3.length;
                                                              }
                                                              if (tmp114) {
                                                                let num45 = 0;
                                                                if (0 < num29) {
                                                                  do {
                                                                    tmp22Result.wnomask[num45] = 1;
                                                                    num45 = num45 + 1;
                                                                  } while (num45 < num29);
                                                                }
                                                              }
                                                              tmp114 = !tmp113 && -1 !== byweekno3.indexOf(num36);
                                                            } else {
                                                              throw new TypeError("Trying to call a non-function");
                                                            }
                                                          } else {
                                                            throw new TypeError("Trying to call a non-function");
                                                          }
                                                        } else {
                                                          throw new TypeError("Trying to call a non-function");
                                                        }
                                                      } else {
                                                        throw new TypeError("Trying to call a non-function");
                                                      }
                                                    } else {
                                                      throw new TypeError("Trying to call a non-function");
                                                    }
                                                  } else {
                                                    throw new TypeError("Trying to call a non-function");
                                                  }
                                                } else {
                                                  throw new TypeError("Trying to call a non-function");
                                                }
                                              } else {
                                                throw new TypeError("Trying to call a non-function");
                                              }
                                            }
                                          } else {
                                            throw new TypeError("Trying to call a non-function");
                                          }
                                        } else {
                                          throw new TypeError("Trying to call a non-function");
                                        }
                                      } else {
                                        throw new TypeError("Trying to call a non-function");
                                      }
                                    } else {
                                      throw new TypeError("Trying to call a non-function");
                                    }
                                  } else {
                                    throw new TypeError("Trying to call a non-function");
                                  }
                                } else {
                                  throw new TypeError("Trying to call a non-function");
                                }
                              } else {
                                throw new TypeError("Trying to call a non-function");
                              }
                            }
                            self.yearinfo = tmp22Result;
                          } else {
                            throw new TypeError("Trying to call a non-function");
                          }
                        } else {
                          throw new TypeError("Trying to call a non-function");
                        }
                      } else {
                        throw new TypeError("Trying to call a non-function");
                      }
                    } else {
                      throw new TypeError("Trying to call a non-function");
                    }
                  } else {
                    throw new TypeError("Trying to call a non-function");
                  }
                } else {
                  throw new TypeError("Trying to call a non-function");
                }
              } else {
                throw new TypeError("Trying to call a non-function");
              }
            } else {
              throw new TypeError("Trying to call a non-function");
            }
          } else {
            throw new TypeError("Trying to call a non-function");
          }
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      } else {
        throw new TypeError("Trying to call a non-function");
      }
    } else {
      throw new TypeError("Trying to call a non-function");
    }
  }
  const bynweekday = options.bynweekday;
  if (typeof closure_1_13 === "function") {
    if (typeof closure_1_12 === "function") {
      if (typeof closure_1_3 === "function") {
        let tmp119 = !tmp118;
        if (null != bynweekday) {
          tmp119 = 0 === bynweekday.length;
        }
        if (!tmp119) {
          if (lastmonth !== self.lastmonth) {
            ({ yearlen, mrange, wdaymask } = self.yearinfo);
            const obj4 = { lastyear, lastmonth, nwdaymask: [] };
            const items2 = [];
            if (options.freq === closure_1_72.YEARLY) {
              const bymonth = options.bymonth;
              if (typeof tmp115 === "function") {
                if (typeof tmp116 === "function") {
                  let tmp123 = !tmp122;
                  if (null != bymonth) {
                    tmp123 = 0 === bymonth.length;
                  }
                  if (tmp123) {
                    const items3 = [0, yearlen];
                    const items4 = [items3];
                    let arr9 = items4;
                  } else {
                    let num48 = 0;
                    arr9 = items2;
                    if (0 < options.bymonth.length) {
                      do {
                        let tmp124 = options.bymonth[num48];
                        let arr = items2.push(mrange.slice(tmp124 - 1, tmp124 + 1));
                        num48 = num48 + 1;
                        arr9 = items2;
                        length = options.bymonth.length;
                      } while (num48 < length);
                    }
                  }
                } else {
                  throw new TypeError("Trying to call a non-function");
                }
              } else {
                throw new TypeError("Trying to call a non-function");
              }
            } else {
              arr9 = items2;
              if (options.freq === tmp121.MONTHLY) {
                const items5 = [mrange.slice(lastmonth - 1, lastmonth + 1)];
                arr9 = items5;
              }
            }
            if (typeof closure_1_12 === "function") {
              if (typeof closure_1_3 === "function") {
                if (0 !== arr9.length) {
                  if (typeof closure_1_8 === "function") {
                    const items6 = [];
                    if (closure_1_6(0)) {
                      let num54 = 0;
                      if (tmp129) {
                        do {
                          let items7 = [];
                          items6[num54] = items7.concat(0);
                          num54 = num54 + 1;
                        } while (num54 < yearlen);
                      }
                    } else {
                      let num52 = 0;
                      if (tmp129) {
                        do {
                          items6[num52] = 0;
                          num52 = num52 + 1;
                        } while (num52 < yearlen);
                      }
                    }
                    obj4.nwdaymask = items6;
                    let num57 = 0;
                    if (0 < arr9.length) {
                      while (true) {
                        let tmp130 = arr9[num57];
                        let first = tmp130[0];
                        let diff6 = tmp130[1] - 1;
                        let num58 = 0;
                        if (0 < options.bynweekday.length) {
                          while (true) {
                            [tmp134, tmp135] = options.bynweekday[num58];
                            if (tmp135 < 0) {
                              let sum19 = diff6 + 7 * (tmp135 + 1);
                              if (typeof closure_1_10 === "function") {
                                let result10 = (wdaymask[sum19] - tmp134) % 7;
                                let sum20 = result10;
                                if (result10 * 7 < 0) {
                                  sum20 = result10 + 7;
                                }
                                let diff7 = sum19 - sum20;
                              } else {
                                let str44 = "Trying to call a non-function";
                                throw new TypeError("Trying to call a non-function");
                              }
                            } else {
                              let sum21 = first + 7 * (tmp135 - 1);
                              if (typeof closure_1_10 !== "function") {
                                break;
                              } else {
                                let result11 = tmp139 % 7;
                                let sum22 = result11;
                                if (result11 * 7 < 0) {
                                  sum22 = result11 + 7;
                                }
                                diff7 = sum21 + sum22;
                              }
                            }
                            let tmp147 = first <= diff7 && diff7 <= diff6;
                            if (tmp147) {
                              obj4.nwdaymask[diff7] = 1;
                            }
                            num58 = num58 + 1;
                            continue;
                          }
                          let str43 = "Trying to call a non-function";
                          throw new TypeError("Trying to call a non-function");
                        }
                        num57 = num57 + 1;
                      }
                    }
                  } else {
                    throw new TypeError("Trying to call a non-function");
                  }
                }
                self.monthinfo = obj4;
              } else {
                throw new TypeError("Trying to call a non-function");
              }
            } else {
              throw new TypeError("Trying to call a non-function");
            }
          }
        }
        if (typeof closure_1_3 === "function") {
          if (null != options.byeaster) {
            let num59 = options.byeaster;
            if (undefined === num59) {
              num59 = 0;
            }
            const result12 = lastyear % 19;
            const _Math4 = Math;
            const rounded1 = Math.floor(lastyear / 100);
            const result13 = lastyear % 100;
            const _Math5 = Math;
            const _Math6 = Math;
            const rounded2 = Math.floor(rounded1 / 4);
            const result14 = rounded1 % 4;
            const _Math7 = Math;
            const _Math8 = Math;
            const result15 =
              Math.floor(
                19 * result12 +
                  rounded1 -
                  rounded2 -
                  Math.floor((rounded1 - Math.floor((rounded1 + 8) / 25) + 1) / 3) +
                  15,
              ) % 30;
            const _Math9 = Math;
            const _Math10 = Math;
            const result16 =
              Math.floor(32 + 2 * result14 + 2 * Math.floor(result13 / 4) - result15 - (result13 % 4)) % 7;
            const _Math11 = Math;
            const rounded3 = Math.floor((result12 + 11 * result15 + 22 * result16) / 451);
            const _Math12 = Math;
            const sum23 = result15 + result16;
            const _Date10 = Date;
            const _Date11 = Date;
            const _Math13 = Math;
            const items8 = [
              Math.ceil(
                (Date.UTC(
                  lastyear,
                  Math.floor((sum23 - 7 * rounded3 + 114) / 31) - 1,
                  ((sum23 - 7 * rounded3 + 114) % 31) + 1 + num59,
                ) -
                  Date.UTC(lastyear, 0, 1)) /
                  86400000,
              ),
            ];
            self.eastermask = items8;
            const UTCResult = Date.UTC(
              lastyear,
              Math.floor((sum23 - 7 * rounded3 + 114) / 31) - 1,
              ((sum23 - 7 * rounded3 + 114) % 31) + 1 + num59,
            );
          }
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      } else {
        throw new TypeError("Trying to call a non-function");
      }
    } else {
      throw new TypeError("Trying to call a non-function");
    }
  } else {
    throw new TypeError("Trying to call a non-function");
  }
}

function get() {
  let lastyear = null;
  if (this.monthinfo) {
    lastyear = this.monthinfo.lastyear;
  }
  return lastyear;
}

function get() {
  let lastmonth = null;
  if (this.monthinfo) {
    lastmonth = this.monthinfo.lastmonth;
  }
  return lastmonth;
}

function get() {
  return this.yearinfo.yearlen;
}

function get() {
  return this.yearinfo.yearordinal;
}

function get() {
  return this.yearinfo.mrange;
}

function get() {
  return this.yearinfo.wdaymask;
}

function get() {
  return this.yearinfo.mmask;
}

function get() {
  return this.yearinfo.wnomask;
}

function get() {
  if (this.monthinfo) {
    let nwdaymask = this.monthinfo.nwdaymask;
  } else {
    nwdaymask = [];
  }
  return nwdaymask;
}

function get() {
  return this.yearinfo.nextyearlen;
}

function get() {
  return this.yearinfo.mdaymask;
}

function get() {
  return this.yearinfo.nmdaymask;
}

function f136701() {
  const items = [closure_1_7(this.yearlen), 0, this.yearlen];
  return items;
}

function f136702(arg0, arg1) {
  const yearlen = this.yearlen;
  if (typeof closure_1_8 === "function") {
    const items = [];
    if (closure_1_6(null)) {
      let num3 = 0;
      if (tmp5) {
        do {
          let items1 = [];
          items[num3] = items1.concat(null);
          num3 = num3 + 1;
        } while (num3 < yearlen);
      }
    } else {
      let num2 = 0;
      if (tmp5) {
        do {
          items[num2] = null;
          num2 = num2 + 1;
        } while (num2 < yearlen);
      }
    }
    let sum = tmp;
    if (tmp < tmp2) {
      do {
        items[sum] = sum;
        sum = sum + 1;
      } while (sum < tmp2);
    }
    const items2 = [items, tmp, tmp2];
    return items2;
  } else {
    throw new TypeError("Trying to call a non-function");
  }
}

function f136703(arg0, arg1, arg2) {
  const self = this;
  const sum = this.yearlen + 7;
  if (typeof closure_1_8 === "function") {
    const items = [];
    if (closure_1_6(null)) {
      let num5 = 0;
      if (0 < sum) {
        do {
          let items1 = [];
          items[num5] = items1.concat(null);
          num5 = num5 + 1;
        } while (num5 < sum);
      }
    } else {
      let num3 = 0;
      if (0 < sum) {
        do {
          items[num3] = null;
          num3 = num3 + 1;
        } while (num3 < sum);
      }
    }
    if (typeof closure_1_15 === "function") {
      const _Date = Date;
      const _Date2 = Date;
      const _Date3 = Date;
      const date = new Date(Date.UTC(arg0, arg1 - 1, arg2, 0, 0, 0));
      if (typeof tmp4 === "function") {
        if (typeof closure_1_23 === "function") {
          const diff = tmp13 - 60 * date.getTimezoneOffset() * 1000;
          if (typeof tmp14 === "function") {
            const _Math = Math;
            const diff1 =
              Math.round((diff - (tmp16 - 60 * closure_1_18.getTimezoneOffset() * 1000)) / closure_1_17) -
              self.yearordinal;
            items[diff1] = diff1;
            const sum1 = diff1 + 1;
            let num12 = 0;
            let tmp20 = sum1;
            let tmp21 = sum1;
            if (self.wdaymask[sum1] !== self.options.wkst) {
              const sum2 = num12 + 1;
              tmp21 = tmp20;
              while (sum2 < 7) {
                items[tmp20] = tmp20;
                let sum3 = tmp20 + 1;
                num12 = sum2;
                tmp20 = sum3;
                tmp21 = sum3;
                if (self.wdaymask[sum3] === self.options.wkst) {
                  break;
                }
              }
            }
            const items2 = [items, diff1, tmp21];
            return items2;
          } else {
            throw new TypeError("Trying to call a non-function");
          }
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      } else {
        throw new TypeError("Trying to call a non-function");
      }
    } else {
      throw new TypeError("Trying to call a non-function");
    }
  } else {
    throw new TypeError("Trying to call a non-function");
  }
}

function f136704(arg0, arg1, arg2) {
  const yearlen = this.yearlen;
  if (typeof closure_1_8 === "function") {
    const items = [];
    if (closure_1_6(null)) {
      let num5 = 0;
      if (tmp4) {
        do {
          let items1 = [];
          items[num5] = items1.concat(null);
          num5 = num5 + 1;
        } while (num5 < yearlen);
      }
    } else {
      let num3 = 0;
      if (tmp4) {
        do {
          items[num3] = null;
          num3 = num3 + 1;
        } while (num3 < yearlen);
      }
    }
    if (typeof closure_1_15 === "function") {
      const _Date = Date;
      const _Date2 = Date;
      const _Date3 = Date;
      const date = new Date(Date.UTC(arg0, arg1 - 1, arg2, 0, 0, 0));
      if (typeof tmp5 === "function") {
        if (typeof closure_1_23 === "function") {
          const diff = tmp14 - 60 * date.getTimezoneOffset() * 1000;
          if (typeof tmp15 === "function") {
            const _Math = Math;
            const diff1 =
              Math.round((diff - (tmp17 - 60 * closure_1_18.getTimezoneOffset() * 1000)) / closure_1_17) -
              tmp.yearordinal;
            items[diff1] = diff1;
            const items2 = [items, diff1, diff1 + 1];
            return items2;
          } else {
            throw new TypeError("Trying to call a non-function");
          }
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      } else {
        throw new TypeError("Trying to call a non-function");
      }
    } else {
      throw new TypeError("Trying to call a non-function");
    }
  } else {
    throw new TypeError("Trying to call a non-function");
  }
}

function f136705(arg0, arg1, arg2, arg3) {
  closure_0 = arg0;
  closure_1 = arg2;
  closure_2 = arg3;
  const self = this;
  closure_4 = [];
  const byminute = this.options.byminute;
  const item = byminute.forEach((item) => {
    closure_4 = closure_4.concat(self.mtimeset(closure_0, item, closure_1, closure_2));
  });
  if (typeof closure_1_32 === "function") {
    const sorted = obj.sort((getTime, getTime2) => {
      const time = getTime.getTime();
      return time - getTime2.getTime();
    });
    return closure_4;
  } else {
    throw new TypeError("Trying to call a non-function");
  }
  obj = closure_4;
}

function f136706(arg0, arg1, arg2, arg3) {
  closure_0 = arg0;
  closure_1 = arg1;
  closure_2 = arg3;
  const bysecond = this.options.bysecond;
  const mapped = bysecond.map((item) => {
    let num = closure_2;
    if (typeof closure_2_49 === "function") {
      const obj = { hour: tmp, minute: tmp2, second: item };
      if (!num) {
        num = 0;
      }
      obj.millisecond = num;
      return obj;
    } else {
      throw new TypeError("Trying to call a non-function");
    }
  });
  if (typeof closure_32 === "function") {
    const sorted = mapped.sort((getTime, getTime2) => {
      const time = getTime.getTime();
      return time - getTime2.getTime();
    });
    return mapped;
  } else {
    throw new TypeError("Trying to call a non-function");
  }
}

function f136707(hour, minute, arg2, arg3) {
  if (typeof closure_1_49 === "function") {
    let num = arg3;
    const obj = { hour, minute, second: arg2 };
    if (!arg3) {
      num = 0;
    }
    obj.millisecond = num;
    const items = [obj];
    return items;
  } else {
    throw new TypeError("Trying to call a non-function");
  }
}

function f136708(arg0) {
  const self = this;
  if (closure_1_39.YEARLY === arg0) {
    const ydayset = self.ydayset;
    return ydayset.bind(self);
  } else if (closure_1_39.MONTHLY === arg0) {
    const mdayset = self.mdayset;
    return mdayset.bind(self);
  } else if (closure_1_39.WEEKLY === arg0) {
    const wdayset = self.wdayset;
    return wdayset.bind(self);
  } else {
    const DAILY = closure_1_39.DAILY;
    const ddayset = self.ddayset;
    return ddayset.bind(self);
  }
}

function f136709(arg0) {
  const self = this;
  if (closure_1_39.HOURLY === arg0) {
    const htimeset = self.htimeset;
    return htimeset.bind(self);
  } else if (closure_1_39.MINUTELY === arg0) {
    const mtimeset = self.mtimeset;
    return mtimeset.bind(self);
  } else if (closure_1_39.SECONDLY === arg0) {
    const stimeset = self.stimeset;
    return stimeset.bind(self);
  }
}

function t(arg0, arg1) {
  const obj = {};
  let obj2 = arg0;
  if (undefined === arg0) {
    obj2 = {};
  }
  let flag = arg1;
  if (undefined === arg1) {
    flag = false;
  }
  let tmp = null;
  if (!flag) {
    if (typeof closure_1_57 === "function") {
      const obj3 = { all: false, before: [], after: [], between: [] };
      tmp = obj3;
    } else {
      throw new TypeError("Trying to call a non-function");
    }
  }
  obj._cache = tmp;
  obj.origOptions = closure_1_51(obj2);
  const tmp5 = closure_1_38(closure_1_38({}, closure_1_70), closure_1_51(obj2));
  if (typeof closure_1_3 === "function") {
    if (null != tmp5.byeaster) {
      tmp5.freq = closure_1_72.YEARLY;
    }
    if (typeof closure_1_3 === "function") {
      if (null != tmp5.freq) {
        if (closure_1_72.FREQUENCIES[tmp5.freq]) {
          if (!tmp5.dtstart) {
            const _Date = Date;
            const _Date2 = Date;
            const date = new Date();
            const date1 = new Date(date.setMilliseconds(0));
            tmp5.dtstart = date1;
          }
          if (typeof closure_1_3 === "function") {
            if (null != tmp5.wkst) {
              if (typeof closure_1_4 === "function") {
                if (typeof tmp5.wkst !== "number") {
                  tmp5.wkst = tmp5.wkst.weekday;
                }
              } else {
                throw new TypeError("Trying to call a non-function");
              }
            } else {
              tmp5.wkst = tmp8.MO.weekday;
            }
            if (typeof closure_1_3 === "function") {
              if (null != tmp5.bysetpos) {
                if (typeof closure_1_4 === "function") {
                  if (typeof tmp5.bysetpos === "number") {
                    const items = [tmp5.bysetpos];
                    tmp5.bysetpos = items;
                  }
                  let num4 = 0;
                  if (0 < tmp5.bysetpos.length) {
                    while (0 !== tmp5.bysetpos[num4]) {
                      if (tmp23 < -366) {
                        break;
                      } else if (tmp23 > 366) {
                        break;
                      } else {
                        num4 = num4 + 1;
                      }
                    }
                    const _Error2 = Error;
                    const error = new Error("bysetpos must be between 1 and 366, or between -366 and -1");
                    throw error;
                  }
                } else {
                  throw new TypeError("Trying to call a non-function");
                }
              }
              const _Boolean = Boolean;
              if (!Boolean(tmp5.byweekno)) {
                const byweekno = tmp5.byweekno;
                if (typeof closure_1_13 === "function") {
                  if (typeof closure_1_12 === "function") {
                    if (typeof closure_1_3 === "function") {
                      let tmp30 = !tmp29;
                      if (null != byweekno) {
                        tmp30 = 0 === byweekno.length;
                      }
                      if (tmp30) {
                        const byyearday = tmp5.byyearday;
                        if (typeof closure_1_13 === "function") {
                          if (typeof closure_1_12 === "function") {
                            if (typeof closure_1_3 === "function") {
                              let tmp32 = !tmp31;
                              if (null != byyearday) {
                                tmp32 = 0 === byyearday.length;
                              }
                              if (tmp32) {
                                const _Boolean2 = Boolean;
                                if (!Boolean(tmp5.bymonthday)) {
                                  const bymonthday = tmp5.bymonthday;
                                  if (typeof closure_1_13 === "function") {
                                    if (typeof closure_1_12 === "function") {
                                      if (typeof closure_1_3 === "function") {
                                        let tmp34 = !tmp33;
                                        if (null != bymonthday) {
                                          tmp34 = 0 === bymonthday.length;
                                        }
                                        if (tmp34) {
                                          if (typeof closure_1_3 === "function") {
                                            if (null == tmp5.byweekday) {
                                              if (typeof closure_1_3 === "function") {
                                                if (null == tmp5.byeaster) {
                                                  const freq = tmp5.freq;
                                                  if (closure_1_72.YEARLY === freq) {
                                                    if (!tmp5.bymonth) {
                                                      const dtstart2 = tmp5.dtstart;
                                                      tmp5.bymonth = dtstart2.getUTCMonth() + 1;
                                                    }
                                                    const dtstart3 = tmp5.dtstart;
                                                    tmp5.bymonthday = dtstart3.getUTCDate();
                                                  } else if (closure_1_72.MONTHLY === freq) {
                                                    const dtstart = tmp5.dtstart;
                                                    tmp5.bymonthday = dtstart.getUTCDate();
                                                  } else if (closure_1_72.WEEKLY === freq) {
                                                    const dtstart7 = tmp5.dtstart;
                                                    if (typeof closure_1_27 === "function") {
                                                      const items1 = [closure_1_19[dtstart7.getUTCDay(dtstart7)]];
                                                      tmp5.byweekday = items1;
                                                    } else {
                                                      throw new TypeError("Trying to call a non-function");
                                                    }
                                                  }
                                                }
                                              } else {
                                                throw new TypeError("Trying to call a non-function");
                                              }
                                            }
                                          } else {
                                            throw new TypeError("Trying to call a non-function");
                                          }
                                        }
                                      } else {
                                        throw new TypeError("Trying to call a non-function");
                                      }
                                    } else {
                                      throw new TypeError("Trying to call a non-function");
                                    }
                                  } else {
                                    throw new TypeError("Trying to call a non-function");
                                  }
                                }
                              }
                            } else {
                              throw new TypeError("Trying to call a non-function");
                            }
                          } else {
                            throw new TypeError("Trying to call a non-function");
                          }
                        } else {
                          throw new TypeError("Trying to call a non-function");
                        }
                      }
                    } else {
                      throw new TypeError("Trying to call a non-function");
                    }
                  } else {
                    throw new TypeError("Trying to call a non-function");
                  }
                } else {
                  throw new TypeError("Trying to call a non-function");
                }
              }
              if (typeof closure_1_3 === "function") {
                let tmp37 = null != tmp5.bymonth;
                if (tmp37) {
                  tmp37 = !closure_1_6(tmp5.bymonth);
                }
                if (tmp37) {
                  const items2 = [tmp5.bymonth];
                  tmp5.bymonth = items2;
                }
                if (typeof closure_1_3 === "function") {
                  let tmp39 = null != tmp5.byyearday;
                  if (tmp39) {
                    tmp39 = !closure_1_6(tmp5.byyearday);
                  }
                  if (tmp39) {
                    if (typeof closure_1_4 === "function") {
                      tmp39 = typeof tmp5.byyearday === "number";
                    } else {
                      throw new TypeError("Trying to call a non-function");
                    }
                  }
                  if (tmp39) {
                    const items3 = [tmp5.byyearday];
                    tmp5.byyearday = items3;
                  }
                  if (typeof closure_1_3 === "function") {
                    if (null != tmp5.bymonthday) {
                      const bymonthday1 = tmp5.bymonthday;
                      if (closure_1_6(tmp5.bymonthday)) {
                        const items4 = [];
                        const items5 = [];
                        let num7 = 0;
                        if (0 < bymonthday1.length) {
                          do {
                            let tmp43 = tmp5.bymonthday[num7];
                            if (tmp43 > 0) {
                              let arr = items4.push(tmp43);
                            } else if (tmp43 < 0) {
                              let arr2 = items5.push(tmp43);
                            }
                            num7 = num7 + 1;
                          } while (num7 < tmp5.bymonthday.length);
                        }
                        tmp5.bymonthday = items4;
                        tmp5.bynmonthday = items5;
                      } else if (bymonthday1 < 0) {
                        const items6 = [tmp5.bymonthday];
                        tmp5.bynmonthday = items6;
                        tmp5.bymonthday = [];
                      } else {
                        tmp5.bynmonthday = [];
                        const items7 = [tmp5.bymonthday];
                        tmp5.bymonthday = items7;
                      }
                    } else {
                      tmp5.bymonthday = [];
                      tmp5.bynmonthday = [];
                    }
                    if (typeof closure_1_3 === "function") {
                      let tmp48 = null != tmp5.byweekno;
                      if (tmp48) {
                        tmp48 = !closure_1_6(tmp5.byweekno);
                      }
                      if (tmp48) {
                        const items8 = [tmp5.byweekno];
                        tmp5.byweekno = items8;
                      }
                      if (typeof closure_1_3 === "function") {
                        if (null != tmp5.byweekday) {
                          if (typeof closure_1_4 === "function") {
                            if (typeof tmp5.byweekday === "number") {
                              const items9 = [tmp5.byweekday];
                              tmp5.byweekday = items9;
                              tmp5.bynweekday = null;
                              let tmp50 = closure_1_3;
                            } else {
                              const byweekday = tmp5.byweekday;
                              if (typeof closure_1_5 === "function") {
                                let hasItem = typeof byweekday === "string";
                                if (typeof byweekday === "string") {
                                  hasItem = closure_1_1.includes(byweekday);
                                }
                                if (hasItem) {
                                  const items10 = [closure_1_2.fromStr(tmp5.byweekday).weekday];
                                  tmp5.byweekday = items10;
                                  tmp5.bynweekday = null;
                                  tmp50 = closure_1_3;
                                } else {
                                  const byweekday1 = tmp5.byweekday;
                                  if (tmp5.byweekday instanceof closure_1_2) {
                                    if (byweekday1.n) {
                                      if (tmp5.freq <= closure_1_72.MONTHLY) {
                                        const items11 = [tmp5.byweekday.weekday, tmp5.byweekday.n];
                                        const items12 = [items11];
                                        tmp5.bynweekday = items12;
                                        tmp5.byweekday = null;
                                        tmp50 = closure_1_3;
                                      }
                                    }
                                    const items13 = [tmp5.byweekday.weekday];
                                    tmp5.byweekday = items13;
                                    tmp5.bynweekday = null;
                                    tmp50 = closure_1_3;
                                  } else {
                                    const items14 = [];
                                    const items15 = [];
                                    let num9 = 0;
                                    if (0 < byweekday1.length) {
                                      while (typeof closure_1_4 === "function") {
                                        if (typeof tmp54 === "number") {
                                          let arr3 = items14.push(tmp54);
                                        } else if (typeof closure_1_5 === "function") {
                                          let hasItem1 = typeof tmp54 === "string";
                                          if (typeof tmp54 === "string") {
                                            hasItem1 = closure_1_1.includes(tmp54);
                                          }
                                          if (hasItem1) {
                                            let arr4 = items14.push(closure_1_2.fromStr(tmp54).weekday);
                                          } else {
                                            if (tmp54.n) {
                                              if (tmp5.freq <= closure_1_72.MONTHLY) {
                                                let items16 = [,];
                                                ({ weekday: arr16[0], n: arr16[1] } = tmp54);
                                                let arr5 = items15.push(items16);
                                              }
                                            }
                                            let arr6 = items14.push(tmp54.weekday);
                                          }
                                        } else {
                                          let str32 = "Trying to call a non-function";
                                          throw new TypeError("Trying to call a non-function");
                                        }
                                        num9 = num9 + 1;
                                      }
                                      throw new TypeError("Trying to call a non-function");
                                    }
                                    if (typeof closure_1_13 === "function") {
                                      if (typeof closure_1_12 === "function") {
                                        if (typeof closure_1_3 === "function") {
                                          let tmp67 = null;
                                          if (0 !== items14.length) {
                                            tmp67 = items14;
                                          }
                                          tmp5.byweekday = tmp67;
                                          if (typeof tmp64 === "function") {
                                            if (typeof tmp65 === "function") {
                                              if (typeof closure_1_3 === "function") {
                                                let tmp68 = null;
                                                if (0 !== items15.length) {
                                                  tmp68 = items15;
                                                }
                                                tmp5.bynweekday = tmp68;
                                                tmp50 = closure_1_3;
                                              } else {
                                                throw new TypeError("Trying to call a non-function");
                                              }
                                            } else {
                                              throw new TypeError("Trying to call a non-function");
                                            }
                                          } else {
                                            throw new TypeError("Trying to call a non-function");
                                          }
                                        } else {
                                          throw new TypeError("Trying to call a non-function");
                                        }
                                      } else {
                                        throw new TypeError("Trying to call a non-function");
                                      }
                                    } else {
                                      throw new TypeError("Trying to call a non-function");
                                    }
                                  }
                                }
                              } else {
                                throw new TypeError("Trying to call a non-function");
                              }
                            }
                          } else {
                            throw new TypeError("Trying to call a non-function");
                          }
                        } else {
                          tmp5.bynweekday = null;
                          tmp50 = closure_1_3;
                        }
                        if (typeof tmp50 === "function") {
                          if (null != tmp5.byhour) {
                            if (typeof closure_1_4 === "function") {
                              if (typeof tmp5.byhour === "number") {
                                const items17 = [tmp5.byhour];
                                tmp5.byhour = items17;
                              }
                            } else {
                              throw new TypeError("Trying to call a non-function");
                            }
                          } else {
                            let tmp72 = null;
                            if (tmp5.freq < closure_1_72.HOURLY) {
                              const dtstart4 = tmp5.dtstart;
                              const items18 = [dtstart4.getUTCHours()];
                              tmp72 = items18;
                            }
                            tmp5.byhour = tmp72;
                          }
                          if (typeof tmp50 === "function") {
                            if (null != tmp5.byminute) {
                              if (typeof closure_1_4 === "function") {
                                if (typeof tmp5.byminute === "number") {
                                  const items19 = [tmp5.byminute];
                                  tmp5.byminute = items19;
                                }
                              } else {
                                throw new TypeError("Trying to call a non-function");
                              }
                            } else {
                              let tmp75 = null;
                              if (tmp5.freq < closure_1_72.MINUTELY) {
                                const dtstart5 = tmp5.dtstart;
                                const items20 = [dtstart5.getUTCMinutes()];
                                tmp75 = items20;
                              }
                              tmp5.byminute = tmp75;
                            }
                            if (typeof tmp50 === "function") {
                              if (null != tmp5.bysecond) {
                                if (typeof closure_1_4 === "function") {
                                  if (typeof tmp5.bysecond === "number") {
                                    const items21 = [tmp5.bysecond];
                                    tmp5.bysecond = items21;
                                  }
                                } else {
                                  throw new TypeError("Trying to call a non-function");
                                }
                              } else {
                                let tmp78 = null;
                                if (tmp5.freq < closure_1_72.SECONDLY) {
                                  const dtstart6 = tmp5.dtstart;
                                  const items22 = [dtstart6.getUTCSeconds()];
                                  tmp78 = items22;
                                }
                                tmp5.bysecond = tmp78;
                              }
                              obj.options = tmp5;
                            } else {
                              throw new TypeError("Trying to call a non-function");
                            }
                          } else {
                            throw new TypeError("Trying to call a non-function");
                          }
                        } else {
                          throw new TypeError("Trying to call a non-function");
                        }
                      } else {
                        throw new TypeError("Trying to call a non-function");
                      }
                    } else {
                      throw new TypeError("Trying to call a non-function");
                    }
                  } else {
                    throw new TypeError("Trying to call a non-function");
                  }
                } else {
                  throw new TypeError("Trying to call a non-function");
                }
              } else {
                throw new TypeError("Trying to call a non-function");
              }
            } else {
              throw new TypeError("Trying to call a non-function");
            }
          } else {
            throw new TypeError("Trying to call a non-function");
          }
        }
      }
      const _Error = Error;
      const concat = "Invalid frequency: ".concat;
      const combined = "Invalid frequency: ".concat(tmp5.freq, " ");
      const error1 = new Error(combined.concat(obj2.freq));
      throw error1;
    } else {
      throw new TypeError("Trying to call a non-function");
    }
  } else {
    throw new TypeError("Trying to call a non-function");
  }
  const tmp4 = closure_1_38({}, closure_1_70);
}

function f136711(arg0, arg1) {
  return closure_1_47(arg0, arg1);
}

function f136712(arg0, arg1) {
  let tmp = arg1;
  if (undefined === arg1) {
    tmp = closure_1_41;
  }
  new closure_1_72(closure_1_47(arg0, tmp) || undefined);
  return new.target;
}

function f136713(arg0) {
  const tmp2 = closure_1_0.parseString(arg0) || undefined;
  new closure_1_0(tmp2);
  return Object.create(closure_1_0.prototype);
}

function f136714(arg0) {
  return closure_1_68(arg0, this.options);
}

function f136715(arg0, arg1) {
  let _cacheGetResult = this._cache;
  if (_cacheGetResult) {
    const _cache = tmp._cache;
    _cacheGetResult = _cache._cacheGet(arg0, arg1);
  }
  return _cacheGetResult;
}

function f136716(arg0, arg1, arg2) {
  if (this._cache) {
    const _cache = tmp._cache;
    return _cache._cacheAdd(arg0, arg1, arg2);
  }
}

function f136717(iterator) {
  const self = this;
  if (iterator) {
    if (typeof closure_1_40 === "function") {
      const obj = {};
      const call = closure_168_0.call;
      if (typeof call === "unknown") {
        closure_168_0("all", obj);
      } else {
        let callResult = call(tmp10, "all", obj);
      }
      if (!callResult) {
        callResult = tmp10;
      }
      callResult.iterator = iterator;
      return tmp8(callResult);
    } else {
      throw new TypeError("Trying to call a non-function");
    }
  } else {
    let _cacheGetResult = self._cacheGet("all");
    if (false === _cacheGetResult) {
      new closure_1_36("all", {});
      const _iterResult = self._iter(new.target);
      self._cacheAdd("all", _iterResult);
      _cacheGetResult = _iterResult;
    }
    return _cacheGetResult;
  }
}

function f136718(getTime, getTime2, arg2, iterator) {
  let flag = arg2;
  if (undefined === arg2) {
    flag = false;
  }
  if (typeof closure_1_22 === "function") {
    if (typeof closure_1_21 === "function") {
      const _Date = Date;
      let tmp5 = getTime instanceof Date;
      if (tmp5) {
        const _isNaN = isNaN;
        tmp5 = !isNaN(getTime.getTime());
      }
      if (tmp5) {
        if (typeof tmp === "function") {
          if (typeof tmp2 === "function") {
            const _Date2 = Date;
            let tmp7 = getTime2 instanceof Date;
            if (tmp7) {
              const _isNaN2 = isNaN;
              tmp7 = !isNaN(getTime2.getTime());
            }
            if (tmp7) {
              const self = this;
              const obj = { before: getTime2, after: getTime, inc: flag };
              if (iterator) {
                if (typeof closure_1_40 === "function") {
                  const call = closure_168_0.call;
                  if (typeof call === "unknown") {
                    closure_168_0("between", obj);
                  } else {
                    let callResult = call(tmp23, "between", obj);
                  }
                  if (!callResult) {
                    callResult = tmp23;
                  }
                  callResult.iterator = iterator;
                  return tmp21(callResult);
                } else {
                  throw new TypeError("Trying to call a non-function");
                }
              } else {
                let _cacheGetResult = self._cacheGet("between", obj);
                if (false === _cacheGetResult) {
                  new closure_1_36("between", obj);
                  const _iterResult = self._iter(new.target);
                  self._cacheAdd("between", _iterResult, obj);
                  _cacheGetResult = _iterResult;
                }
                return _cacheGetResult;
              }
            }
          } else {
            throw new TypeError("Trying to call a non-function");
          }
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      }
      const _Error = Error;
      const error = new Error("Invalid date passed in to RRule.between");
      throw error;
    } else {
      throw new TypeError("Trying to call a non-function");
    }
  } else {
    throw new TypeError("Trying to call a non-function");
  }
}

function f136719(getTime, arg1) {
  let flag = arg1;
  if (undefined === arg1) {
    flag = false;
  }
  if (typeof closure_1_22 === "function") {
    if (typeof closure_1_21 === "function") {
      const _Date = Date;
      let tmp4 = getTime instanceof Date;
      if (tmp4) {
        const _isNaN = isNaN;
        tmp4 = !isNaN(getTime.getTime());
      }
      if (tmp4) {
        const self = this;
        const obj = { dt: getTime, inc: flag };
        let _cacheGetResult = this._cacheGet("before", obj);
        if (false === _cacheGetResult) {
          new closure_1_36("before", obj);
          const _iterResult = self._iter(new.target);
          self._cacheAdd("before", _iterResult, obj);
          _cacheGetResult = _iterResult;
        }
        return _cacheGetResult;
      } else {
        const _Error = Error;
        const error = new Error("Invalid date passed in to RRule.before");
        throw error;
      }
    } else {
      throw new TypeError("Trying to call a non-function");
    }
  } else {
    throw new TypeError("Trying to call a non-function");
  }
}

function f136720(getTime, arg1) {
  let flag = arg1;
  if (undefined === arg1) {
    flag = false;
  }
  if (typeof closure_1_22 === "function") {
    if (typeof closure_1_21 === "function") {
      const _Date = Date;
      let tmp4 = getTime instanceof Date;
      if (tmp4) {
        const _isNaN = isNaN;
        tmp4 = !isNaN(getTime.getTime());
      }
      if (tmp4) {
        const self = this;
        const obj = { dt: getTime, inc: flag };
        let _cacheGetResult = this._cacheGet("after", obj);
        if (false === _cacheGetResult) {
          new closure_1_36("after", obj);
          const _iterResult = self._iter(new.target);
          self._cacheAdd("after", _iterResult, obj);
          _cacheGetResult = _iterResult;
        }
        return _cacheGetResult;
      } else {
        const _Error = Error;
        const error = new Error("Invalid date passed in to RRule.after");
        throw error;
      }
    } else {
      throw new TypeError("Trying to call a non-function");
    }
  } else {
    throw new TypeError("Trying to call a non-function");
  }
}

function f136721() {
  return this.all().length;
}

function f136722() {
  return closure_1_56(this.origOptions);
}

function f136723(arg0, arg1, arg2) {
  new closure_1_45(this, arg0, arg1, arg2);
  return new.target.toString();
}

function f136724() {
  return closure_1_48(this);
}

function f136725() {
  new closure_1_0(this.origOptions);
  return Object.create(closure_1_0.prototype);
}

function e(arg0) {
  let flag = arg0;
  if (undefined === arg0) {
    flag = false;
  }
  const self = this;
  const call = closure_1_0.call;
  const obj = {};
  if (typeof call === "unknown") {
    closure_1_0(obj, flag);
  } else {
    let callResult = call(self, obj, flag);
  }
  if (!callResult) {
    callResult = self;
  }
  callResult.dtstart = closure_2_75.apply(callResult, ["dtstart"]);
  callResult.tzid = closure_2_75.apply(callResult, ["tzid"]);
  callResult._rrule = [];
  callResult._rdate = [];
  callResult._exrule = [];
  callResult._exdate = [];
  return callResult;
}

function f136727(accept) {
  ({ _rrule, _exrule, _rdate, _exdate } = this);
  let after = accept;
  let before = _exrule;
  accept = undefined;
  const dependencyMap = {};
  accept = accept.accept;
  let item = _exdate.forEach((getTime) => {
    if (typeof closure_2_55 === "function") {
      const obj = {};
      const _isNaN = isNaN;
      if (isNaN(getTime.getTime())) {
        const _RangeError = RangeError;
        const rangeError = new RangeError("Invalid date passed to DateWithZone");
        throw rangeError;
      } else {
        obj.date = getTime;
        obj.tzid = tmp;
        const _Number = Number;
        closure_3[Number(obj.rezonedDate())] = true;
      }
    } else {
      throw new TypeError("Trying to call a non-function");
    }
  });
  accept.accept = function (comma) {
    const self = this;
    const NumberResult = Number(comma);
    if (isNaN(NumberResult)) {
      const call2 = accept.call;
      typeof call2 === "unknown" ? accept(comma) : call2(self, comma);
    } else {
      let tmp4 = !tmp3;
      if (!dependencyMap[NumberResult]) {
        const _Date = Date;
        const _Date2 = Date;
        const date1 = new Date(NumberResult + 1);
        let item = date1.forEach((between) => {
          const item = between.between(date, date1, true).forEach((item) => {
            closure_1_3[Number(item)] = true;
          });
        });
        tmp4 = !dependencyMap[NumberResult];
        const date = new Date(NumberResult - 1);
      }
      if (!tmp4) {
        return !tmp4;
      } else {
        dependencyMap[NumberResult] = true;
        const call = accept.call;
        typeof call === "unknown" ? accept(comma) : call(self, comma);
      }
    }
  };
  if ("between" === accept.method) {
    after = accept.args.after;
    before = accept.args.before;
    const item1 = _exrule.forEach((between) => {
      const item = between.between(date, date1, true).forEach((item) => {
        closure_1_3[Number(item)] = true;
      });
    });
    accept.accept = function (comma) {
      const NumberResult = Number(comma);
      if (dependencyMap[NumberResult]) {
        return tmp3;
      } else {
        const self = this;
        tmp2[NumberResult] = true;
        const call = accept.call;
        typeof call === "unknown" ? accept(comma) : call(self, comma);
      }
    };
  }
  let num = 0;
  if (0 < _rdate.length) {
    while (typeof closure_55 === "function") {
      let obj2 = {};
      let _isNaN = isNaN;
      if (isNaN(obj.getTime())) {
        let _RangeError = RangeError;
        let tmp16 = new.target;
        let str3 = "Invalid date passed to DateWithZone";
        let tmp17 = new.target;
        let rangeError = new RangeError("Invalid date passed to DateWithZone");
        throw rangeError;
      } else {
        obj2.date = obj;
        obj2.tzid = tzidResult;
        let rezonedDateResult = obj2.rezonedDate();
        let _Date = Date;
        let tmp7 = new.target;
        let tmp8 = new.target;
        let date = new Date(rezonedDateResult.getTime());
        if (accept.accept(date)) {
          num = num + 1;
        }
      }
    }
    throw new TypeError("Trying to call a non-function");
  }
  const item2 = _rrule.forEach((options) => {
    closure_2_68(after, options.options);
  });
  const _result = accept._result;
  if (typeof closure_32 === "function") {
    const sorted = _result.sort((getTime, getTime2) => {
      const time = getTime.getTime();
      return time - getTime2.getTime();
    });
    const method = accept.method;
    let tmp13 = _result;
    if ("all" !== method) {
      tmp13 = _result;
      if ("between" !== method) {
        if ("before" === method) {
          tmp13 = (_result.length && _result[_result.length - 1]) || null;
          const tmp15 = (_result.length && _result[_result.length - 1]) || null;
        } else {
          tmp13 = (_result.length && _result[0]) || null;
          const tmp14 = (_result.length && _result[0]) || null;
        }
      }
    }
    return tmp13;
  } else {
    throw new TypeError("Trying to call a non-function");
  }
  tzidResult = this.tzid();
}

function f136728(arg0) {
  closure_1_77(arg0, this._rrule);
}

function f136729(arg0) {
  closure_1_77(arg0, this._exrule);
}

function f136730(arg0) {
  closure_1_78(arg0, this._rdate);
}

function f136731(arg0) {
  closure_1_78(arg0, this._exdate);
}

function f136732() {
  const _rrule = this._rrule;
  return _rrule.map((item) => closure_1_74(item.toString()));
}

function f136733() {
  const _exrule = this._exrule;
  return _exrule.map((item) => closure_1_74(item.toString()));
}

function f136734() {
  const _rdate = this._rdate;
  return _rdate.map((getTime) => new Date(getTime.getTime()));
}

function f136735() {
  const _exdate = this._exdate;
  return _exdate.map((getTime) => new Date(getTime.getTime()));
}

function f136736() {
  const self = this;
  closure_0 = [];
  let _dtstart = !length;
  if (!this._rrule.length) {
    _dtstart = self._dtstart;
  }
  if (_dtstart) {
    const obj = { dtstart: self._dtstart };
    closure_0 = closure_0.concat(closure_56(obj));
  }
  const _rrule = self._rrule;
  const item = _rrule.forEach((item) => {
    closure_0 = closure_0.concat(item.toString().split("\n"));
  });
  const _exrule = self._exrule;
  const item1 = _exrule.forEach((item) => {
    const parts = item.toString().split("\n");
    const mapped = parts.map((item) => item.replace(/^RRULE:/, "EXRULE:"));
    closure_0 = closure_0.concat(mapped.filter((item) => !/^DTSTART/.test(item)));
  });
  if (!self._rdate.length) {
    if (!self._exdate.length) {
      return closure_0;
    } else {
      let _exdate = self._exdate;
      const str8 = self.tzid();
      closure_129_0 = undefined;
      let tmp10 = !str8;
      if (str8) {
        tmp10 = "UTC" === str8.toUpperCase();
      }
      closure_129_0 = tmp10;
      const concat3 = "".concat;
      let str12 = ":";
      if (tmp10) {
        let concat3Result = concat3("EXDATE", `:`);
      } else {
        concat3Result = concat3("EXDATE", ";TZID=").concat(str8, `:`);
        const concat3Result1 = concat3("EXDATE", ";TZID=");
      }
      _exdate = _exdate.map((item) => {
        const valueOfResult = item.valueOf();
        let flag = closure_0;
        if (typeof closure_2_33 === "function") {
          if (undefined === flag) {
            flag = true;
          }
          const _Date = Date;
          const date = new Date(valueOfResult);
          const items = [
            closure_2_9(date.getUTCFullYear().toString(), 4, "0"),
            closure_2_9(date.getUTCMonth() + 1, 2, "0"),
            closure_2_9(date.getUTCDate(), 2, "0"),
            "T",
            closure_2_9(date.getUTCHours(), 2, "0"),
            closure_2_9(date.getUTCMinutes(), 2, "0"),
            closure_2_9(date.getUTCSeconds(), 2, "0"),
          ];
          let str5 = "";
          if (flag) {
            str5 = "Z";
          }
          items[7] = str5;
          return items.join("");
        } else {
          throw new TypeError("Trying to call a non-function");
        }
      });
      str12 = _exdate.join(",");
      const concat4 = "".concat;
      const combined = "".concat(concat3Result);
      closure_0.push(combined.concat(str12));
    }
  } else {
    let _rdate = self._rdate;
    const str = self.tzid();
    closure_0 = undefined;
    let tmp6 = !str;
    if (str) {
      tmp6 = "UTC" === str.toUpperCase();
    }
    closure_0 = tmp6;
    const concat = "".concat;
    let str5 = ":";
    if (tmp6) {
      let combined1 = concat("RDATE", `:`);
    } else {
      const combined2 = concat("RDATE", ";TZID=");
      combined1 = combined2.concat(str, `:`);
    }
    _rdate = _rdate.map((item) => {
      const valueOfResult = item.valueOf();
      let flag = closure_0;
      if (typeof closure_2_33 === "function") {
        if (undefined === flag) {
          flag = true;
        }
        const _Date = Date;
        const date = new Date(valueOfResult);
        const items = [
          closure_2_9(date.getUTCFullYear().toString(), 4, "0"),
          closure_2_9(date.getUTCMonth() + 1, 2, "0"),
          closure_2_9(date.getUTCDate(), 2, "0"),
          "T",
          closure_2_9(date.getUTCHours(), 2, "0"),
          closure_2_9(date.getUTCMinutes(), 2, "0"),
          closure_2_9(date.getUTCSeconds(), 2, "0"),
        ];
        let str5 = "";
        if (flag) {
          str5 = "Z";
        }
        items[7] = str5;
        return items.join("");
      } else {
        throw new TypeError("Trying to call a non-function");
      }
    });
    str5 = _rdate.join(",");
    const concat2 = "".concat;
    const combined3 = "".concat(combined1);
    closure_0.push(combined3.concat(str5));
  }
}

function f136737() {
  return this.valueOf().join("\n");
}

function f136738() {
  const self = this;
  const _cache = this._cache;
  const obj2 = Object.create(closure_1_1.prototype);
  const call = closure_1_0.call;
  const obj = {};
  if (typeof call === "unknown") {
    closure_1_0(obj, _cache);
  } else {
    let callResult = call(obj2, obj, _cache);
  }
  if (!callResult) {
    callResult = obj2;
  }
  callResult.dtstart = closure_2_75.apply(callResult, ["dtstart"]);
  callResult.tzid = closure_2_75.apply(callResult, ["tzid"]);
  callResult._rrule = [];
  callResult._rdate = [];
  callResult._exrule = [];
  callResult._exdate = [];
  const _rrule = self._rrule;
  const item = _rrule.forEach((clone) => callResult.rrule(clone.clone()));
  const _exrule = self._exrule;
  const item1 = _exrule.forEach((clone) => callResult.exrule(clone.clone()));
  const _rdate = self._rdate;
  const item2 = _rdate.forEach((getTime) => callResult.rdate(new Date(getTime.getTime())));
  const _exdate = self._exdate;
  const item3 = _exdate.forEach((getTime) => callResult.exdate(new Date(getTime.getTime())));
  return callResult;
}

function f138404() {}

function f138405(gameId) {
  return gameId.gameId;
}

function f138406(tags) {
  tags = tags.tags;
  if (tags == null) {
    tags = [];
  }
  return tags;
}

function f138407(arg0) {
  return arg0.toString();
}

function f138408(arg0, comment) {
  comment = comment.comment;
  let num;
  if (comment != null) {
    num = comment.length;
  }
  if (num == null) {
    num = 0;
  }
  return arg0 + num;
}

function f138771(channelId) {
  let result = closure_3_22.canWithPartialContext(closure_3_31.CONNECT, { channelId: channelId.channel_id });
  if (result) {
    const channel_id = channelId.channel_id;
    let flag = false;
    if (null != channel_id) {
      const voiceStatesForChannel = closure_3_27.getVoiceStatesForChannel(channel_id);
      let someResult = null != voiceStatesForChannel;
      if (someResult) {
        const mapped = closure_3_1(closure_3_2[31])(voiceStatesForChannel).map((userId) => {
          user = user.getUser(userId.userId);
          let id;
          if (user != null) {
            id = user.id;
          }
          return id;
        });
        const found = mapped.filter(closure_3_0(closure_3_2[39]).isNotNullish);
        someResult = found.some((item) => blockedOrIgnored.isBlockedOrIgnored(item));
        const arr = closure_3_1(closure_3_2[31])(voiceStatesForChannel);
      }
      flag = someResult;
    }
    result = !flag;
  }
  if (result) {
    const obj2 = { kind: "live-guild-stage", stage: channelId };
    closure_1_2[channelId.channel_id] = obj2;
  }
  const obj = { channelId: channelId.channel_id };
}

function f138772(id) {
  const user = closure_3_26.getUser(id);
  if (null != user) {
    if (user.bot) {
      closure_1_12.add(id);
    } else {
      const primaryActivity = closure_3_23.getPrimaryActivity(id, closure_2_1);
      if (null != closure_2_1) {
        let voiceStateForUser = closure_1_11[id];
      } else {
        voiceStateForUser = closure_3_27.getVoiceStateForUser(id);
      }
      let tmp8 = closure_2_10;
      if (!closure_2_10) {
        tmp8 = null != primaryActivity;
      }
      if (!tmp8) {
        tmp8 = null != voiceStateForUser;
      }
      if (tmp8) {
        closure_1_10(id, primaryActivity, voiceStateForUser);
      }
    }
  }
}

function f138773(userId) {
  userId = userId.userId;
  closure_1_10(userId, closure_3_23.getPrimaryActivity(userId, closure_2_1), userId);
}

function f138774(event) {
  return closure_1_13.push({ kind: "guild-event", event, isLive: true });
}

function f138775(event) {
  return closure_1_13.push({ kind: "guild-event", event, isLive: false });
}

function f139445(arg0) {
  return closure_2_0.deleteMessage(closure_1_1, closure_1_0, arg0);
}

function f140398(arg0) {
  return closure_1_0[arg0];
}
