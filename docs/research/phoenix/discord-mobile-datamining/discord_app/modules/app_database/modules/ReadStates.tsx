// discord_app/modules/app_database/modules/ReadStates.tsx
import LoggerDefault from "../../debug/Logger.tsx";
import SnowflakeUtilsDefault from "../../../utils/SnowflakeUtils.tsx";
import applyDefault from "../../../../_runtime/00012_apply.js";
import DatabaseDaosDefault from "../DatabaseDaos.tsx";
import asyncGeneratorStep from "../../../../_runtime/00005_asyncGeneratorStep.js";
import ChannelStore from "../../../stores/ChannelStore.tsx";
import ReadStateStore from "../../../stores/ReadStateStore.tsx";

const ReadStateTypes = fn(6553).ReadStateTypes;
const logger = new LoggerDefault("ReadStates");
class ReadStates {
  constructor() {
    obj = Object.create(new.target.prototype);
    closure_0 = obj;
    obj.readStateVersion = null;
    obj.actions = {
      CONNECTION_OPEN(arg0) {
        return obj.handleConnectionOpen(arg0);
      },
      CHANNEL_PINS_ACK(version) {
        return obj.handleReadStateAction(version);
      },
      MESSAGE_ACK(version) {
        return obj.handleReadStateAction(version);
      },
      BACKGROUND_SYNC_FINISHED(messagesOnly, arg1) {
        if (!messagesOnly.messagesOnly) {
          obj.handleWriteCaches(arg1, false);
        }
      },
      WRITE_CACHES(arg0, arg1) {
        return obj.handleWriteCaches(arg1, true);
      },
    };
    return obj;
  }
}
const prototype = ReadStates.prototype;
prototype["getAll"] = function getAll(arg0) {
  closure_0 = arg0;
  return (async () => {
    const _performance2 = performance;
    closure_128_0 = performance.now();
    const states = tmp2(tmp5[5]).readStates(tmp2);
    await states.getMany();
    closure_128_1 = value;
    const _performance = performance;
    closure_128_2 = performance.now();
    const _HermesInternal = HermesInternal;
    logger.log(
      "asynchronously loaded in " + closure_128_2 - closure_128_0 + "ms (readStates: " + closure_128_1.length + ")",
    );
    return closure_128_1;
  })();
};
prototype["resetInMemoryState"] = function resetInMemoryState() {
  this.readStateVersion = null;
};
prototype["handleConnectionOpen"] = function handleConnectionOpen(readState) {
  this.readStateVersion = readState.readState.version;
};
prototype["handleReadStateAction"] = function handleReadStateAction(version) {
  if (null != this.readStateVersion) {
    if (null != version.version) {
      tmp.readStateVersion = version.version;
    } else {
      logger.log("Received null read states version", version);
    }
  }
};
prototype["handleWriteCaches"] = function handleWriteCaches(database, arg1) {
  const allReadStates = ReadStateStore.getAllReadStates(false);
  if (arg1) {
    if (null != this.readStateVersion) {
      let str2 = "0";
      const _Object = Object;
      const keys = Object.keys(ChannelStore.getMutablePrivateChannels());
      const _Set = Set;
      const set = new Set(keys);
      const sorted = applyDefault(keys).sort(SnowflakeUtilsDefault.compare);
      const obj11 = applyDefault(keys);
      let str = sorted.reverse().value()[0];
      if (str == null) {
        str = "0";
      }
      let _lastMessageId = str;
      const iter = allReadStates[Symbol.iterator]();
      const nextResult = iter.next();
      while (iter !== undefined) {
        let tmp7 = nextResult;
        let tmp9 = nextResult.type === ReadStateTypes.CHANNEL;
        if (tmp9) {
          tmp9 = null != tmp7._lastMessageId;
        }
        if (tmp9) {
          let obj = SnowflakeUtilsDefault;
          if (1 === obj.compare(tmp7._lastMessageId, str2)) {
            str2 = tmp7._lastMessageId;
          }
          let hasItem = set.has(tmp7.channelId);
          if (hasItem) {
            let tmp12Result = SnowflakeUtilsDefault;
            hasItem = 1 === tmp12Result.compare(tmp7._lastMessageId, _lastMessageId);
          }
          if (hasItem) {
            _lastMessageId = tmp7._lastMessageId;
          }
        }
        continue;
      }
      const iter2 = sorted.reverse();
      const result = DatabaseDaosDefault.nonGuildVersionsTransaction(database);
      const obj2 = { id: "highest_last_message_id", versionString: str2 };
      items = [obj2, ,];
      const obj4 = { id: "private_channels_version", versionString: _lastMessageId };
      items[1] = obj4;
      const obj5 = { id: "read_state_version", version: tmp.readStateVersion };
      items[2] = obj5;
      result.putAll(items);
    }
  }
  const statesTransaction = DatabaseDaosDefault.readStatesTransaction(database);
  statesTransaction.delete();
  const item = allReadStates.forEach((type) => statesTransaction.put("" + type.type + "-" + type.channelId, type));
};
let obj2 = Object.create(ReadStates.prototype);
let closure_129_0 = obj2;
obj2.readStateVersion = null;
obj2.actions = {
  CONNECTION_OPEN(arg0) {
    return obj.handleConnectionOpen(arg0);
  },
  CHANNEL_PINS_ACK(version) {
    return obj.handleReadStateAction(version);
  },
  MESSAGE_ACK(version) {
    return obj.handleReadStateAction(version);
  },
  BACKGROUND_SYNC_FINISHED(messagesOnly, arg1) {
    if (!messagesOnly.messagesOnly) {
      obj.handleWriteCaches(arg1, false);
    }
  },
  WRITE_CACHES(arg0, arg1) {
    return obj.handleWriteCaches(arg1, true);
  },
};
const size = fn(2);
let result = size.fileFinishedImporting("modules/app_database/modules/ReadStates.tsx");

export default obj2;
