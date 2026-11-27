import { ScrollView } from "@requiem/ui";
import type { ChannelId } from "@requiem/types";
import { MessageEntry } from "./MessageEntry";
import cls from "./message_list.module.css";

interface MessageListProps {
  channelId: ChannelId;
}

export function MessageList({ channelId }: MessageListProps) {
  const rows = state.messages.rows(channelId);
  const position = state.messages.scroll(channelId);
  return (
    <ScrollView
      className={cls.messageList}
      itemHeight="height"
      position={position}
      onPosition={(next) =>
        actions.messageListScrolled(channelId, next.pinned, next.key, next.offset, next.height)
      }
      onMeasure={(measured) => actions.messageRowsMeasured(measured.keys, measured.heights)}
    >
      {rows.map((row) => (
        <MessageEntry key={row.key} row={row} channelId={channelId} />
      ))}
    </ScrollView>
  );
}
