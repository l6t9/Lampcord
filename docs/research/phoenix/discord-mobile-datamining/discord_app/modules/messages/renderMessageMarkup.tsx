// discord_app/modules/messages/renderMessageMarkup.tsx
import MarkupUtilsDefault from "../markup/MarkupUtils.tsx";
import MarkupPostProcessors from "MarkupPostProcessors.tsx";
import size from "../../../_runtime/metro/00002__.js";

function getInitialParserState(channelId) {
  const renderOptions = channelId.renderOptions;
  return {
    channelId: channelId.channelId,
    messageId: channelId.messageId,
    authorId: channelId.authorId,
    allowLinks: Boolean(renderOptions.allowLinks),
    allowDevLinks: Boolean(renderOptions.allowDevLinks),
    allowGameMentions: Boolean(renderOptions.allowGameMentions),
    allowTimeMentionInput: Boolean(renderOptions.allowTimeMentionInput),
    formatInline: Boolean(renderOptions.formatInline),
    noStyleAndInteraction: Boolean(renderOptions.noStyleAndInteraction),
    allowHeading: Boolean(renderOptions.allowHeading),
    allowList: Boolean(renderOptions.allowList),
    previewLinkTarget: Boolean(renderOptions.previewLinkTarget),
    disableAnimatedEmoji: Boolean(renderOptions.disableAnimatedEmoji),
    allowEmojiLinks: false,
    disableAutoBlockNewlines: true,
    mentionChannels: [],
    soundboardSounds: [],
    muted: false,
    unknownUserMentionPlaceholder: true,
    viewingChannelId: renderOptions.viewingChannelId,
    forceWhite: Boolean(renderOptions.forceWhite),
    textColor: renderOptions.textColor,
    disablePressableChannelMention: Boolean(renderOptions.disablePressableChannelMention),
  };
}
function render(fn, channelId, toAST) {
  const message = channelId;
  toAST = toAST.toAST;
  toAST = undefined !== toAST && toAST;
  let hideSimpleEmbedContent = toAST.hideSimpleEmbedContent;
  hideSimpleEmbedContent = undefined === hideSimpleEmbedContent || hideSimpleEmbedContent;
  let formatInline = toAST.formatInline;
  formatInline = undefined !== formatInline && formatInline;
  ({ postProcessor: render, contentMessage } = toAST);
  hasSpoilerEmbeds = false;
  if (contentMessage == null) {
    contentMessage = channelId;
  }
  const content = contentMessage.content;
  const obj = { channelId: channelId.channel_id, messageId: channelId.id, authorId: null, renderOptions: null };
  const author = channelId.author;
  let id;
  if (author != null) {
    id = author.id;
  }
  obj.authorId = id;
  obj.renderOptions = toAST;
  const tmpResult = formatInline(obj);
  const obj3 = {};
  const merged = Object.assign(tmpResult);
  let allowLinks = tmp4;
  if (null == channelId.webhookId) {
    allowLinks = tmpResult.allowLinks;
  }
  obj3.allowLinks = allowLinks;
  obj3.allowEmojiLinks = null != channelId.webhookId;
  ({ mentionChannels: obj2.mentionChannels, soundboardSounds } = channelId);
  if (soundboardSounds == null) {
    soundboardSounds = [];
  }
  obj3.soundboardSounds = soundboardSounds;
  return {
    hasSpoilerEmbeds,
    hasBailedAst: false,
    content: fn(content, true, obj3, (ast, inline, arg2) => {
      flag = arg2;
      if (arg2 == null) {
        flag = false;
      }
      const result = MarkupPostProcessors.runMessageMarkupPostProcessors({
        ast,
        inline,
        hasBailedAst: flag,
        message,
        contentMessage,
        messageContent: content,
        hideSimpleEmbedContent,
        formatInline,
        toAST,
      });
      ({ ast, hasSpoilerEmbeds: c6 } = result);
      let tmp2 = ast;
      if (null != render) {
        tmp2 = render(ast, inline);
      }
      return tmp2;
    }),
  };
}
let result = size.fileFinishedImporting("modules/messages/renderMessageMarkup.tsx");

