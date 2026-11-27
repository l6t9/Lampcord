declare const state: {
  channels: {
    get(id: Types.ChannelId): Types.ChannelView | null;
    inGuild(guildId: Types.GuildId): Types.ChannelView[];
  };
  format: {
    ago(unixMs: number): string;
  };
  messages: {
    inChannel(channelId: Types.ChannelId): Types.MessageView[];
    latestIn(channelId: Types.ChannelId): Types.MessageView | null;
    rows(channelId: Types.ChannelId): Types.RowView[];
    scroll(channelId: Types.ChannelId): Types.ScrollPositionView;
  };
  permissions: {
    can(channelId: Types.ChannelId, permission: "createInstantInvite" | "kickMembers" | "banMembers" | "administrator" | "manageChannels" | "manageGuild" | "addReactions" | "viewAuditLog" | "prioritySpeaker" | "stream" | "viewChannel" | "sendMessages" | "sendTtsMessages" | "manageMessages" | "embedLinks" | "attachFiles" | "readMessageHistory" | "mentionEveryone" | "useExternalEmojis" | "viewGuildInsights" | "connect" | "speak" | "muteMembers" | "deafenMembers" | "moveMembers" | "useVad" | "changeNickname" | "manageNicknames" | "manageRoles" | "manageWebhooks" | "manageGuildExpressions" | "useApplicationCommands" | "requestToSpeak" | "manageEvents" | "manageThreads" | "createPublicThreads" | "createPrivateThreads" | "useExternalStickers" | "sendMessagesInThreads" | "useEmbeddedActivities" | "moderateMembers" | "viewCreatorMonetizationAnalytics" | "useSoundboard" | "createGuildExpressions" | "createEvents" | "useExternalSounds" | "sendVoiceMessages" | "setVoiceChannelStatus" | "sendPolls" | "useExternalApps" | "pinMessages" | "bypassSlowmode"): boolean;
  };
  readStates: {
    lastMessageId(channelId: Types.ChannelId): Types.MessageId | null;
  };
  threads: {
    archivedActivity(channelId: Types.ChannelId): number;
    browserAccess(channelId: Types.ChannelId): Types.BrowserAccess | null;
    browserView(channelId: Types.ChannelId): Types.BrowserView | null;
    lastActivity(channelId: Types.ChannelId): number;
    scrollOffset(): number;
    searchQuery(): string;
  };
  users: {
    get(id: Types.UserId): Types.UserView | null;
  };
};
