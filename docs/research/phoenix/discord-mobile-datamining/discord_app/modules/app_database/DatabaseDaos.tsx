// discord_app/modules/app_database/DatabaseDaos.tsx
import initializeDefault from "../../../discord_common/js/packages/flux/index.tsx";
import DispatcherDefault from "../../Dispatcher.tsx";
import DatabaseManagerDefault from "system/DatabaseManager.tsx";
import AuthenticationStore from "../../stores/AuthenticationStore.tsx";

let require = fn;
const items = [fn(4792).TableId.KvCache, "guild_basic_channels"];
const items1 = [items];
const items2 = [fn(4792).TableId.KvCache, "basic_channels_stale"];
items1[1] = items2;
const Store = initializeDefault.Store;
class DatabaseDaos extends Store {
  constructor() {
    applyArgumentsResult = HermesBuiltin.applyArguments(new.target, new.target);
    closure_0 = applyArgumentsResult;
    applyArgumentsResult.database = function database(arg0) {
      id = arg0;
      if (arg0 == null) {
        id = id.getId();
      }
      return DatabaseManagerDefault.database(id);
    };
    closure_129_0 = (database) => {
      const guildEntityDao = new applyArgumentsResult(4792).GuildEntityDao(
        "guild_channels",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return guildEntityDao;
    };
    applyArgumentsResult.channels = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_130_0 = (database) => {
      const guildDao = new applyArgumentsResult(4792).GuildDao(
        "guild_channels_temp",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return guildDao;
    };
    applyArgumentsResult.channelsTemp = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_131_0 = (database) => {
      const dao = new applyArgumentsResult(4792).Dao(
        "basic_channels",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return dao;
    };
    applyArgumentsResult.basicChannels = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_132_0 = (database) => {
      const dao = new applyArgumentsResult(4792).Dao(
        "basic_channels_synced",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return dao;
    };
    applyArgumentsResult.syncedBasicChannels = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_133_0 = (database) => {
      const dao = new applyArgumentsResult(4792).Dao("cache", applyArgumentsResult(4792).TableId.KvCache, database);
      return dao;
    };
    applyArgumentsResult.cache = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_134_0 = (database) => {
      const dao = new applyArgumentsResult(4792).Dao(
        "force_resync_version",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return dao;
    };
    applyArgumentsResult.forceResyncVersion = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_135_0 = (database) => {
      const guildEntityDao = new applyArgumentsResult(4792).GuildEntityDao(
        "guild_emojis",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return guildEntityDao;
    };
    applyArgumentsResult.emojis = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_136_0 = (database) => {
      const entityDao = new applyArgumentsResult(4792).EntityDao(
        "guilds",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return entityDao;
    };
    applyArgumentsResult.guilds = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_137_0 = (database) => {
      const entityDao = new applyArgumentsResult(4792).EntityDao(
        "guilds_requiring_deleted_ids_sync",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return entityDao;
    };
    applyArgumentsResult.guildsRequiringDeletedIdsSync = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_138_0 = (database) => {
      const entityDao = new applyArgumentsResult(4792).EntityDao(
        "guilds_requiring_channel_sync",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return entityDao;
    };
    applyArgumentsResult.guildsRequiringChannelSync = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_139_0 = (database) => {
      const messageDao = new applyArgumentsResult(4792).MessageDao(
        "messages",
        applyArgumentsResult(4792).TableId.Messages,
        database,
      );
      return messageDao;
    };
    applyArgumentsResult.messages = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_140_0 = (database) => {
      const guildEntityDao = new applyArgumentsResult(4792).GuildEntityDao(
        "guild_stickers",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return guildEntityDao;
    };
    applyArgumentsResult.stickers = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_141_0 = (database) => {
      const entityDao = new applyArgumentsResult(4792).EntityDao(
        "guild_versions",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return entityDao;
    };
    applyArgumentsResult.guildVersions = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_142_0 = (database) => {
      const entityDao = new applyArgumentsResult(4792).EntityDao(
        "non_guild_versions",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return entityDao;
    };
    applyArgumentsResult.nonGuildVersions = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_143_0 = (database) => {
      const entityDao = new applyArgumentsResult(4792).EntityDao(
        "user_settings",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return entityDao;
    };
    applyArgumentsResult.userSettings = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_144_0 = (database) => {
      const dao = new applyArgumentsResult(4792).Dao(
        "read_states",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return dao;
    };
    applyArgumentsResult.readStates = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_145_0 = (database) => {
      const dao = new applyArgumentsResult(4792).Dao(
        "user_guild_settings",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return dao;
    };
    applyArgumentsResult.userGuildSettings = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    closure_146_0 = (database) => {
      const entityDao = new applyArgumentsResult(4792).EntityDao(
        "user_search_items",
        applyArgumentsResult(4792).TableId.KvCache,
        database,
      );
      return entityDao;
    };
    applyArgumentsResult.userSearchItems = (arg0) => {
      let databaseResult = arg0;
      if (arg0 == null) {
        databaseResult = DatabaseManagerDefault.database(AuthenticationStore.getId());
      }
      let tmp5 = null;
      if (null != databaseResult) {
        tmp5 = applyArgumentsResult(databaseResult);
      }
      return tmp5;
    };
    applyArgumentsResult.channelsTransaction = function channelsTransaction(database) {
      return applyArgumentsResult.channels(database.database).upgradeTransaction(database);
    };
    applyArgumentsResult.channelsTempTransaction = function channelsTempTransaction(database) {
      return applyArgumentsResult.channelsTemp(database.database).upgradeTransaction(database);
    };
    applyArgumentsResult.basicChannelsTransaction = function basicChannelsTransaction(database) {
      return applyArgumentsResult.basicChannels(database.database).upgradeTransaction(database);
    };
    applyArgumentsResult.syncedBasicChannelsTransaction = function syncedBasicChannelsTransaction(database) {
      return applyArgumentsResult.syncedBasicChannels(database.database).upgradeTransaction(database);
    };
    applyArgumentsResult.cacheTransaction = function cacheTransaction(database) {
      return applyArgumentsResult.cache(database.database).upgradeTransaction(database);
    };
    applyArgumentsResult.forceResyncVersionTransaction = function forceResyncVersionTransaction(database) {
      return applyArgumentsResult.forceResyncVersion(database.database).upgradeTransaction(database);
    };
    applyArgumentsResult.emojisTransaction = function emojisTransaction(database) {
      return applyArgumentsResult.emojis(database.database).upgradeTransaction(database);
    };
    applyArgumentsResult.guildsTransaction = function guildsTransaction(database) {
      return applyArgumentsResult.guilds(database.database).upgradeTransaction(database);
    };
    applyArgumentsResult.messagesTransaction = function messagesTransaction(database) {
      return applyArgumentsResult.messages(database.database).upgradeTransaction(database);
    };
    applyArgumentsResult.stickersTransaction = function stickersTransaction(database) {
      return applyArgumentsResult.stickers(database.database).upgradeTransaction(database);
    };
    applyArgumentsResult.guildVersionsTransaction = function guildVersionsTransaction(database) {
      return applyArgumentsResult.guildVersions(database.database).upgradeTransaction(database);
    };
    applyArgumentsResult.nonGuildVersionsTransaction = function nonGuildVersionsTransaction(database) {
      return applyArgumentsResult.nonGuildVersions(database.database).upgradeTransaction(database);
    };
    applyArgumentsResult.userSettingsTransaction = function userSettingsTransaction(database) {
      return applyArgumentsResult.userSettings(database.database).upgradeTransaction(database);
    };
    applyArgumentsResult.readStatesTransaction = function readStatesTransaction(database) {
      const states = applyArgumentsResult.readStates(database.database);
      return states.upgradeTransaction(database);
    };
    applyArgumentsResult.userGuildSettingsTransaction = function userGuildSettingsTransaction(database) {
      return applyArgumentsResult.userGuildSettings(database.database).upgradeTransaction(database);
    };
    applyArgumentsResult.guildsRequiringDeletedIdsSyncTransaction = function guildsRequiringDeletedIdsSyncTransaction(
      database,
    ) {
      const result = applyArgumentsResult.guildsRequiringDeletedIdsSync(database.database);
      return result.upgradeTransaction(database);
    };
    applyArgumentsResult.guildsRequiringChannelSyncTransaction = function guildsRequiringChannelSyncTransaction(
      database,
    ) {
      const result = applyArgumentsResult.guildsRequiringChannelSync(database.database);
      return result.upgradeTransaction(database);
    };
    applyArgumentsResult.userSearchItemsTransaction = function userSearchItemsTransaction(database) {
      return applyArgumentsResult.userSearchItems(database.database).upgradeTransaction(database);
    };
    return applyArgumentsResult;
  }
}
DatabaseDaos.prototype["initialize"] = function initialize() {
  this.waitFor(AuthenticationStore);
};
const databaseDaos = new DatabaseDaos(DispatcherDefault, {});
const size = fn(2);
let result = size.fileFinishedImporting("modules/app_database/DatabaseDaos.tsx");

export default databaseDaos;
export const DEPRECATED_KEYSPACES = items1;
