import ContainersSection from "@/components/docker/containers-section";
import { useRead } from "@/lib/hooks";
import { ReactNode } from "react";
import { useServerDockerSearch } from ".";

export default function ServerContainers({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const _search = useServerDockerSearch();
  const containers =
    useRead("ListDockerContainers", { server: id }, { refetchInterval: 10_000 })
      .data ?? [];
  return (
    <ContainersSection
      serverId={id}
      containers={containers}
      titleOther={titleOther}
      _search={_search}
      pruneButton
      forceTall
    />
  );
}
