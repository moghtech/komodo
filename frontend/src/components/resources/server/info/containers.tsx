import { DockerContainersSection } from "@components/util";
import { useRead } from "@lib/hooks";
import { Dispatch, ReactNode, SetStateAction } from "react";

export const Containers = ({
  id,
  titleOther,
  _search,
}: {
  id: string;
  titleOther: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const containers =
    useRead("ListDockerContainers", { server: id }, { refetchInterval: 10_000 })
      .data ?? [];
  return (
    <DockerContainersSection
      server_id={id}
      containers={containers}
      titleOther={titleOther}
      _search={_search}
      pruneButton
      forceTall
    />
  );
};
