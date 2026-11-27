import { Button, Stack, Text, ThreadIcon, XSmallIcon } from "@requiem/ui";
import { SearchBar } from "@requiem/app";
import type { ChannelId } from "@requiem/types";
import cls from "./thread_browser.module.css";

interface ThreadBrowserHeaderProps {
  channelId: ChannelId;
  modal: boolean;
}

export function ThreadBrowserHeader({ channelId, modal }: ThreadBrowserHeaderProps) {
  const access = state.threads.browserAccess(channelId);
  return (
    <Stack direction="horizontal" gap={0} align="center" className={cls.header}>
      <ThreadIcon color="text-secondary" className={cls.headerIcon} />
      <Text variant="heading-md/semibold" color="text-strong" className={cls.title}>
        {t("threads-browser-title")}
      </Text>
      {access && access.canRead && (
        <>
          <div className={cls.divider} />
          <div className={cls.search}>
            <SearchBar
              placeholder={t("threads-browser-search")}
              text={state.threads.searchQuery()}
              autofocus={true}
              clearable={true}
              onChange={(query) => actions.threadSearchChanged(query)}
              onSubmit={(query) => actions.submitThreadSearch(channelId, query)}
            />
          </div>
        </>
      )}
      <div className={cls.spacer} />
      {access && access.canCreate && (
        <div className={cls.create}>
          <Button variant="primary" size="small" onClick={() => actions.startThread(channelId)}>
            {t("threads-browser-create")}
          </Button>
        </div>
      )}
      {modal && (
        <Button variant="bare" className={cls.close} onClick={() => actions.closeThreadBrowser()}>
          <XSmallIcon color="text-secondary" className={cls.closeIcon} />
        </Button>
      )}
    </Stack>
  );
}
