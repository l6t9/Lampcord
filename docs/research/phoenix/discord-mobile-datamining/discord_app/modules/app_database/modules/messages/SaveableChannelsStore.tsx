// discord_app/modules/app_database/modules/messages/SaveableChannelsStore.tsx
import ExtendedMemoryLru from "../../util/ExtendedMemoryLru.tsx";
import Lru from "../../util/Lru.tsx";
import isPrivateChannel from "isPrivateChannel.tsx";
import isReadableChannel from "isReadableChannel.tsx";
import isLimitedChannel from "isLimitedChannel.tsx";
import withFallbacks from "withFallbacks.tsx";
import ChannelStore from "../../../../stores/ChannelStore.tsx";
import GuildMemberCountStore from "../../../../stores/GuildMemberCountStore.tsx";
import MobileCacheSnapshotStore from "../../../../stores/MobileCacheSnapshotStore.tsx";
import SelectedChannelStore from "../../../../stores/SelectedChannelStore.tsx";
import FileSystemStore from "../../stores/FileSystemStore.tsx";

require = fn;
function handleSelectedChannelStoreChanged() {
  const channelId = SelectedChannelStore.getChannelId();
  if (null != channelId) {
    SaveableChannelsStore.recordChannel(channelId);
  }
}
function handleConnectionOpenSupplemental() {
  const result = SaveableChannelsStore.dropUnreachableChannels();
  SaveableChannelsStore.replaceLru(withFallbacks.withFallbacks(global, 1250));
}
function handleChannelUpdate(id) {
  id = id.id;
  const isReadableChannelResult = isReadableChannel.isReadableChannel(id);
  let tmp2 = isReadableChannelResult;
  if (isReadableChannelResult) {
    tmp2 = id === SelectedChannelStore.getChannelId();
  }
  if (tmp2) {
    SaveableChannelsStore.recordChannel(id);
  }
  if (!isReadableChannelResult) {
    SaveableChannelsStore.deleteChannel(id);
  }
}
function handleChannelUpdates(arg0) {
  while (tmp !== undefined) {
    let tmp4 = handleChannelUpdate(tmp2);
    continue;
  }
  tmp = arg0.channels[Symbol.iterator]();
}
function handleChannelDelete(channel) {
  SaveableChannelsStore.deleteChannel(channel.channel.id);
}
function handleThreadUpdate(channel) {
  channel = channel.channel;
  const id = channel.id;
  const isReadableChannelResult = isReadableChannel.isReadableChannel(channel);
  let tmp2 = isReadableChannelResult;
  if (isReadableChannelResult) {
    tmp2 = id === SelectedChannelStore.getChannelId();
  }
  if (tmp2) {
    SaveableChannelsStore.recordChannel(id);
  }
  if (!isReadableChannelResult) {
    SaveableChannelsStore.deleteChannel(id);
  }
}
function handleThreadDelete(channel) {
  SaveableChannelsStore.deleteChannel(channel.channel.id);
}
function handleGuildDelete(guild) {
  const unavailable = guild.guild.unavailable;
  let flag = !unavailable;
  if (!unavailable) {
    SaveableChannelsStore.deleteGuild(guild.guild.id);
    flag = true;
  }
  return flag;
}
function handleLoginSuccess() {
  global.clear();
  lru.clear();
  c9 = false;
}
function handleCacheLoadedLazyNoCache() {
  c9 = true;
}
let lastChannel = null;
const bound = Math.max(25, 25, 1);
let extendedMemoryLru = new fn(8273).ExtendedMemoryLru(750, 500);
let global = extendedMemoryLru;
let lru = new fn(8274).Lru(15);
let c9 = false;
let SaveableChannelsStore;
class SaveableChannelsStore extends tmp3 {
  constructor() {
    closure_0 = undefined;
    obj = {
      CACHE_LOADED_LAZY_NO_CACHE: handleCacheLoadedLazyNoCache,
      CACHE_LOADED_LAZY() {
        return closure_0.loadCache();
      },
      CHANNEL_DELETE: handleChannelDelete,
      CHANNEL_UPDATES: handleChannelUpdates,
      CONNECTION_OPEN_SUPPLEMENTAL: handleConnectionOpenSupplemental,
      GUILD_DELETE: handleGuildDelete,
      LOGIN_SUCCESS: handleLoginSuccess,
      THREAD_DELETE: handleThreadDelete,
      THREAD_UPDATE: handleThreadUpdate,
    };
    tmp1 = new tmp(obj, handleThreadDelete, new.target, tmp);
    closure_0 = tmp1;
    return tmp1;
  }
}
const prototype = SaveableChannelsStore.prototype;
prototype["initialize"] = function initialize() {
  this.waitFor(ChannelStore);
  this.waitFor(SelectedChannelStore);
  this.waitFor(GuildMemberCountStore);
  items = [FileSystemStore];
  this.syncWith(items, () => true);
  const items1 = [SelectedChannelStore];
  this.syncWith(items1, handleSelectedChannelStoreChanged);
};
prototype["loadCache"] = function loadCache() {
  const snapshot = this.readSnapshot(SaveableChannelsStore.LATEST_SNAPSHOT_VERSION);
  if (null != snapshot) {
    c9 = true;
    SaveableChannelsStore.mergeSnapshot(snapshot);
  }
};
prototype["canEvictOrphans"] = function canEvictOrphans() {
  return c9;
};
prototype["saveLimit"] = function saveLimit(channelId) {
  const basicChannel = ChannelStore.getBasicChannel(channelId);
  if (null == basicChannel) {
    if (null == basicChannel) {
      let num3 = 1;
    } else {
      num3 = 25;
      if (SelectedChannelStore.getChannelId() !== channelId) {
        num3 = 25;
      }
    }
    let num = num3;
  } else {
    isPrivateChannel;
    num = 25;
  }
  return num;
};
prototype["getSaveableChannels"] = function getSaveableChannels() {
  const channelIds = ChannelStore.getChannelIds(null);
  const mapped = channelIds.map((channelId) => ({ guildId: null, channelId }));
  if (FileSystemStore.isLowDisk) {
    let tmp9 = mapped;
    if (null != obj) {
      items = [];
      items[HermesBuiltin.arraySpread(mapped, 0)] = obj;
      tmp9 = items;
    }
    let items1 = tmp9;
  } else {
    items1 = [];
    HermesBuiltin.arraySpread(global.values(), HermesBuiltin.arraySpread(mapped, 0));
    const arraySpreadResult = HermesBuiltin.arraySpread(mapped, 0);
  }
  return items1;
};
prototype["takeSnapshot"] = function takeSnapshot() {
  lastChannel = { version: SaveableChannelsStore.LATEST_SNAPSHOT_VERSION, data: null };
  const obj2 = { channels: null, penalized: [...lru.keys()], lastChannel };
  items = [...global.allValues()];
  obj2.channels = items.filter((fallback) => !fallback.fallback);
  lastChannel.data = obj2;
  return lastChannel;
};
SaveableChannelsStore["mergeSnapshot"] = function mergeSnapshot(snapshot) {
  let obj = global;
  const extendedMemoryLru = new ExtendedMemoryLru.ExtendedMemoryLru(global.primaryCapacity, global.extendedCapacity);
  global = extendedMemoryLru;
  lru = new Lru.Lru(lru.capacity);
  lastChannel = obj;
  if (obj == null) {
    lastChannel = snapshot.lastChannel;
  }
  obj = lastChannel;
  items = [snapshot.channels, obj.values()];
  for (const item10038 of items) {
    for (const item10043 of item10038) {
      if (!item10043.fallback) {
        let putResult = global.put(item10043.channelId, item10043);
      }
      continue;
    }
    continue;
  }
  const set = new Set();
  const items1 = [snapshot.penalized, lru.keys()];
  for (const item10067 of items1) {
    for (const item10072 of item10067) {
      let putResult1 = lru.put(item10072, null);
      if (null != putResult1) {
        let addResult = set.add(tmp14[0]);
      }
      continue;
    }
    continue;
  }
  for (const item10087 of tmp9) {
    if (!lru.has(item10087)) {
      let deleteResult = global.delete(item10087);
    }
    continue;
  }
  const tmp9 = set;
};
SaveableChannelsStore["recordChannel"] = function recordChannel(id) {
  const basicChannel = ChannelStore.getBasicChannel(id);
  if (null != basicChannel) {
    if (obj3.isReadableChannel(basicChannel)) {
      let guild_id = basicChannel.guild_id;
      if (guild_id == null) {
        guild_id = null;
      }
      const obj = { guildId: guild_id, channelId: id, channelType: basicChannel.type };
      global.put(id, obj);
      if (tmp9Result.isLimitedChannel(basicChannel)) {
        const putResult1 = lru.put(id, null);
        if (null != putResult1) {
          global.delete(putResult1[0]);
        }
      }
      tmp9Result = isLimitedChannel;
    }
    obj3 = isReadableChannel;
  }
};
SaveableChannelsStore["deleteChannel"] = function deleteChannel(arg0) {
  global.delete(arg0);
};
SaveableChannelsStore["deleteGuild"] = function deleteGuild(arg0) {
  items = [...global.allValues()];
  for (const item10013 of items) {
    if (item10013.guildId === arg0) {
      let deleteResult = global.delete(tmp.channelId);
    }
    continue;
  }
};
SaveableChannelsStore["dropUnreachableChannels"] = function dropUnreachableChannels() {
  items = [...global.allKeys()];
  for (const item10012 of items) {
    let basicChannel = ChannelStore.getBasicChannel(item10012);
    let obj = isReadableChannel;
    if (!obj.isReadableChannel(basicChannel)) {
      let deleteChannelResult = SaveableChannelsStore.deleteChannel(item10012);
    }
    continue;
  }
};
SaveableChannelsStore["deleteUnreadableGuildChannels"] = function deleteUnreadableGuildChannels(arg0) {
  items = [...global.allValues()];
  for (const item10013 of items) {
    let isReadableChannelIdResult = arg0 !== item10013.guildId;
    if (!isReadableChannelIdResult) {
      let obj = isReadableChannel;
      isReadableChannelIdResult = obj.isReadableChannelId(item10013.channelId);
    }
    if (!isReadableChannelIdResult) {
      let deleteChannelResult = SaveableChannelsStore.deleteChannel(item10013.channelId);
    }
    continue;
  }
};
SaveableChannelsStore["replaceLru"] = function replaceLru(arg0) {
  global = arg0;
};
SaveableChannelsStore.displayName = "SaveableChannelsStore";
SaveableChannelsStore.LATEST_SNAPSHOT_VERSION = 1;
let closure_129_0;
lastChannel = {
  CACHE_LOADED_LAZY_NO_CACHE: handleCacheLoadedLazyNoCache,
  CACHE_LOADED_LAZY: null,
  CHANNEL_DELETE: null,
  CHANNEL_UPDATES: null,
  CONNECTION_OPEN_SUPPLEMENTAL: null,
  GUILD_DELETE: null,
  LOGIN_SUCCESS: null,
  THREAD_DELETE: null,
  THREAD_UPDATE: null,
};
class CACHE_LOADED_LAZY {
  constructor() {
    return closure_0.loadCache();
  }
}
lastChannel.CACHE_LOADED_LAZY = CACHE_LOADED_LAZY;
lastChannel.CHANNEL_DELETE = handleChannelDelete;
lastChannel.CHANNEL_UPDATES = handleChannelUpdates;
lastChannel.CONNECTION_OPEN_SUPPLEMENTAL = handleConnectionOpenSupplemental;
lastChannel.GUILD_DELETE = handleGuildDelete;
lastChannel.LOGIN_SUCCESS = handleLoginSuccess;
lastChannel.THREAD_DELETE = handleThreadDelete;
lastChannel.THREAD_UPDATE = handleThreadUpdate;
const prototype1 = new prototype(
  lastChannel,
  500,
  tmp,
  Object,
  CACHE_LOADED_LAZY,
  handleChannelDelete,
  handleChannelUpdates,
  handleConnectionOpenSupplemental,
  handleGuildDelete,
  handleLoginSuccess,
  handleThreadDelete,
  SaveableChannelsStore,
  prototype,
  new.target,
);
closure_129_0 = prototype1;
const size = fn(2);
let result = size.fileFinishedImporting("modules/app_database/modules/messages/SaveableChannelsStore.tsx");

export default prototype1;
export const MAXIMUM_MESSAGES_PER_CHANNEL_DM = 25;
export const MAXIMUM_MESSAGES_PER_CHANNEL_NON_DM = 25;
export const MAXIMUM_MESSAGES_PER_CHANNEL_DEFAULT = 1;
export const MAXIMUM_MESSAGES_PER_CHANNEL_EVER = bound;
