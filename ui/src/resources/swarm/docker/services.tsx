import { useRead } from "@/lib/hooks";
import { ReactNode } from "react";
import { useSwarmDockerSearch } from ".";
import SwarmServicesSection from "@/components/swarm/services-section";

export default function SwarmServices({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) {
  const _search = useSwarmDockerSearch();
  const services =
    useRead("ListSwarmServices", { swarm: id }, { refetchInterval: 10_000 })
      .data ?? [];

  return (
    <SwarmServicesSection
      id={id}
      services={services}
      titleOther={titleOther}
      _search={_search}
    />
  );
}
