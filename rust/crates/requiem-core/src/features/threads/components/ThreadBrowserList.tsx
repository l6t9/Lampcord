import { ScrollView } from "@requiem/ui";
import type { ChannelId } from "@requiem/types";
import { ThreadBrowserEmpty } from "./ThreadBrowserEmpty";
import { ThreadBrowserEntry } from "./ThreadBrowserEntry";
import cls from "./thread_browser.module.css";

interface ThreadBrowserListProps {
  channelId: ChannelId;
}

export function ThreadBrowserList({ channelId }: ThreadBrowserListProps) {
  const view = state.threads.browserView(channelId);
  return (
    <div className={cls.listHost}>
      {view && view.phase === "loading" && (
        <div className={cls.list}>
          <div className={cls.loading} />
        </div>
      )}
      {view && view.phase === "empty" && <ThreadBrowserEmpty channelId={channelId} canCreate={view.canCreate} />}
      {view && view.phase === "list" && (
        <ScrollView
          className={cls.list}
          itemHeight="height"
          itemGap={8}
          initialOffset={state.threads.scrollOffset()}
          onScroll={(event) => actions.rememberThreadScroll(event.offset)}
          onNearEnd={() => actions.loadMoreThreads(channelId)}
        >
          {view.entries.map((entry) => (
            <ThreadBrowserEntry key={entry.key} entry={entry} />
          ))}
        </ScrollView>
      )}
    </div>
  );
}
