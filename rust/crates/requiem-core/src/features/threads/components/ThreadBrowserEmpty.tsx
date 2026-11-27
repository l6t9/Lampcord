import { Button, Stack, Text } from "@requiem/ui";
import { AssetImage } from "@requiem/app";
import type { ChannelId } from "@requiem/types";
import cls from "./thread_browser.module.css";

interface ThreadBrowserEmptyProps {
  channelId: ChannelId;
  canCreate: boolean;
}

export function ThreadBrowserEmpty({ channelId, canCreate }: ThreadBrowserEmptyProps) {
  return (
    <Stack gap={0} align="center" className={cls.empty}>
      <AssetImage name="image/threads_empty" width={256} height={144} className={cls.emptyImage} />
      <Text variant="heading-xl/semibold" color="text-strong" center={true} className={cls.emptyTitle}>
        {t("threads-browser-empty-title")}
      </Text>
      <Text variant="text-md/normal" color="text-default" center={true}>
        {t("threads-browser-empty-body")}
      </Text>
      {canCreate && (
        <div className={cls.emptyCta}>
          <Button variant="primary" onClick={() => actions.startThread(channelId)}>
            {t("threads-create")}
          </Button>
        </div>
      )}
    </Stack>
  );
}
