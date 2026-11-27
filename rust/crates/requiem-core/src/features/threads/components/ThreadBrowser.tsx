import type { ChannelId } from "@requiem/types";
import { ThreadBrowserHeader } from "./ThreadBrowserHeader";
import { ThreadBrowserList } from "./ThreadBrowserList";
import cls from "./thread_browser.module.css";

interface ThreadBrowserProps {
  channelId: ChannelId;
  modal: boolean;
}

export function ThreadBrowser({ channelId, modal }: ThreadBrowserProps) {
  return (
    <div className={cls.browser}>
      <ThreadBrowserHeader channelId={channelId} modal={modal} />
      <ThreadBrowserList channelId={channelId} />
    </div>
  );
}
