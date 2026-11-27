// discord_app/modules/app_database/system/AppDatabaseManager.tsx
import LoggerDefault from "../../debug/Logger.tsx";
import Dispatcher from "../../../Dispatcher.tsx";
import DatabaseManagerDefault from "DatabaseManager.tsx";
import actions2 from "AppDatabaseManager.Entry.tsx";
import _slicedToArray from "../../../../_runtime/metro/00032__.js";
import AuthenticationStore from "../../../stores/AuthenticationStore.tsx";

const DispatcherDefault = Dispatcher;

require = fn;
const hasOwnProperty = new LoggerDefault("AppDatabaseManager");
const set = new Set(["MESSAGE_CREATE"]);
let AppDatabaseManager;
class AppDatabaseManager {
  constructor(arg0, arg1, arg2) {
    obj = Object.create(new.target.prototype);
    closure_0 = obj;
    tmp2 = closure_3(AppDatabaseManager.computeEntries(global, importDefault), 2);
    obj.name = global;
    [tmp.actions, tmp.entries] = tmp2;
    obj.lastDatabase = null;
    registerResult = AppDatabaseManager.register(`${global}_CLEAR_CACHES`, ["CLEAR_CACHES"], [], () => {
      for (const item10006 of tmp) {
        let resetResult = item10006.reset();
        continue;
      }
    });
    actions = obj.actions;
    items = [...actions.keys()];
    handleAction = obj.handleAction;
    registerResult1 = AppDatabaseManager.register(global, items, fn, handleAction.bind(obj));
    verboseResult = closure_5.verbose(
      "" + global + " created with " + importDefault.length + " modules, " + obj.actions.size + " distinct actions.",
    );
    return obj;
  }
}
const prototype = AppDatabaseManager.prototype;
prototype["handleAction"] = function handleAction(type) {
  this.validateInDev(type.type);
  const id = AuthenticationStore.getId();
  const databaseResult = DatabaseManagerDefault.database(id);
  this.resetModules(type, databaseResult);
  this.executeModules(type, databaseResult);
  return false;
};
prototype["resetModules"] = function resetModules(type, databaseResult) {
  const self = this;
  if (databaseResult !== this.lastDatabase) {
    const _HermesInternal = HermesInternal;
    closure_5.verbose(
      "database has changed (was: " +
        self.lastDatabase +
        ", now: " +
        databaseResult +
        ", action: " +
        type.type +
        "). resetting modules.",
    );
    const entries = self.entries;
    for (const item10004 of entries) {
      let resetResult = item10004.reset();
      continue;
    }
    self.lastDatabase = databaseResult;
  }
};
prototype["executeModules"] = function executeModules(type, databaseResult) {
  _require = type;
  closure_1 = databaseResult;
  type = type.type;
  const actions = this.actions;
  value = actions.get(type);
  dependencyMap = value;
  if (databaseResult != null) {
    const stateResult = databaseResult.state();
  }
  if (null != value) {
    if (0 !== value.length) {
      if (null != databaseResult) {
        if (
          stateResult === require("../../../../discord_common/js/packages/kv-storage/js/index.tsx").DatabaseState.Open
        ) {
          let combined = null;
          if (!set.has(type.type)) {
            const _HermesInternal2 = HermesInternal;
            combined = "Dispatch " + type.type;
          }
          databaseResult.transaction((arg0) => {
            closure_0 = arg0;
            return value.forEach((execute) => execute.execute(closure_0, closure_0));
          }, combined);
          if ("WRITE_CACHES" === type.type) {
            const promisesToWaitOn = type.promisesToWaitOn;
            promisesToWaitOn.push(tmp14);
          }
        }
      }
      const _HermesInternal = HermesInternal;
      closure_5.verbose(
        "no usable database; skipping action (type: " +
          type +
          ", database: " +
          databaseResult +
          ", state: " +
          stateResult +
          ")",
      );
    }
  }
};
AppDatabaseManager["handleException"] = function handleException(arg0, type, error) {
  closure_5.info("disabling database \u00B7 error encountered during dispatch", error, error.stack);
  const obj2 = { type: "RESET_SOCKET", args: null };
  const obj = DispatcherDefault;
  obj2.args = { error, action: "AppDatabaseManager(" + type.type + ")" };
  obj.dispatch(obj2);
};
AppDatabaseManager["computeEntries"] = function computeEntries(MobileAppDatabaseManager, arr) {
  closure_0 = MobileAppDatabaseManager;
  const map = new Map();
  const mapped = arr.map((item) => {
    const entry = new actions2.Entry(closure_0, item);
    return entry;
  });
  const result = map.set("LOGOUT", []);
  const result1 = map.set("LOGIN_RESET", []);
  for (const item10025 of mapped) {
    let actions = item10025.actions;
    for (const item10032 of actions) {
      if (!map.has(item10032)) {
        let result2 = map.set(item10032, []);
      }
      value = map.get(item10032);
      arr = value.push(tmp4);
      continue;
    }
    continue;
  }
  items = [map, mapped];
  return items;
};
AppDatabaseManager["register"] = function register(arg0, arr, arg2, arg3) {
  closure_0 = arg3;
  const obj = DispatcherDefault;
  const registerResult = obj.register(
    arg0,
    Object.fromEntries(
      arr.map((item) => {
        items = [item, closure_0];
        return items;
      }),
    ),
    () => {},
    Dispatcher.DispatchBand.Database,
  );
  const fromEntriesResult = Object.fromEntries(
    arr.map((item) => {
      items = [item, closure_0];
      return items;
    }),
  );
  DispatcherDefault.addDependencies(registerResult, arg2);
  return registerResult;
};
prototype["validateInDev"] = function validateInDev() {};
const size = fn(2);
let result = size.fileFinishedImporting("modules/app_database/system/AppDatabaseManager.tsx");

export { AppDatabaseManager };
