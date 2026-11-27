// discord_app/modules/app_startup/AllCacheStores.native.tsx
import SaveableChannelsStore from "../app_database/modules/messages/SaveableChannelsStore.tsx";
import GatedChannelStore from "../channel/GatedChannelStore.tsx";
import ClientThemesBackgroundStore from "../client_themes/ClientThemesBackgroundStore.tsx";
import CustomThemeMobileStore from "../client_themes/native/CustomThemeMobileStore.tsx";
import ExperimentStore from "../experiments/ExperimentStore.tsx";
import MemberSafetyStore from "../guild_mod_dash_member_safety/MemberSafetyStore.tsx";
import ChannelListStore from "../guild_sidebar/ChannelListStore.tsx";
import GuildThemeRuntimeStore from "../guild_themes/GuildThemeRuntimeStore.tsx";
import MessageRequestStore from "../message_request/MessageRequestStore.tsx";
import SpamMessageRequestStore from "../message_request/SpamMessageRequestStore.tsx";
import EphemeralMessageStore from "../messages/EphemeralMessageStore.tsx";
import FamilyCenterStore from "../parent_tools/FamilyCenterStore.tsx";
import NewChannelsStore from "../recent_channels/NewChannelsStore.tsx";
import ReferencedMessageStore from "../replies/ReferencedMessageStore.tsx";
import UserProfileStore from "../user_profile/UserProfileStore.tsx";
import LocaleStore from "../user_settings/LocaleStore.tsx";
import ThemeStore from "../user_settings/ThemeStore.tsx";
import UserSettingsProtoStore from "../user_settings/UserSettingsProtoStore.tsx";
import ChannelStore from "../../stores/ChannelStore.tsx";
import GuildCategoryStore from "../../stores/GuildCategoryStore.tsx";
import GuildChannelStore from "../../stores/GuildChannelStore.tsx";
import GuildMemberStore from "../../stores/GuildMemberStore.tsx";
import GuildMembershipStore from "../../stores/GuildMembershipStore.tsx";
import GuildReadStateStore from "../../stores/GuildReadStateStore.tsx";
import GuildRoleStore from "../../stores/GuildRoleStore.tsx";
import GuildStore from "../../stores/GuildStore.tsx";
import MessageStore from "../../stores/MessageStore.tsx";
import PermissionStore from "../../stores/PermissionStore.tsx";
import ReadStateStore from "../../stores/ReadStateStore.tsx";
import SortedGuildStore from "../../stores/SortedGuildStore.tsx";
import UserGuildSettingsStore from "../../stores/UserGuildSettingsStore.tsx";
import UserStore from "../../stores/UserStore.tsx";
import NUFStore from "../../stores/native/NUFStore.tsx";
import PrivateChannelSortStore from "../../stores/views/PrivateChannelSortStore.tsx";
import size from "../../../_runtime/metro/00002__.js";

const result = size.fileFinishedImporting("modules/app_startup/AllCacheStores.native.tsx");

export default function noop() {}
