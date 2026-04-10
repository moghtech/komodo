import { useRead } from "@/lib/hooks";
import { Types } from "komodo_client";
import { ReactNode, useState } from "react";
import { useStack } from "..";
import ContainerStats from "@/components/docker/container-stats";
import Section from "@/ui/section";
import { Group } from "@mantine/core";
import SearchInput from "@/ui/search-input";

export default function StackContainerStats({ id, titleOther }: { id: string, titleOther: ReactNode }) {
  const [search, setSearch] = useState("");
  const isStackAvailable = useStack(id)?.info.state !== Types.StackState.Down;
  const containers = useRead(
    "ListStackServices",
    {
        stack: id,
    },
    {
        enabled: isStackAvailable,
    },
  ).data?.map((c) => c.container).filter(c => c !== undefined);
  return (
      <Section titleOther={titleOther}>
        <Section title="Container Stats" titleRight={
                  <Group ml={{ sm: "xl" }}>
                    <SearchInput
                      value={search}
                      onSearch={setSearch}
                      w={{ base: 200, lg: 300 }}
                    />
                  </Group>
                }>  
          <ContainerStats search={search} containers={containers || []} />
        </Section>
    </Section>
  )
}
