import { useState } from "react";
import { Button, Chevron, ScrollView, Stack, Text, ThreadIcon } from "@requiem/ui";
import { ThreadRow } from "./ThreadRow";
import type { Thread } from "@requiem/types";
import cls from "./ThreadList.module.css";

interface ThreadListProps {
  threads: Thread[];
}

export function ThreadList({ threads }: ThreadListProps) {
  const [expanded, setExpanded] = useState(true);
  const title = `Threads (${threads.length})`;
  return (
    <div className={cls.browser}>
      <Button variant="bare" className={cls.header} onClick={() => setExpanded(!expanded)}>
        <Stack direction="horizontal" align="center">
          <ThreadIcon width={16} height={16} color="text-secondary" />
          <Text variant="heading-md/semibold" color="text-strong">
            {title}
          </Text>
          <Chevron open={expanded} color="text-secondary" />
        </Stack>
      </Button>
      <Button variant="primary" size="small" onClick={() => actions.createThread()}>
        New thread
      </Button>
      <Button variant="secondary" size="small" onClick={() => setExpanded(!expanded)}>
        {expanded ? "Hide" : "Show"}
      </Button>
      {expanded && (
        <ScrollView>
          <div className={cls.rows}>
            {threads.map((t) => (
              <ThreadRow
                key={t.id}
                thread={t}
                onSelect={(id, shiftKey) => actions.openThread(id, shiftKey)}
              />
            ))}
          </div>
        </ScrollView>
      )}
      {threads.length === 0 ? (
        <Stack align="center" padding={32}>
          <Text variant="heading-xl/semibold" color="text-strong" center>
            No threads yet
          </Text>
        </Stack>
      ) : null}
    </div>
  );
}
