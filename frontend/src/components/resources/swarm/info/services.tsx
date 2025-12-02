import { useRead } from "@lib/hooks";
import { Dispatch, ReactNode, SetStateAction } from "react";
import { SwarmServicesTable } from "../table";

export const SwarmServices = ({
  id,
  titleOther,
  _search,
}: {
  id: string;
  titleOther: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const services =
    useRead("ListSwarmServices", { swarm: id }, { refetchInterval: 10_000 })
      .data ?? [];

  return (
    <SwarmServicesTable
      id={id}
      services={services}
      titleOther={titleOther}
      _search={_search}
    />
  );
};
