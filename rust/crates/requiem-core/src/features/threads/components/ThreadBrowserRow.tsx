import { Button, Stack, Text } from "@requiem/ui";
import type { ChannelView } from "@requiem/types";
import { ActiveThreadSubtext, ArchiveTimeSubtext } from "./ThreadBrowserRowSubtext";
import cls from "./thread_browser.module.css";

interface ThreadBrowserRowProps {
  thread: ChannelView;
}

export function ThreadBrowserRow({ thread }: ThreadBrowserRowProps) {
  return (
    <Button
      variant="bare"
      className={cls.row}
      onClick={(event) => actions.goToThread(thread.id, event.shiftKey)}
      onContextMenu={(event) =>
        actions.openChannelMenu(thread.id, thread.guildId, event.x, event.y)
      }
    >
      <Stack gap={0} className={cls.rowLeft}>
        <Text variant="heading-md/semibold" lineClamp={1}>
          {thread.name ?? ""}
        </Text>
        {thread.archived ? (
          <ArchiveTimeSubtext thread={thread} />
        ) : (
          <ActiveThreadSubtext thread={thread} />
        )}
      </Stack>
    </Button>
  );
}
