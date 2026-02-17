import { useRead } from "@/lib/hooks";
import { ReactNode } from "react";
import { useSwarmDockerSearch } from ".";
import SwarmTasksSection from "@/components/swarm/tasks-section";

export default function SwarmTasks({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const _search = useSwarmDockerSearch();
  const tasks =
    useRead("ListSwarmTasks", { swarm: id }, { refetchInterval: 10_000 })
      .data ?? [];

  return (
    <SwarmTasksSection
      id={id}
      tasks={tasks}
      titleOther={titleOther}
      _search={_search}
    />
  );
}
