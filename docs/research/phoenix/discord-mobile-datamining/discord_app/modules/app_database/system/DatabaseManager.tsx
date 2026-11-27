// discord_app/modules/app_database/system/DatabaseManager.tsx
import LoggerDefault from "../../debug/Logger.tsx";
import initializeDefault from "../../../../discord_common/js/packages/flux/index.tsx";
import Dispatcher2 from "../../../Dispatcher.tsx";
import _mod4792 from "../../../../discord_common/js/packages/kv-storage/js/index.tsx";
import StartupDataAll from "StartupData.native.tsx";
import asyncGeneratorStep from "../../../../_runtime/00005_asyncGeneratorStep.js";
import AuthenticationStore from "../../../stores/AuthenticationStore.tsx";

const Dispatcher = Dispatcher2;

require = fn;
function databaseName(arg0) {
  return "@account." + arg0;
}
let closure_8 = async function _trySpeculativelyOpenDatabaseAsync(arg0) {
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
          const tmp25 = databaseName(closure_0);
          const _HermesInternal = HermesInternal;
          timestampProducer.verbose("speculatively opening " + tmp25);
          c4 = 1;
          const Database = require("../../../../discord_common/js/packages/kv-storage/js/index.tsx").Database;
          c5 = 2;
          c6 = 1;
          const obj4 = { value: Database.open(tmp25), done: false };
          return obj4;
        }
      } else if (1 === tmp7) {
        c4 = 0;
        closure_129_0 = closure_3;
        closure_130_6.warn("couldn't speculatively open database.", closure_129_0);
        c6 = 3;
        return { value: null, done: true };
      } else if (arg0 === 1) {
        c6 = 3;
        throw value;
      } else if (arg0 === 2) {
        c4 = 0;
        c6 = 3;
        const obj5 = { value, done: true };
        return obj5;
      } else {
        c4 = 0;
        c6 = 3;
        const obj = { value, done: true };
        return obj;
      }
    } catch (tmp15) {
      closure_3 = tmp15;
      if (tmp4 === c4) {
        c6 = tmp2;
        throw tmp15;
      } else {
        c5 = tmp;
      }
    }
  }
};
let closure_6 = new LoggerDefault("DatabaseManager");
const Store = initializeDefault.Store;
class DatabaseManager extends Store {
  constructor() {
    closure_0 = undefined;
    tmp2 = closure_1(closure_3[4]);
    obj = {
      CLEAR_CACHES(arg0) {
        return closure_0.handleClearCaches(arg0);
      },
      CONNECTION_CLOSED() {
        return closure_0.handleAuthenticationStoreChanged();
      },
      CONNECTION_OPEN() {
        return closure_0.handleConnectionOpen();
      },
      LOGOUT: null,
    };
    class LOGOUT {
      constructor() {
        return closure_0.handleAuthenticationStoreChanged();
      }
    }
    obj.LOGOUT = LOGOUT;
    tmp1 = new tmp(tmp2, obj, closure_0(closure_3[4]).DispatchBand.Early, LOGOUT, new.target, tmp, tmp2);
    closure_0 = tmp1;
    map = new Map();
    tmp1.databases = map;
    tmp1.activeUserId = null;
    tmp1.preventWritingCachesAgainThisSession = false;
    return tmp1;
  }
}
const prototype = DatabaseManager.prototype;
prototype["initialize"] = function initialize() {
  const self = this;
  this.waitFor(AuthenticationStore);
  const result = this.carefullySpeculativelyOpen(StartupDataAll.getUserId());
  const result1 = this.handleAuthenticationStoreChanged();
  AuthenticationStore.addChangeListener(() => self.handleAuthenticationStoreChanged());
};
prototype["databaseName"] = function databaseName(arg0) {
  return "@account." + arg0;
};
prototype["database"] = function database(arg0) {
  let tmp = null;
  if (null != arg0) {
    const self = this;
    const databases = this.databases;
    value = databases.get(arg0);
    if (value == null) {
      value = null;
    }
    tmp = value;
  }
  return tmp;
};
prototype["carefullyOpenDatabase"] = function carefullyOpenDatabase(id) {
  const self = this;
  if (this.preventWritingCachesAgainThisSession) {
    logger.verbose("Not opening database because caches have been manually cleared.");
    return null;
  } else {
    if (null != id) {
      const databases2 = self.databases;
      if (!databases2.has(id)) {
        const _HermesInternal = HermesInternal;
        const combined = "@account." + id;
        const _HermesInternal2 = HermesInternal;
        logger.verbose("synchronously opening " + combined);
        const tmp6 = (function tryUntil(arg0, fn) {
          if (0 >= 50) {
            return null;
          } else {
            try {
              return fn();
            } catch (tmp7) {
              logger.error(tmp2 + tmp5, tmp7);
              const num = tmp5 + tmp;
            }
          }
        })(50, () => {
          const Database = _mod4792.Database;
          return Database.openSyncUnsafe(combined, { invalidateDisabledHandles: true });
        });
        const _HermesInternal3 = HermesInternal;
        logger.verbose("added database (" + id + " \u2192 " + tmp6 + ")");
        const databases = self.databases;
        const result = databases.set(id, tmp6);
        self.emitChange();
      }
    }
    return self.database(id);
  }
};
prototype["replaceDisableAllDatabases"] = function replaceDisableAllDatabases(arg0) {
  const self = this;
  closure_6.info("disabling and nulling all databases (reason: " + arg0 + ")");
  const databases = this.databases;
  const keys = databases.keys();
  const iter = keys[Symbol.iterator]();
  const nextResult = iter.next();
  while (iter !== undefined) {
    let databases2 = self.databases;
    let tmp4 = nextResult;
    value = databases2.get(nextResult);
    let obj2 = value;
    if (value != null) {
      let disableResult = value.disable(arg0);
    }
    if (obj2 != null) {
      let closeResult = obj2.close();
    }
    let databases3 = self.databases;
    let result = databases3.set(tmp4, null);
    continue;
  }
  self.emitChange();
};
prototype["remove"] = function remove(arg0) {
  const self = this;
  const databases = this.databases;
  value = databases.get(arg0);
  closure_6.log("removing database (user: " + arg0 + ", database: " + value + ")");
  if (value != null) {
    value.close();
  }
  const databases2 = self.databases;
  databases2.delete(arg0);
  self.emitChange();
};
prototype["handleClearCaches"] = function handleClearCaches(preventWritingCachesAgainThisSession) {
  const self = this;
  if (preventWritingCachesAgainThisSession.preventWritingCachesAgainThisSession) {
    self.preventWritingCachesAgainThisSession = true;
  }
  const result = self.replaceDisableAllDatabases(
    "DatabaseManager (" + preventWritingCachesAgainThisSession.reason + ")",
  );
};
prototype["handleConnectionOpen"] = function handleConnectionOpen() {
  const self = this;
  const id = AuthenticationStore.getId();
  const databases = this.databases;
  value = databases.get(id);
  if (value != null) {
    const stateResult = value.state();
  }
  let tmp3 = null == value;
  if (tmp3) {
    tmp3 = stateResult !== _mod4792.DatabaseState.Open;
  }
  if (tmp3) {
    self.remove(id);
  }
  const result = self.carefullyOpenDatabase(id);
};
prototype["handleAuthenticationStoreChanged"] = function handleAuthenticationStoreChanged() {
  const self = this;
  const id = AuthenticationStore.getId();
  const activeUserId = this.activeUserId;
  if (id !== activeUserId) {
    const databases2 = self.databases;
    value = databases2.get(activeUserId);
    const _HermesInternal = HermesInternal;
    closure_6.verbose("active user changed (now: " + id + ", was: " + activeUserId + ", was: " + value + ")");
    if (value != null) {
      value.close();
    }
    StartupDataAll.setUserId(id);
    self.activeUserId = id;
    const databases = self.databases;
    databases.delete(activeUserId);
  }
};
function carefullySpeculativelyOpen(userId) {
  closure_0 = userId;
  let self = this;
  return (async () => {
    if (c3 === 2) {
      c3 = 3;
      throw new TypeError("Generator functions may not be called on executing generators");
    } else if (tmp5 === 3) {
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
        c3 = 2;
        let verboseResult = 0;
        if (0 === c2) {
          if (arg0 === 1) {
            c3 = 3;
            throw value;
          } else if (arg0 === 2) {
            c3 = 3;
            const obj3 = { value, done: true };
            return obj3;
          } else {
            closure_1 = tmp2;
            closure_0 = tmp3;
            closure_128_0 = undefined;
            if (self.preventWritingCachesAgainThisSession) {
              verboseResult = closure_1_6.verbose("Not opening database because caches have been manually cleared.");
            } else if (null != closure_0) {
              c2 = 1;
              c3 = 1;
              const obj4 = {
                value: (function trySpeculativelyOpenDatabaseAsync() {
                  self = this;
                  const apply = closure_1_8.apply;
                  if (typeof apply === "unknown") {
                    let applyArgumentsResult = HermesBuiltin.applyArguments(self);
                  } else {
                    applyArgumentsResult = apply(self, arguments);
                  }
                  return applyArgumentsResult;
                })(closure_0),
                done: false,
              };
              return obj4;
            }
            c3 = 3;
          }
        } else if (arg0 === 1) {
          c3 = 3;
          throw value;
        } else if (arg0 === 2) {
          c3 = 3;
          const obj5 = { value, done: true };
          return obj5;
        } else {
          closure_128_0 = value;
          verboseResult = null;
          if (null != closure_128_0) {
            const databases2 = closure_129_1.databases;
            if (!databases2.has(closure_129_0)) {
              const _HermesInternal = HermesInternal;
              closure_1_6.verbose("added speculative database (" + closure_129_0 + " \u2192 " + closure_128_0 + ")");
              const databases = closure_129_1.databases;
              const result = databases.set(closure_129_0, closure_128_0);
              closure_129_1.emitChange();
            }
          }
        }
        const _HermesInternal2 = HermesInternal;
        closure_1_6.verbose("discarding speculative database (" + closure_129_0 + " \u2192 " + closure_128_0 + ")");
        if (closure_128_0 != verboseResult) {
          verboseResult = obj.close();
        }
        obj = closure_128_0;
      } catch (tmp32) {
        c3 = tmp;
        throw tmp32;
      }
    }
  })();
}
prototype["carefullySpeculativelyOpen"] = carefullySpeculativelyOpen;
let obj = {
  CLEAR_CACHES(arg0) {
    return closure_0.handleClearCaches(arg0);
  },
  CONNECTION_CLOSED() {
    return closure_0.handleAuthenticationStoreChanged();
  },
  CONNECTION_OPEN() {
    return closure_0.handleConnectionOpen();
  },
  LOGOUT() {
    return closure_0.handleAuthenticationStoreChanged();
  },
};
let tmp2 = new tmp(
  Dispatcher,
  obj,
  fn(572).DispatchBand.Early,
  DatabaseManager,
  tmp,
  Dispatcher,
  obj,
  new.target,
  undefined,
  carefullySpeculativelyOpen,
  globalThis,
  fn,
);
let closure_129_0 = tmp2;
let tmp3 = new LoggerDefault("DatabaseManager");
tmp2.databases = new Map();
tmp2.activeUserId = null;
tmp2.preventWritingCachesAgainThisSession = false;
const size = fn(2);
let result = size.fileFinishedImporting("modules/app_database/system/DatabaseManager.tsx");

export default tmp2;
