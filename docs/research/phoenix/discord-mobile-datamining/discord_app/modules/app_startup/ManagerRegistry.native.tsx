// discord_app/modules/app_startup/ManagerRegistry.native.tsx
import PlatformUtils from "../../utils/PlatformUtils.tsx";
import ManagerRegistryShared from "ManagerRegistryShared.tsx";
import size from "../../../_runtime/metro/00002__.js";

const require = globalThis.__r;

const obj = {
  AcceptInviteManager: {
    actions: ["DISPLAYED_INVITE_SHOW", "DISPLAYED_INVITE_CLEAR", "DEFERRED_INVITE_SHOW", "REGISTER_SUCCESS"],
    inlineRequire() {
      return require("AcceptInviteManager").default;
    },
  },
  AgeGateManager: {
    actions: ["POST_CONNECTION_OPEN", "CHANNEL_SELECT", "AGE_GATE_MODAL_OPEN", "AGE_GATE_MODAL_CLOSE", "GUILD_UPDATE"],
    inlineRequire() {
      return require("AgeGateManager").default;
    },
    neverLoadBeforeConnectionOpen: true,
  },
  AgeVerificationManager: {
    actions: ["POST_CONNECTION_OPEN", "CURRENT_USER_UPDATE", "MESSAGE_CREATE"],
    inlineRequire() {
      return require("AgeVerificationManager").default;
    },
    neverLoadBeforeConnectionOpen: true,
  },
  AppAnalyticsManager: {
    inlineRequire() {
      return require("AppAnalyticsManager").default;
    },
    hasStoreChangeListeners: true,
  },
  AppIconPremiumManager: {
    actions: ["APP_STATE_UPDATE"],
    inlineRequire() {
      return require("AppIconPremiumManager").default;
    },
    neverLoadBeforeConnectionOpen: true,
  },
  AutomodRemovedContentManager: {
    actions: ["AUTO_MODERATION_CONTENT_DELETED"],
    inlineRequire() {
      return require("AutomodRemovedContentManager").default;
    },
  },
  AppStoreOverlayTelemetryManager: {
    actions: ["APP_STATE_UPDATE"],
    inlineRequire() {
      return require("AppStoreOverlayTelemetryManager").default;
    },
  },
  ATTManager: null,
  AudioSessionModeManager: null,
  AudioSettingsManager: null,
  BackgroundSyncManager: null,
  BlockedDomainManager: null,
  BugReportManager: null,
  CallIdleManager: null,
  CacheManager: null,
  ChangelogManager: null,
  ChannelCallManager: null,
  ChannelResyncManager: null,
  ChannelSafetyWarningsManager: null,
  CommonTriggerPointManager: null,
  CommunicationDisabledManager: null,
  ConjureBackgroundFramesManager: null,
  ConjureProjectReconnectManager: null,
  ConjureTurnNotificationManager: null,
  ContactSyncLifecycleManager: null,
  CustomStatusManager: null,
  DeclarativeSystemNotifPermissionManager: null,
  DeviceSilentModeManager: null,
  DiceRollLifecycleManager: null,
  DmSettingsUpsellManager: null,
  TopEmojisDataManager: null,
  EmbeddedAppShimManager: null,
  EntityVersionsManager: null,
  ExplicitMediaManager: null,
  KvBackgroundManager: null,
  FeedbackManager: null,
  ForumManager: null,
  ForumGuidelinesManager: null,
  ForumPostAnalyticsManager: null,
  FramesNativeManager: null,
  FrecencyUserSettingsManager: null,
  GameConsoleManager: null,
  GuildOnboardingHomeManager: null,
  GuildOnboardingManager: null,
  GuildPowerupsManager: null,
  GuildRoomManager: null,
  GuildScheduledEventManager: null,
  GuildVerificationManager: null,
  MemberVerificationApplicationManager: null,
  MessageSessionMetadataManager: null,
  HolidayEventsManager: null,
  IAPManager: null,
  InteractionModalManager: null,
  IOSUserIdentifiersManager: null,
  DiskUsageManager: null,
  InstantInviteManager: null,
  JankSessionManager: null,
  JankStatsManager: null,
  JSWatchdogManager: null,
  LoginRequiredActionManager: null,
  MessageCodedLinkManager: null,
  MessageManager: null,
  MessageQueueManager: null,
  MessagePreviewManager: null,
  MetricKitManager: null,
  MultiAccountManagerNative: null,
  NativeOnDemandResourceManager: null,
  NewUserManager: null,
  RedesignNewUserManager: null,
  NotificationsManager: null,
  NotifSettingsProtoManager: null,
  NUFChannelsManager: null,
  ParentalConsentManager: null,
  ParentalConsentWarningManager: null,
  ParticipantFocusManager: null,
  MobileGiftIntentCardManager: null,
  PhoenixRolloutManager: null,
  PromotionsManager: null,
  ProximitySensorManager: null,
  PushNotificationCacheManager: null,
  RelationshipManager: null,
  RTCLatencyTestManager: null,
  RTCReconnectTimeoutManager: null,
  SavedMessagesManager: null,
  MessageRemindersNotificationManager: null,
  SearchTokensManager: null,
  SelectedChannelManager: null,
  SelfPresenceStoreManager: null,
  SettingTreeManagerLifecycleManager: null,
  SharedSpacesWarningManager: null,
  StageBoostUpsellManager: null,
  StageChannelRequestToSpeakMessageManager: null,
  StageSpeakInviteNotificationManager: null,
  StageMusicManager: null,
  StaffMemberPreloaderManager: null,
  SubscriptionManager: null,
  TelecomManager: null,
  ThreadManager: null,
  UrgentSystemDMManagerBase: null,
  FavoriteManager: null,
  UserGuildSettingsManager: null,
  UserSearchManager: null,
  UserSettingsManager: null,
  UserOfferManager: null,
  VideoBackgroundManager: null,
  VoiceChannelSettingsManager: null,
  VoicePermissionManager: null,
  VoiceProcessingErrorManager: null,
  DeprecatedModalManager: null,
  VoicePanelManager: null,
  MediaPostEmbedManager: null,
  ClipsManager: null,
  MidjourneyOnboardingManager: null,
  NativeNotificationsManager: null,
  FriendOnlineTimerManager: null,
  InappropriateConversationsManager: null,
  ContentInventoryManager: null,
  OrbsArcadeFetchManager: null,
  QuestFetchManager: null,
  QuestMobileEmbedVisibilityManager: null,
  MessageSendFailureNotificationManager: null,
  NativeExperimentBridgeManager: null,
  SocialRpcNetworkConfigManager: null,
  ApiRequestConfigManager: null,
  UserSettingsNativeBridgeManager: null,
  ForwardGuildBreadcrumbManager: null,
  PasskeyUpsellManager: null,
  WebAuthnSignalManager: null,
  GlobalDiscoveryServersFeaturedSearchManager: null,
  ReferralMessageManager: null,
  AVErrorManager: null,
  ApplicationStreamingManager: null,
  RequestToStreamManager: null,
  LibdiscoreExperimentManager: null,
  NativeIntentsManager: null,
  NavigationTTIDispatcherManager: null,
  SafetyFlowsManager: null,
  UserApplicationIdentityManager: null,
  LocalMessageCacheStatsManager: null,
  RestrictedHoursManager: null,
  SentryExperimentFeatureFlagManager: null,
  PlayableProgressManager: null,
  QuestProgressManager: null,
  AccountLinkManagerNative: null,
  AccountLinkingCampaignManager: null,
};
obj.ATTManager = {
  actions: PlatformUtils.isIOS()
    ? ["POST_CONNECTION_OPEN", "APP_STATE_UPDATE", "PUSH_NOTIFICATION_AUTHORIZATION_STATUS_UPDATE"]
    : ["POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("ATTManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.AudioSessionModeManager = {
  inlineRequire() {
    return require("AudioSessionModeManager").default;
  },
  hasStoreChangeListeners: true,
};
obj.AudioSettingsManager = {
  actions: [
    "POST_CONNECTION_OPEN",
    "AUDIO_SET_LOCAL_VOLUME",
    "AUDIO_TOGGLE_LOCAL_MUTE",
    "AUDIO_TOGGLE_LOCAL_SOUNDBOARD_MUTE",
    "MEDIA_ENGINE_RESET_SETTINGS",
  ],
  inlineRequire() {
    return require("AudioSettingsManager").default;
  },
};
obj.BackgroundSyncManager = {
  actions: ["MESSAGE_CREATE", "POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("BackgroundSyncManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.BlockedDomainManager = {
  inlineRequire() {
    return require("BlockedDomainManager").default;
  },
  loadAfterConnectionOpen: true,
};
obj.BugReportManager = {
  inlineRequire() {
    return require("BugReportManager").default;
  },
  loadAfterConnectionOpen: true,
};
obj.CallIdleManager = {
  actions: ["VOICE_STATE_UPDATES", "EMBEDDED_ACTIVITY_CLOSE", "CONNECTION_CLOSED"],
  inlineRequire() {
    return require("CallIdleManager").default;
  },
};
obj.CacheManager = {
  actions: ["POST_CONNECTION_OPEN", "CONNECTION_CLOSED", "APP_STATE_UPDATE", "WINDOW_FOCUS"],
  inlineRequire() {
    return require("CacheManager").default;
  },
};
obj.ChangelogManager = {
  actions: ["POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("ChangelogManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.ChannelCallManager = {
  inlineRequire() {
    return require("ChannelCallManager").default;
  },
  hasStoreChangeListeners: true,
};
obj.ChannelResyncManager = {
  actions: ["GUILD_CREATE", "POST_CONNECTION_OPEN", "CHANNEL_SYNC", "LOGOUT"],
  inlineRequire() {
    return require("ChannelResyncManager").default;
  },
};
obj.ChannelSafetyWarningsManager = {
  actions: ["CHANNEL_SELECT", "CHANNEL_UPDATES"],
  inlineRequire() {
    return require("ChannelSafetyWarningsManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.CommonTriggerPointManager = {
  actions: ["VOICE_CHANNEL_SELECT", "CALL_CREATE", "USER_SETTINGS_MODAL_OPEN"],
  inlineRequire() {
    return require("CommonTriggerPointManager").default;
  },
};
obj.CommunicationDisabledManager = {
  inlineRequire() {
    return require("CommunicationDisabledManager").default;
  },
  loadAfterConnectionOpen: true,
};
obj.ConjureBackgroundFramesManager = {
  actions: ["FRAME_LAUNCH_START"],
  inlineRequire() {
    return require("ConjureBackgroundFramesManager").default;
  },
};
obj.ConjureProjectReconnectManager = {
  actions: ["POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("ConjureProjectReconnectManager").default;
  },
};
obj.ConjureTurnNotificationManager = {
  actions: ["CONJURE_TURN_NOTIFICATION", "CONJURE_TURN_SETTLED", "LOGOUT"],
  inlineRequire() {
    return require("ConjureTurnNotificationManager").default;
  },
};
obj.ContactSyncLifecycleManager = {
  actions: ["POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("ContactSyncManager").default;
  },
};
obj.CustomStatusManager = {
  actions: ["USER_SETTINGS_PROTO_UPDATE", "POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("CustomStatusManager").default;
  },
  loadAfterConnectionOpen: true,
};
obj.DeclarativeSystemNotifPermissionManager = {
  actions: ["APP_STATE_UPDATE"],
  inlineRequire() {
    return require("DeclarativeSystemNotifPermissionManager").default;
  },
};
obj.DeviceSilentModeManager = {
  actions: ["FRAME_LAUNCH"],
  inlineRequire() {
    return require("DeviceSilentModeManager").default;
  },
  hasStoreChangeListeners: true,
};
obj.DiceRollLifecycleManager = {
  actions: ["CHANNEL_SELECT", "DICE_ROLL_START"],
  inlineRequire() {
    return require("DiceRollLifecycleManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.DmSettingsUpsellManager = {
  actions: ["DM_SETTINGS_UPSELL_SHOW"],
  inlineRequire() {
    return require("DmSettingsUpsellManager").default;
  },
};
obj.TopEmojisDataManager = {
  actions: ["EMOJI_INTERACTION_INITIATED"],
  inlineRequire() {
    return require("TopEmojisDataManager").default;
  },
};
obj.EmbeddedAppShimManager = {
  actions: ["EMBEDDED_APP_SHIM_PIPE_OPEN"],
  inlineRequire() {
    return require("EmbeddedAppShimManager").default;
  },
  hasStoreChangeListeners: true,
};
obj.EntityVersionsManager = {
  actions: ["GUILD_CREATE", "DELETED_ENTITY_IDS"],
  inlineRequire() {
    return require("EntityVersionsManager").default;
  },
  loadRightBeforeConnectionOpen: true,
};
obj.ExplicitMediaManager = {
  actions: [
    "CHANNEL_SELECT",
    "LOAD_MESSAGES_SUCCESS",
    "MESSAGE_CREATE",
    "MESSAGE_UPDATE",
    "SEARCH_MESSAGES_SUCCESS",
    "MOD_VIEW_SEARCH_MESSAGES_SUCCESS",
    "LOAD_FORUM_POSTS",
    "LOAD_ARCHIVED_THREADS_SUCCESS",
    "LOAD_THREADS_SUCCESS",
    "LOGOUT",
    "LOAD_PINNED_MESSAGES_SUCCESS",
    "USER_SETTINGS_PROTO_UPDATE",
    "CHANNEL_RTC_UPDATE_CHAT_OPEN",
    "SIDEBAR_VIEW_CHANNEL",
  ],
  inlineRequire() {
    return require("ExplicitMediaManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.KvBackgroundManager = {
  actions: ["APP_STATE_UPDATE", "LOGOUT", "POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("KvBackgroundManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.FeedbackManager = {
  actions: ["VOICE_CHANNEL_SHOW_FEEDBACK", "STREAM_CLOSE", "EMBEDDED_ACTIVITY_CLOSE", "IN_APP_REPORTS_SHOW_FEEDBACK"],
  inlineRequire() {
    return require("FeedbackManager").default;
  },
};
obj.ForumManager = {
  actions: ["CHANNEL_PRELOAD"],
  inlineRequire() {
    return require("ForumManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.ForumGuidelinesManager = {
  inlineRequire() {
    return require("ForumGuidelinesManager").default;
  },
  loadAfterConnectionOpen: true,
};
obj.ForumPostAnalyticsManager = {
  actions: ["CHANNEL_SELECT", "THREAD_CREATE"],
  inlineRequire() {
    return require("ForumPostAnalyticsManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.FramesNativeManager = {
  actions: [
    "RPC_APP_DISCONNECTED",
    "FRAME_LAUNCH",
    "FRAME_LAUNCH_FAIL",
    "FRAME_STOP",
    "VOICE_CHANNEL_SELECT",
    "CHANNEL_DELETE",
    "CHANNEL_UPDATES",
    "GUILD_DELETE",
  ],
  inlineRequire() {
    return require("FramesNativeManager").default;
  },
};
obj.FrecencyUserSettingsManager = {
  actions: ["POST_CONNECTION_OPEN", "CONNECTION_RESUMED", "CONNECTION_CLOSED", "APP_STATE_UPDATE", "LOGOUT"],
  inlineRequire() {
    return require("FrecencyUserSettingsManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.GameConsoleManager = {
  actions: [
    "WAIT_FOR_REMOTE_SESSION",
    "POST_CONNECTION_OPEN",
    "SESSIONS_REPLACE",
    "AUDIO_TOGGLE_SELF_DEAF",
    "AUDIO_TOGGLE_SELF_MUTE",
    "VOICE_STATE_UPDATES",
    "CONSOLE_COMMAND_UPDATE",
    "PASSIVE_UPDATE_V2",
    "REMOTE_SESSION_DISCONNECT",
  ],
  inlineRequire() {
    return require("GameConsoleManager").default;
  },
};
obj.GuildOnboardingHomeManager = {
  actions: [
    "POST_CONNECTION_OPEN",
    "CHANNEL_SELECT",
    "MESSAGE_CREATE",
    "THREAD_CREATE",
    "GUILD_MEMBER_UPDATE",
    "GUILD_DELETE",
  ],
  inlineRequire() {
    return require("GuildOnboardingHomeManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.GuildOnboardingManager = {
  actions: ["CHANNEL_SELECT", "GUILD_DELETE", "POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("GuildOnboardingManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.GuildPowerupsManager = {
  actions: ["GUILD_POWERUP_ENTITLEMENTS_CREATE", "GUILD_POWERUP_ENTITLEMENTS_DELETE", "GUILD_APPLIED_BOOSTS_UPDATE"],
  inlineRequire() {
    return require("GuildPowerupsManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
  hasStoreChangeListeners: true,
};
obj.GuildRoomManager = {
  actions: ["VOICE_STATE_UPDATES", "CONNECTION_RESUMED"],
  inlineRequire() {
    return require("GuildRoomManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.GuildScheduledEventManager = {
  actions: ["POST_CONNECTION_OPEN", "GUILD_DELETE", "GUILD_UNAVAILABLE", "INVITE_RESOLVE_SUCCESS", "CHANNEL_SELECT"],
  inlineRequire() {
    return require("GuildScheduledEventManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.GuildVerificationManager = {
  actions: ["INVITE_ACCEPT_SUCCESS"],
  inlineRequire() {
    return require("GuildVerificationManager").default;
  },
};
obj.MemberVerificationApplicationManager = {
  actions: ["GUILD_JOIN_REQUEST_UPDATE", "CHANNEL_SELECT", "POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("MemberVerificationApplicationManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.MessageSessionMetadataManager = {
  actions: ["MESSAGE_UPDATE", "MESSAGE_CREATE"],
  inlineRequire() {
    return require("MessageSessionMetadataManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.HolidayEventsManager = {
  actions: ["NOTIFICATIONS_SET_DISABLED_SOUNDS"],
  inlineRequire() {
    return require("HolidayEventsManager").default;
  },
  hasStoreChangeListeners: true,
};
obj.IAPManager = {
  actions: ["APP_STATE_UPDATE", "POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("IAPManager").default;
  },
};
obj.InteractionModalManager = {
  actions: [
    "INTERACTION_MODAL_CREATE",
    "INTERACTION_IFRAME_MODAL_CREATE",
    "INTERACTION_IFRAME_MODAL_CLOSE",
    "RPC_APP_DISCONNECTED",
  ],
  inlineRequire() {
    return require("InteractionModalManager").default;
  },
};
obj.IOSUserIdentifiersManager = {
  actions: ["POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("IOSUserIdentifiersManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.DiskUsageManager = {
  actions: ["APP_STATE_UPDATE"],
  inlineRequire() {
    return require("DiskUsageManager").default;
  },
};
obj.InstantInviteManager = {
  actions: ["NATIVE_APP_INSTANT_INVITE_GDM_SHARE_FAILED"],
  inlineRequire() {
    return require("InstantInviteManager").default;
  },
};
obj.JankSessionManager = {
  actions: ["APP_STATE_UPDATE", "CONNECTION_OPEN_SUPPLEMENTAL"],
  inlineRequire() {
    return require("JankSessionManager").default;
  },
};
obj.JankStatsManager = {
  actions: ["APP_STATE_UPDATE", "CONNECTION_OPEN_SUPPLEMENTAL"],
  inlineRequire() {
    return require("JankStatsManager").default;
  },
};
obj.JSWatchdogManager = {
  actions: ["APP_STATE_UPDATE", "CONNECTION_OPEN_SUPPLEMENTAL"],
  inlineRequire() {
    return require("JSWatchdogManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.LoginRequiredActionManager = {
  actions: ["POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("LoginRequiredActionManager").default;
  },
};
obj.MessageCodedLinkManager = {
  actions: [
    "POST_CONNECTION_OPEN",
    "MESSAGE_UPDATE",
    "LOAD_MESSAGES_SUCCESS",
    "LOAD_MESSAGES_AROUND_SUCCESS",
    "LOAD_RECENT_MENTIONS_SUCCESS",
    "LOAD_PINNED_MESSAGES_SUCCESS",
    "SEARCH_MESSAGES_SUCCESS",
    "MOD_VIEW_SEARCH_MESSAGES_SUCCESS",
  ],
  inlineRequire() {
    return require("MessageCodedLinkManager").default;
  },
};
obj.MessageManager = {
  actions: [
    "APP_STATE_UPDATE_WILL_BECOME_ACTIVE",
    "OVERLAY_INITIALIZE",
    "CHANNEL_SELECT",
    "VOICE_CHANNEL_SELECT",
    "THREAD_CREATE",
    "THREAD_LIST_SYNC",
    "CHANNEL_CREATE",
    "CHANNEL_PRELOAD",
    "GUILD_CREATE",
    "MESSAGE_END_EDIT",
    "LOAD_MESSAGES_SUCCESS",
    "UPLOAD_FAIL",
    "CHANNEL_DELETE",
    "THREAD_DELETE",
    "CHANNEL_RTC_JUMP_TO_VOICE_CHANNEL_MESSAGE",
  ],
  inlineRequire() {
    return require("MessageManager").default;
  },
  neverLoadBeforeConnectionOpen: false,
  hasStoreChangeListeners: true,
  loadRightBeforeConnectionOpen: true,
};
obj.MessageQueueManager = {
  actions: ["LOGOUT"],
  inlineRequire() {
    return require("MessageQueueManager").default;
  },
};
obj.MessagePreviewManager = {
  actions: [
    "CONNECTION_OPEN_SUPPLEMENTAL",
    "CONNECTION_RESUMED",
    "GUILD_CREATE",
    "GUILD_DELETE",
    "LOAD_MESSAGES_SUCCESS",
    "LOCAL_MESSAGES_LOADED",
    "LOGOUT",
    "MESSAGE_CREATE",
    "MESSAGE_DELETE",
    "MESSAGE_UPDATE",
    "THREAD_LIST_SYNC",
  ],
  inlineRequire() {
    return require("MessagePreviewManager").default;
  },
  neverLoadBeforeConnectionOpen: false,
};
obj.MetricKitManager = {
  inlineRequire() {
    return require("MetricKitManager").default;
  },
  loadAfterConnectionOpen: true,
};
obj.MultiAccountManagerNative = {
  actions: ["LOGOUT", "MULTI_ACCOUNT_SWITCH_START"],
  inlineRequire() {
    return require("MultiAccountManagerNative").default;
  },
  loadAfterConnectionOpen: true,
};
obj.NativeOnDemandResourceManager = {
  actions: ["POST_CONNECTION_OPEN", "APP_STATE_UPDATE"],
  inlineRequire() {
    return require("NativeOnDemandResourceManager").default;
  },
};
obj.NewUserManager = {
  actions: ["ONBOARDING_STEP"],
  inlineRequire() {
    return require("NewUserManager").default;
  },
};
obj.RedesignNewUserManager = {
  actions: ["POST_CONNECTION_OPEN", "ONBOARDING_START", "USER_REQUIRED_ACTION_UPDATE"],
  inlineRequire() {
    return require("RedesignNewUserManager").default;
  },
};
obj.NotificationsManager = {
  actions: [
    "MESSAGE_CREATE",
    "MESSAGE_REACTION_ADD",
    "INVITE_ACCEPT_SUCCESS",
    "RELATIONSHIP_ADD",
    "POST_CONNECTION_OPEN",
    "APP_STATE_UPDATE",
  ],
  inlineRequire() {
    return require("NotificationsManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.NotifSettingsProtoManager = {
  actions: [
    "POST_CONNECTION_OPEN",
    "CONNECTION_RESUMED",
    "APP_STATE_UPDATE",
    "USER_SETTINGS_PROTO_UPDATE",
    "NOTIFICATION_SETTINGS_UPDATE",
    "EFFECTIVE_NOTIFICATION_SETTINGS_SCREEN_OPEN",
    "EFFECTIVE_NOTIFICATION_SETTINGS_ARM_CHANGE",
    "DECLARATIVE_NOTIFICATION_SETTINGS_UPDATE",
    "LOGOUT",
  ],
  inlineRequire() {
    return require("NotifSettingsProtoManager").default;
  },
  hasStoreChangeListeners: true,
  neverLoadBeforeConnectionOpen: true,
};
obj.NUFChannelsManager = {
  actions: ["LOGOUT"],
  inlineRequire() {
    return require("NUFChannelsManager").default;
  },
  loadAfterConnectionOpen: true,
};
obj.ParentalConsentManager = {
  actions: ["CONNECTION_OPEN_SUPPLEMENTAL", "POST_CONNECTION_OPEN", "APP_STATE_UPDATE", "LOGOUT"],
  inlineRequire() {
    return require("ParentalConsentManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.ParentalConsentWarningManager = {
  actions: [
    "POST_CONNECTION_OPEN",
    "APP_STATE_UPDATE",
    "CURRENT_USER_UPDATE",
    "LOGOUT",
    "PARENTAL_CONSENT_WARNING_FETCH_SUCCESS",
  ],
  inlineRequire() {
    return require("ParentalConsentWarningManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.ParticipantFocusManager = {
  inlineRequire() {
    return require("ParticipantFocusManager").default;
  },
  hasStoreChangeListeners: true,
};
obj.MobileGiftIntentCardManager = {
  actions: [
    "POST_CONNECTION_OPEN",
    "CHANNEL_SELECT",
    "GIFT_INTENT_DISMISSALS_FETCH_SUCCESS",
    "GIFT_INTENT_DISMISSALS_FETCH_FAILURE",
    "LOGOUT",
  ],
  inlineRequire() {
    return require("MobileGiftIntentCardManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
  hasStoreChangeListeners: true,
};
obj.PhoenixRolloutManager = {
  actions: ["POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("PhoenixRolloutManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.PromotionsManager = {
  actions: [
    "POST_CONNECTION_OPEN",
    "EXPERIMENTS_FETCH_SUCCESS",
    "IAP_PURCHASE_PRODUCT_SUCCESS",
    "GPLAY_PURCHASE_VERIFIED",
    "BILLING_USER_OFFER_ACKNOWLEDGED_SUCCESS",
    "BILLING_USER_TRIAL_OFFER_ACKNOWLEDGED_SUCCESS",
    "BILLING_REFERRAL_RESOLVE_SUCCESS",
    "VIRTUAL_CURRENCY_REDEEM_SUCCESS",
  ],
  inlineRequire() {
    return require("PromotionsManager").default;
  },
  hasStoreChangeListeners: true,
};
obj.ProximitySensorManager = {
  inlineRequire() {
    return require("ProximitySensorManager").default;
  },
  hasStoreChangeListeners: true,
};
obj.PushNotificationCacheManager = {
  actions: ["POST_CONNECTION_OPEN", "CURRENT_USER_UPDATE", "LOGOUT"],
  hasStoreChangeListeners: true,
  inlineRequire() {
    return require("PushNotificationCacheManager").default;
  },
};
obj.RelationshipManager = {
  actions: ["RELATIONSHIP_ADD", "FRIEND_REQUEST_ACCEPTED"],
  inlineRequire() {
    return require("RelationshipManager").default;
  },
};
obj.RTCLatencyTestManager = {
  actions: ["POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("RTCLatencyTestManager").default;
  },
};
obj.RTCReconnectTimeoutManager = {
  actions: ["VOICE_STATE_UPDATES"],
  inlineRequire() {
    return require("RTCReconnectTimeoutManager").default;
  },
};
obj.SavedMessagesManager = {
  actions: ["POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("SavedMessagesManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.MessageRemindersNotificationManager = {
  actions: ["SAVED_MESSAGES_UPDATE", "SAVED_MESSAGE_CREATE", "SAVED_MESSAGE_DELETE"],
  inlineRequire() {
    return require("MessageRemindersNotificationManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.SearchTokensManager = {
  actions: ["USER_SETTINGS_PROTO_UPDATE", "POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("SearchTokensManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.SelectedChannelManager = {
  actions: ["GUILD_CREATE", "CHANNEL_CREATE", "LOGOUT"],
  inlineRequire() {
    return require("SelectedChannelManager").default;
  },
};
obj.SelfPresenceStoreManager = {
  inlineRequire() {
    return require("SelfPresenceStoreManager").default;
  },
  hasStoreChangeListeners: true,
};
obj.SettingTreeManagerLifecycleManager = {
  actions: ["POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("SettingTreeCacheLifecycleManager").default;
  },
};
obj.SharedSpacesWarningManager = {
  actions: ["CHANNEL_SELECT", "APP_STATE_UPDATE"],
  inlineRequire() {
    return require("SharedSpacesWarningManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.StageBoostUpsellManager = {
  actions: ["VOICE_CHANNEL_SELECT", "VOICE_STATE_UPDATES"],
  inlineRequire() {
    return require("StageBoostUpsellManager").default;
  },
};
obj.StageChannelRequestToSpeakMessageManager = {
  actions: ["VOICE_STATE_UPDATES"],
  inlineRequire() {
    return require("StageChannelRequestToSpeakMessageManager").default;
  },
};
obj.StageSpeakInviteNotificationManager = {
  actions: ["VOICE_STATE_UPDATES"],
  inlineRequire() {
    return require("StageSpeakInviteNotificationManager").default;
  },
};
obj.StageMusicManager = {
  actions: [
    "VOICE_CHANNEL_SELECT",
    "LOGOUT",
    "STAGE_MUSIC_MUTE",
    "STAGE_MUSIC_PLAY",
    "VOICE_STATE_UPDATES",
    "AUDIO_SET_OUTPUT_VOLUME",
    "AUDIO_TOGGLE_SELF_DEAF",
  ],
  inlineRequire() {
    return require("StageMusicManager").default;
  },
};
obj.StaffMemberPreloaderManager = {
  actions: ["POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("StaffMemberPreloaderManager").default;
  },
};
obj.SubscriptionManager = {
  actions: ["POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("SubscriptionManager").default;
  },
};
obj.TelecomManager = {
  actions: ["CALL_CREATE", "CALL_UPDATE", "CALL_DELETE"],
  inlineRequire() {
    return require("TelecomManager").default;
  },
  hasStoreChangeListeners: true,
};
obj.ThreadManager = {
  actions: ["CHANNEL_DELETE", "MESSAGE_CREATE", "GUILD_DELETE"],
  inlineRequire() {
    return require("ThreadManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.UrgentSystemDMManagerBase = {
  actions: ["POST_CONNECTION_OPEN", "MESSAGE_CREATE", "CHANNEL_SELECT"],
  inlineRequire() {
    return require("UrgentSystemDMManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.FavoriteManager = {
  actions: [
    "THREAD_CREATE",
    "CHANNEL_DELETE",
    "CATEGORY_COLLAPSE",
    "CATEGORY_EXPAND",
    "CATEGORY_COLLAPSE_ALL",
    "CATEGORY_EXPAND_ALL",
    "LOGOUT",
    "THREAD_MEMBERS_UPDATE",
    "THREAD_MEMBER_UPDATE",
  ],
  inlineRequire() {
    return require("FavoriteManager").default;
  },
};
obj.UserGuildSettingsManager = {
  actions: [
    "CATEGORY_COLLAPSE",
    "CATEGORY_EXPAND",
    "CATEGORY_COLLAPSE_ALL",
    "CATEGORY_EXPAND_ALL",
    "POST_CONNECTION_OPEN",
    "USER_GUILD_SETTINGS_FULL_UPDATE",
  ],
  inlineRequire() {
    return require("UserGuildSettingsManager").default;
  },
};
obj.UserSearchManager = {
  actions: [
    "LOGOUT",
    "POST_CONNECTION_OPEN",
    "CONNECTION_OPEN_SUPPLEMENTAL",
    "OVERLAY_INITIALIZE",
    "CURRENT_USER_UPDATE",
    "GUILD_CREATE",
    "GUILD_MEMBERS_CHUNK_BATCH",
    "GUILD_MEMBER_ADD",
    "GUILD_MEMBER_UPDATE",
    "RELATIONSHIP_ADD",
    "RELATIONSHIP_UPDATE",
    "RELATIONSHIP_REMOVE",
    "CHANNEL_CREATE",
    "CHANNEL_UPDATES",
    "CHANNEL_RECIPIENT_ADD",
    "PASSIVE_UPDATE_V2",
    "THREAD_LIST_SYNC",
    "LOAD_FORUM_POSTS",
    "LOAD_MESSAGES_SUCCESS",
    "SEARCH_MESSAGES_SUCCESS",
    "LOAD_THREADS_SUCCESS",
    "LOAD_ARCHIVED_THREADS_SUCCESS",
    "LOAD_PINNED_MESSAGES_SUCCESS",
    "GUILD_SCHEDULED_EVENT_USERS_FETCH_SUCCESS",
    "MESSAGE_CREATE",
    "MESSAGE_UPDATE",
  ],
  inlineRequire() {
    return require("UserSearchManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.UserSettingsManager = {
  actions: ["POST_CONNECTION_OPEN", "OVERLAY_INITIALIZE", "USER_SETTINGS_PROTO_UPDATE"],
  inlineRequire() {
    return require("UserSettingsManager").default;
  },
};
obj.UserOfferManager = {
  actions: ["POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("UserOfferManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.VideoBackgroundManager = {
  actions: ["VOICE_CHANNEL_SELECT"],
  inlineRequire() {
    return require("VideoBackgroundManager").default;
  },
};
obj.VoiceChannelSettingsManager = {
  actions: ["CHANNEL_UPDATES", "VOICE_STATE_UPDATES"],
  inlineRequire() {
    return require("VoiceChannelSettingsManager").default;
  },
};
obj.VoicePermissionManager = {
  actions: ["VOICE_CHANNEL_SELECT", "VOICE_STATE_UPDATES"],
  inlineRequire() {
    return require("VoicePermissionManager").default;
  },
};
obj.VoiceProcessingErrorManager = {
  actions: ["MEDIA_ENGINE_NOISE_CANCELLATION_ERROR", "MEDIA_ENGINE_VOICE_ACTIVITY_DETECTION_ERROR"],
  inlineRequire() {
    return require("VoiceProcessingErrorManager").default;
  },
};
obj.DeprecatedModalManager = {
  actions: [
    "CONNECTION_OPEN_SUPPLEMENTAL",
    "GUILD_SETTINGS_CLOSE",
    "NOTIFICATION_SETTINGS_MODAL_CLOSE",
    "PREMIUM_PAYMENT_MODAL_CLOSE",
    "EMAIL_VERIFICATION_MODAL_CLOSE",
    "IFE_EXPERIMENT_SEARCH_MODAL_CLOSE",
    "CREATE_INVITE_MODAL_CLOSE",
    "QUICKSWITCHER_HIDE",
    "EMAIL_VERIFICATION_MODAL_OPEN",
    "USER_REQUIRED_ACTION_UPDATE",
    "GUILD_SETTINGS_OPEN",
    "NOTIFICATION_SETTINGS_MODAL_OPEN",
    "CREATE_INVITE_MODAL_OPEN",
  ],
  inlineRequire() {
    return require("DeprecatedModalManager").default;
  },
};
obj.VoicePanelManager = {
  actions: ["VOICE_CHANNEL_SELECT", "RTC_CONNECTION_STATE"],
  inlineRequire() {
    return require("VoicePanelManager").default;
  },
};
obj.MediaPostEmbedManager = {
  actions: [
    "POST_CONNECTION_OPEN",
    "MESSAGE_UPDATE",
    "LOAD_MESSAGES_SUCCESS",
    "LOAD_MESSAGES_AROUND_SUCCESS",
    "LOAD_RECENT_MENTIONS_SUCCESS",
    "LOAD_PINNED_MESSAGES_SUCCESS",
    "SEARCH_MESSAGES_SUCCESS",
    "MOD_VIEW_SEARCH_MESSAGES_SUCCESS",
    "LOAD_THREADS_SUCCESS",
    "LOAD_ARCHIVED_THREADS_SUCCESS",
    "LOAD_FORUM_POSTS",
  ],
  inlineRequire() {
    return require("MediaPostEmbedManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.ClipsManager = {
  actions: [
    "RTC_CONNECTION_FLAGS",
    "RTC_CONNECTION_STATE",
    "POST_CONNECTION_OPEN",
    "RTC_CONNECTION_USERS_MERGED",
    "CLIPS_ALLOW_VOICE_RECORDING_UPDATE",
    "CLIPS_SETTINGS_UPDATE",
    "CLIPS_INIT_FAILURE",
    "STREAM_START",
    "STREAM_DELETE",
    "STREAM_CLOSE",
    "RUNNING_GAMES_CHANGE",
    "RUNNING_GAME_TOGGLE_DETECTION",
    "CLIPS_RESTART",
    "RTC_CONNECTION_VIDEO",
  ],
  inlineRequire() {
    return require("ClipsManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.MidjourneyOnboardingManager = {
  actions: ["CHANNEL_CREATE"],
  inlineRequire() {
    return require("MidjourneyOnboardingManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.NativeNotificationsManager = {
  actions: ["MESSAGE_ACK", "CHANNEL_SELECT", "POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("NativeNotificationsManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.FriendOnlineTimerManager = {
  actions: [
    "POST_CONNECTION_OPEN",
    "CONNECTION_RESUMED",
    "CONNECTION_CLOSED",
    "CONNECTION_INTERRUPTED",
    "LOGOUT",
    "SELF_PRESENCE_STORE_UPDATE",
  ],
  inlineRequire() {
    return require("FriendOnlineTimer").default;
  },
  loadRightBeforeConnectionOpen: true,
};
obj.InappropriateConversationsManager = {
  actions: ["VIBING_WUMPUS_PLAY_MUSIC", "VIBING_WUMPUS_STOP_MUSIC", "VIBING_WUMPUS_PAUSE_MUSIC"],
  inlineRequire() {
    return require("InappropriateConversationsManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.ContentInventoryManager = {
  actions: [
    "POST_CONNECTION_OPEN",
    "CONNECTION_CLOSED",
    "IDLE",
    "WINDOW_FOCUS",
    "CONTENT_INVENTORY_TOGGLE_FEED_HIDDEN",
    "CONTENT_INVENTORY_MANUAL_REFRESH",
    "CONTENT_INVENTORY_INBOX_STALE",
    "SPOTIFY_NEW_TRACK",
    "GAME_PROFILE_OPEN",
  ],
  inlineRequire() {
    return require("ContentInventoryManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.OrbsArcadeFetchManager = {
  actions: ["POST_CONNECTION_OPEN", "USER_SETTINGS_PROTO_UPDATE", "ORBS_ARCADE_GAME_STATE_UPDATE", "LOGOUT"],
  inlineRequire() {
    return require("OrbsArcadeFetchManager").default;
  },
  hasStoreChangeListeners: true,
  neverLoadBeforeConnectionOpen: true,
};
obj.QuestFetchManager = {
  actions: [
    "QUESTS_FETCH_CURRENT_QUESTS_BEGIN",
    "QUESTS_FETCH_QUEST_HOME_HERO_BEGIN",
    "POST_CONNECTION_OPEN",
    "RUNNING_GAMES_CHANGE",
    "RUNNING_NON_GAMES_CHANGE",
    "USER_SETTINGS_PROTO_UPDATE",
    "START_SESSION",
    "LOGOUT",
  ],
  inlineRequire() {
    return require("QuestFetchManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.QuestMobileEmbedVisibilityManager = {
  actions: ["QUESTS_VISIBLE_MOBILE_MESSAGES_CHANGED"],
  inlineRequire() {
    return require("QuestMobileEmbedVisibilityManager").default;
  },
  hasStoreChangeListeners: true,
  neverLoadBeforeConnectionOpen: true,
};
obj.MessageSendFailureNotificationManager = {
  actions: ["MESSAGE_CREATE", "MESSAGE_SEND_FAILED"],
  inlineRequire() {
    return require("MessageSendFailureNotificationManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.NativeExperimentBridgeManager = {
  actions: ["APP_STATE_UPDATE", "POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("NativeExperimentBridgeManager").default;
  },
};
obj.SocialRpcNetworkConfigManager = {
  actions: ["POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("SocialRpcNetworkConfigManager").default;
  },
};
obj.ApiRequestConfigManager = {
  actions: ["POST_CONNECTION_OPEN", "APP_STATE_UPDATE"],
  inlineRequire() {
    return require("ApiRequestConfigManager").default;
  },
};
obj.UserSettingsNativeBridgeManager = {
  actions: ["POST_CONNECTION_OPEN", "USER_SETTINGS_PROTO_UPDATE"],
  inlineRequire() {
    return require("UserSettingsNativeBridgeManager").default;
  },
};
obj.ForwardGuildBreadcrumbManager = {
  actions: [
    "POST_CONNECTION_OPEN",
    "MESSAGE_UPDATE",
    "LOAD_MESSAGES_SUCCESS",
    "LOAD_MESSAGES_AROUND_SUCCESS",
    "LOAD_RECENT_MENTIONS_SUCCESS",
    "LOAD_PINNED_MESSAGES_SUCCESS",
    "SEARCH_MESSAGES_SUCCESS",
    "MOD_VIEW_SEARCH_MESSAGES_SUCCESS",
  ],
  inlineRequire() {
    return require("ForwardGuildBreadcrumbManager").default;
  },
};
obj.PasskeyUpsellManager = {
  actions: ["POST_CONNECTION_OPEN", "LOGIN_SUCCESS", "LOGIN_RESET", "LOGOUT"],
  inlineRequire() {
    return require("PasskeyUpsellManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
  hasStoreChangeListeners: true,
};
obj.WebAuthnSignalManager = {
  actions: ["AUTHENTICATOR_DELETE", "CURRENT_USER_UPDATE", "MFA_WEBAUTHN_CREDENTIALS_LOADED"],
  inlineRequire() {
    return require("WebAuthnSignalManager").default;
  },
};
obj.GlobalDiscoveryServersFeaturedSearchManager = {
  actions: ["POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("GlobalDiscoveryServersFeaturedSearchManager").default;
  },
};
obj.ReferralMessageManager = {
  actions: [
    "POST_CONNECTION_OPEN",
    "MESSAGE_UPDATE",
    "LOAD_MESSAGES_SUCCESS",
    "LOAD_MESSAGES_AROUND_SUCCESS",
    "LOAD_RECENT_MENTIONS_SUCCESS",
    "LOAD_PINNED_MESSAGES_SUCCESS",
    "SEARCH_MESSAGES_SUCCESS",
    "MOD_VIEW_SEARCH_MESSAGES_SUCCESS",
  ],
  inlineRequire() {
    return require("ReferralMessageManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.AVErrorManager = {
  actions: [
    "MEDIA_ENGINE_SET_AUDIO_ENABLED",
    "AUDIO_INPUT_DETECTED",
    "AUDIO_SET_DISPLAY_SILENCE_WARNING",
    "CERTIFIED_DEVICES_SET",
    "AUDIO_SET_INPUT_DEVICE",
    "AUDIO_SET_OUTPUT_DEVICE",
    "MEDIA_ENGINE_DEVICES",
    "RTC_CONNECTION_STATE",
    "VOICE_STATE_UPDATES",
    "MEDIA_ENGINE_SET_GO_LIVE_SOURCE",
    "MEDIA_ENGINE_SOUNDSHARE_FAILED",
    "MEDIA_ENGINE_NOISE_CANCELLATION_ERROR",
    "MEDIA_ENGINE_VOICE_ACTIVITY_DETECTION_ERROR",
    "MEDIA_ENGINE_VIDEO_FILTER_ERROR",
    "MEDIA_ENGINE_VIDEO_STATE_CHANGED",
    "NATIVE_SCREEN_SHARE_PICKER_UPDATE",
    "NATIVE_SCREEN_SHARE_PICKER_ERROR",
    "MEDIA_SESSION_JOINED",
    "RTC_CONNECTION_UPDATE_ID",
    "RTC_CONNECTION_REMOTE_VIDEO_SINK_WANTS",
    "REPORT_AV_ERROR",
    "VIDEO_STREAM_READY_TIMEOUT",
    "CLEAR_VIDEO_STREAM_READY_TIMEOUT",
    "RTC_CONNECTION_VIDEO",
    "STREAM_CLOSE",
  ],
  inlineRequire() {
    return require("AVErrorManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.ApplicationStreamingManager = {
  actions: [
    "STREAM_WATCH",
    "STREAM_START",
    "STREAM_CREATE",
    "STREAM_UPDATE",
    "STREAM_DELETE",
    "STREAM_CLOSE",
    "CALL_UPDATE",
    "CHANNEL_UPDATES",
    "VOICE_CHANNEL_SELECT",
    "VOICE_STATE_UPDATES",
    "CONNECTION_CLOSED",
    "LOGOUT",
  ],
  inlineRequire() {
    return require("ApplicationStreamingManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.RequestToStreamManager = {
  actions: ["VOICE_STATE_UPDATES"],
  inlineRequire() {
    return require("RequestToStreamManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.LibdiscoreExperimentManager = {
  actions: [],
  inlineRequire() {
    return require("LibdiscoreExperimentManager").default;
  },
  hasStoreChangeListeners: true,
};
obj.NativeIntentsManager = {
  actions: [
    "POST_CONNECTION_OPEN",
    "CHANNEL_SELECT",
    "CHANNEL_CREATE",
    "CHANNEL_DELETE",
    "CHANNEL_UPDATES",
    "GUILD_CREATE",
    "GUILD_UPDATE",
    "GUILD_DELETE",
    "LOGOUT",
    "RELATIONSHIP_ADD",
    "RELATIONSHIP_REMOVE",
    "RELATIONSHIP_UPDATE",
    "THREAD_CREATE",
    "THREAD_DELETE",
    "THREAD_UPDATE",
    "USER_UPDATE",
  ],
  inlineRequire() {
    return require("NativeIntentsManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.NavigationTTIDispatcherManager = {
  actions: ["CHANNEL_SELECT"],
  inlineRequire() {
    return require("NavigationTTIDispatcherManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.SafetyFlowsManager = {
  actions: ["CONNECTION_OPEN_SUPPLEMENTAL", "SAFETY_FLOWS_MODAL_OPEN", "USER_REQUIRED_ACTION_UPDATE"],
  inlineRequire() {
    return require("SafetyFlowsManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.UserApplicationIdentityManager = {
  actions: ["USER_APPLICATION_IDENTITY_UPDATE", "USER_APPLICATION_IDENTITY_REMOVE"],
  inlineRequire() {
    return require("UserApplicationIdentityManager").default;
  },
};
obj.LocalMessageCacheStatsManager = {
  actions: ["APP_STATE_UPDATE"],
  inlineRequire() {
    return require("LocalMessageCacheStatsManager").default;
  },
};
obj.RestrictedHoursManager = {
  actions: ["POST_CONNECTION_OPEN", "CURRENT_USER_UPDATE", "LOGOUT", "NOTIFICATIONS_SET_SCREEN_DOWNTIME_REMINDER"],
  inlineRequire() {
    return require("RestrictedHoursManager").default;
  },
  hasStoreChangeListeners: true,
  neverLoadBeforeConnectionOpen: true,
};
obj.SentryExperimentFeatureFlagManager = {
  actions: [],
  inlineRequire() {
    return require("SentryExperimentFeatureFlagManager").default;
  },
  hasStoreChangeListeners: true,
  loadRightBeforeConnectionOpen: true,
};
obj.PlayableProgressManager = {
  actions: [],
  inlineRequire() {
    return require("PlayableProgressManager").default;
  },
  hasStoreChangeListeners: true,
};
obj.QuestProgressManager = {
  actions: [
    "QUESTS_SEND_HEARTBEAT_SUCCESS",
    "QUESTS_SEND_HEARTBEAT_FAILURE",
    "QUESTS_ENROLL_SUCCESS",
    "RUNNING_GAMES_CHANGE",
    "RUNNING_NON_GAMES_CHANGE",
    "LOCAL_ACTIVITY_UPDATE",
    "RPC_APP_DISCONNECTED",
    "QUESTS_FETCH_CURRENT_QUESTS_SUCCESS",
    "QUESTS_FETCH_QUEST_TO_DELIVER_SUCCESS",
    "STREAM_START",
    "STREAM_CREATE",
    "STREAM_CLOSE",
    "PASSIVE_UPDATE_V2",
    "VOICE_STATE_UPDATES",
    "EMBEDDED_ACTIVITY_LAUNCH_SUCCESS",
    "FRAME_LAUNCH",
    "FRAME_STOP",
    "EMBEDDED_ACTIVITY_UPDATE_V2",
    "QUESTS_PREVIEW_UPDATE_SUCCESS",
    "QUEST_APPLICATION_START_TIMER",
    "GAME_FETCH_SUCCESS",
    "APPLICATIONS_FETCH_SUCCESS",
  ],
  inlineRequire() {
    return require("QuestProgressManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
obj.AccountLinkManagerNative = {
  actions: [
    "ACCOUNT_LINK_AUTHORIZATION_COMPLETED",
    "OAUTH2_TOKEN_CREATE",
    "USER_AUTHORIZED_APPS_UPDATE",
    "APP_STATE_UPDATE",
  ],
  inlineRequire() {
    return require("AccountLinkManagerNative").default;
  },
  loadRightBeforeConnectionOpen: true,
};
obj.AccountLinkingCampaignManager = {
  actions: ["LOGOUT", "POST_CONNECTION_OPEN"],
  inlineRequire() {
    return require("AccountLinkingCampaignManager").default;
  },
  neverLoadBeforeConnectionOpen: true,
};
ManagerRegistryShared.initialize(obj);
const result = size.fileFinishedImporting("modules/app_startup/ManagerRegistry.native.tsx");

export default obj;
