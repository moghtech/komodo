import { useRead } from "@lib/hooks";
import { Dispatch, ReactNode, SetStateAction } from "react";
import { SwarmTasksTable } from "../table";

export const SwarmTasks = ({
  id,
  titleOther,
  _search,
}: {
  id: string;
  titleOther: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const tasks =
    useRead("ListSwarmTasks", { swarm: id }, { refetchInterval: 10_000 })
      .data ?? [];

  return (
    <SwarmTasksTable
      id={id}
      tasks={tasks}
      titleOther={titleOther}
      _search={_search}
    />
  );
};
