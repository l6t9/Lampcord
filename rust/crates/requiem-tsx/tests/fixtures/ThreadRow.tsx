import { Button, Text } from "@requiem/ui";
import type { Thread, ThreadId } from "@requiem/types";
import cls from "./ThreadRow.module.css";

interface ThreadRowProps {
  thread: Thread;
  onSelect: (id: ThreadId, shiftKey: boolean) => void;
}

export function ThreadRow({ thread, onSelect }: ThreadRowProps) {
  return (
    <Button
      variant="bare"
      className={cls.row}
      onClick={(event) => onSelect(thread.id, event.shiftKey)}
    >
      <div className={cls.rowText}>
        <Text variant="heading-md/semibold" lineClamp={1}>
          {thread.name}
        </Text>
        <Text variant="text-sm/normal" color="text-secondary" lineClamp={1}>
          {thread.subtext}
        </Text>
      </div>
    </Button>
  );
}
