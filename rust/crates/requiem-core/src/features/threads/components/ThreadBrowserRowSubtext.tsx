import { Stack, Text, ThreadIcon } from "@requiem/ui";
import { Avatar, AuthorName } from "@requiem/app";
import type { ChannelView } from "@requiem/types";
import cls from "./thread_browser.module.css";

interface ThreadBrowserRowSubtextProps {
  thread: ChannelView;
}

export function ActiveThreadSubtext({ thread }: ThreadBrowserRowSubtextProps) {
  const message = state.messages.latestIn(thread.id);
  const time = `  •  ${state.format.ago(state.threads.lastActivity(thread.id))}`;
  return (
    <Stack direction="horizontal" gap={0} align="center" className={cls.subtext}>
      {message ? (
        <>
          <Avatar userId={message.authorId} className={cls.avatar} />
          <AuthorName userId={message.authorId} guildId={thread.guildId} className={cls.author} />
          <Text variant="text-sm/normal" lineClamp={1} className={cls.message}>
            {": "}
            {message.contentPreview}
          </Text>
          <Text variant="text-sm/normal" lineClamp={1} className={cls.time}>
            {time}
          </Text>
        </>
      ) : (
        <Text variant="text-sm/normal" lineClamp={1}>
          {t("threads-browser-no-recent")}
          {time}
        </Text>
      )}
    </Stack>
  );
}

export function ArchiveTimeSubtext({ thread }: ThreadBrowserRowSubtextProps) {
  const owner = thread.ownerId ? state.users.get(thread.ownerId) : null;
  const lastActive = t("threads-browser-last-active", {
    time: state.format.ago(state.threads.archivedActivity(thread.id)),
  });
  return (
    <Stack direction="horizontal" gap={0} align="center" className={cls.subtext}>
      {owner ? (
        <>
          <Avatar userId={owner.id} className={cls.avatar} />
          <Text variant="text-sm/normal" lineClamp={1}>
            {t("threads-browser-started-by")}
          </Text>
          <AuthorName
            userId={owner.id}
            guildId={thread.guildId}
            className={`${cls.author} ${cls.startedByName}`}
          />
          <div className={cls.bullet}>
            <Text variant="text-sm/normal" color="text-default">
              •
            </Text>
          </div>
          <Text variant="text-sm/normal" lineClamp={1}>
            {lastActive}
          </Text>
        </>
      ) : (
        <>
          <div className={cls.noAvatar}>
            <ThreadIcon width={10} height={10} color="text-default" />
          </div>
          <Text variant="text-sm/normal" lineClamp={1}>
            {lastActive}
          </Text>
        </>
      )}
    </Stack>
  );
}
