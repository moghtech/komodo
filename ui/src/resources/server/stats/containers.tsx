import { useRead } from "@/lib/hooks";
import { useLocalStorage } from "@mantine/hooks";
import { useState } from "react";
import { useIsServerAvailable } from "../hooks";
import ContainerStats from "@/components/docker/container-stats";
import SearchInput from "@/ui/search-input";
import { ICONS } from "@/theme/icons";
import Section from "@/ui/section";
import ShowHideButton from "@/ui/show-hide-button";
import { Group } from "@mantine/core";

export default function ServerContainerStats({ id }: { id: string }) {
  const [search, setSearch] = useState("");
  const [show, setShow] = useLocalStorage({
    key: "server-stats-containers-show-v2",
    defaultValue: true,
  });
  const isServerAvailable = useIsServerAvailable(id);
  const containers = useRead(
    "ListDockerContainers",
    {
      server: id,
    },
    {
      enabled: isServerAvailable && show,
    },
  ).data?.filter((c) => c.stats);
  return (
        <Section
          withBorder
          title="Containers"
          icon={<ICONS.Container size="1.3rem" />}
          titleRight={
            <Group ml={{ sm: "xl" }}>
              <SearchInput
                value={search}
                onSearch={setSearch}
                w={{ base: 200, lg: 300 }}
              />
              <ShowHideButton show={show} setShow={setShow} />
            </Group>
          }
          onHeaderClick={() => setShow((s) => !s)}
        >
          <ContainerStats containers={containers || []} search={search} />
        </Section>
  )
}
