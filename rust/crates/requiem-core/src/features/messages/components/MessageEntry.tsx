import { Stack } from "@requiem/ui";
import {
  MessageDayDivider,
  MessageHistoryHeader,
  MessagePendingBottom,
  MessagePendingNotices,
  MessageRowHost,
} from "@requiem/app";
import type { ChannelId, RowView } from "@requiem/types";

interface MessageEntryProps {
  row: RowView;
  channelId: ChannelId;
}

export function MessageEntry({ row, channelId }: MessageEntryProps) {
  return (
    <Stack gap={0}>
      {row.kind === "history" && <MessageHistoryHeader channelId={channelId} loading={row.loading} />}
      {row.kind === "message" && (row.day !== null || row.unread) && (
        <MessageDayDivider day={row.day} unread={row.unread} startsGroup={row.startsGroup} />
      )}
      {row.kind === "message" && (
        <MessageRowHost channelId={channelId} messageId={row.key} startsGroup={row.startsGroup} />
      )}
      {row.kind === "message" && <MessagePendingNotices channelId={channelId} messageId={row.key} />}
      {row.kind === "tail" && <MessagePendingBottom channelId={channelId} />}
      {row.kind === "tail" && <div style={`height: ${row.height}px`} />}
    </Stack>
  );
}