export default function renderMessageMarkup(arg0) {
  let obj = arg1;
  if (arg1 === undefined) {
    obj = {};
  }
  const tmp2 = MarkupUtilsDefault;
  return render(obj.formatInline ? tmp2.parseInlineReply : tmp2.parse, arg0, obj);
}
export function getInitialParserStateFromMessage(message, renderOptions) {
  const obj = { channelId: message.channel_id, messageId: message.id, authorId: null, renderOptions: null };
  const author = message.author;
  let id;
  if (author != null) {
    id = author.id;
  }
  obj.authorId = id;
  obj.renderOptions = renderOptions;
  const tmpResult = getInitialParserState(obj);
  const obj3 = {};
  const merged = Object.assign(tmpResult);
  let allowLinks = tmp4;
  if (null == message.webhookId) {
    allowLinks = tmpResult.allowLinks;
  }
  obj3.allowLinks = allowLinks;
  obj3.allowEmojiLinks = null != message.webhookId;
  ({ mentionChannels: obj2.mentionChannels, soundboardSounds } = message);
  if (soundboardSounds == null) {
    soundboardSounds = [];
  }
  obj3.soundboardSounds = soundboardSounds;
  return obj3;
}
export { getInitialParserState };
export function renderMessageMarkupWithParser(NativeSearchResultLinkPreviewParser, arg1, arg2) {
  let obj = arg2;
  if (arg2 === undefined) {
    obj = {};
  }
  return render(NativeSearchResultLinkPreviewParser, arg1, obj);
}
export function renderMessageMarkupToAST(message, arg1) {
  let obj = arg1;
  if (arg1 === undefined) {
    obj = {};
  }
  const tmp2 = MarkupUtilsDefault;
  const obj2 = {};
  const merged = Object.assign(obj);
  obj2.toAST = true;
  return render(obj.formatInline ? tmp2.parseInlineReplyToAST : tmp2.parseToAST, message, obj2);
}
export function renderMessageMarkupToASTWithParser(arg0, message, arg2) {
  let obj = arg2;
  if (arg2 === undefined) {
    obj = {};
  }
  const obj2 = {};
  const merged = Object.assign(obj);
  obj2.toAST = true;
  return render(arg0, message, obj2);
}
export function renderMessageContentMarkup(notifCenterV2MessagePreviewParser, guildId, arg2) {
  let obj = arg2;
  if (arg2 === undefined) {
    obj = {};
  }
  if (obj === undefined) {
    obj = {};
  }
  return notifCenterV2MessagePreviewParser(
    guildId.content,
    true,
    {
      allowLinks: false,
      allowDevLinks: false,
      allowEmojiLinks: false,
      allowGameMentions: false,
      mentionChannels: [],
      soundboardSounds: [],
      formatInline: true,
      noStyleAndInteraction: false,
      allowHeading: false,
      allowList: false,
      disableAutoBlockNewlines: true,
      previewLinkTarget: false,
      disableAnimatedEmoji: true,
      guildId: guildId.guildId,
      channelId: guildId.channelId,
      messageId: guildId.messageId,
      authorId: guildId.authorId,
      muted: false,
      unknownUserMentionPlaceholder: true,
      disablePressableChannelMention: true,
      textColor: obj.textColor,
    },
    (arg0) => {
      let tmp = arg0;
      if (!Array.isArray(arg0)) {
        items = [arg0];
        tmp = items;
      }
      return tmp;
    },
  );
}
export function renderAutomodMessageMarkup(arg0, highlightWord, channelId) {
  return MarkupUtilsDefault.parseAutoModerationSystemMessage(
    arg0,
    true,
    {
      allowLinks: false,
      allowDevLinks: false,
      allowEmojiLinks: false,
      allowGameMentions: false,
      mentionChannels: [],
      soundboardSounds: [],
      formatInline: false,
      noStyleAndInteraction: false,
      allowHeading: false,
      allowList: false,
      disableAutoBlockNewlines: true,
      highlightWord,
      disableAnimatedEmoji: false,
      channelId,
      muted: false,
    },
    (arg0) => {
      let tmp = arg0;
      if (!Array.isArray(arg0)) {
        items = [arg0];
        tmp = items;
      }
      return tmp;
    },
  );
}
export function renderAutomodMessageMarkupToAST(arg0, highlightWord, channelId) {
  return MarkupUtilsDefault.parseAutoModerationSystemMessageToAST(
    arg0,
    true,
    {
      allowLinks: false,
      allowDevLinks: false,
      allowEmojiLinks: false,
      allowGameMentions: false,
      mentionChannels: [],
      soundboardSounds: [],
      formatInline: false,
      noStyleAndInteraction: false,
      allowHeading: false,
      allowList: false,
      disableAutoBlockNewlines: true,
      highlightWord,
      disableAnimatedEmoji: false,
      channelId,
      muted: false,
    },
    (arg0) => {
      let tmp = arg0;
      if (!Array.isArray(arg0)) {
        items = [arg0];
        tmp = items;
      }
      return tmp;
    },
  );
}
