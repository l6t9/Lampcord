import { Text } from "@requiem/ui";
import type { BrowserEntry } from "@requiem/types";
import { ThreadBrowserRow } from "./ThreadBrowserRow";
import cls from "./thread_browser.module.css";

interface ThreadBrowserEntryProps {
  entry: BrowserEntry;
}

export function ThreadBrowserEntry({ entry }: ThreadBrowserEntryProps) {
  const joined = t("threads-browser-joined", { count: entry.count });
  const other = t("threads-browser-other", { count: entry.count });
  const older = t("threads-browser-older");
  const title = entry.section === "joined" ? joined : entry.section === "other" ? other : older;
  return (
    <div className={cls.entry} style={`height: ${entry.height}px`}>
      {entry.kind === "header" && (
        <Text variant="text-xs/bold" color="text-default" className={cls.sectionHeader}>
          {title.toUpperCase()}
        </Text>
      )}
      {entry.thread && <ThreadBrowserRow thread={entry.thread} />}
    </div>
  );
}
