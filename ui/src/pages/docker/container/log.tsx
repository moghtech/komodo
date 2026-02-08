import LogViewer from "@/ui/log-viewer";
import { useRead } from "@/lib/hooks";
import Section from "@/ui/section";
import { ReactNode } from "react";

export interface ContainerLogProps {
  server: string;
  container: string;
  titleOther: ReactNode;
  tail?: number;
  timestamps?: boolean;
  poll?: boolean;
}

export default function ContainerLog({
  server,
  container,
  titleOther,
  tail = 50,
  timestamps,
  poll,
}: ContainerLogProps) {
  const { data: log, refetch } = useRead(
    "GetContainerLog",
    {
      server,
      container,
      tail,
      timestamps,
    },
    { refetchInterval: poll ? 3000 : false },
  );
  return (
    <Section titleOther={titleOther}>
      <LogViewer log={log?.stdout ?? "NO LOG"} />
    </Section>
  );
}
