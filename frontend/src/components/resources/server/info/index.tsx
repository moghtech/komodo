import { Section } from "@components/layouts";
import { ReactNode, useState } from "react";
import { Networks } from "./networks";
import { useServer } from "..";
import { Types } from "komodo_client";
import { useLocalStorage } from "@lib/hooks";
import { Images } from "./images";
import { Containers } from "./containers";
import { Volumes } from "./volumes";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@ui/tabs";

export const ServerInfo = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const _search = useState("");
  const state = useServer(id)?.info.state ?? Types.ServerState.NotOk;
  const [show2, setShow2] = useLocalStorage<
    "Containers" | "Networks" | "Volumes" | "Images"
  >("server-info-show-config-v2", "Containers");

  if ([Types.ServerState.NotOk, Types.ServerState.Disabled].includes(state)) {
    return (
      <Section titleOther={titleOther}>
        <h2 className="text-muted-foreground">
          Server unreachable, info is not available
        </h2>
      </Section>
    );
  }

  const tabsList = (
    <TabsList className="justify-start w-fit">
      <TabsTrigger value="Containers" className="w-[110px]">
        Containers
      </TabsTrigger>
      <TabsTrigger value="Networks" className="w-[110px]">
        Networks
      </TabsTrigger>
      <TabsTrigger value="Volumes" className="w-[110px]">
        Volumes
      </TabsTrigger>
      <TabsTrigger value="Images" className="w-[110px]">
        Images
      </TabsTrigger>
    </TabsList>
  );

  return (
    <Section titleOther={titleOther}>
      <Tabs value={show2} onValueChange={setShow2 as any}>
        <TabsContent value="Containers">
          <Containers id={id} titleOther={tabsList} _search={_search} />
        </TabsContent>
        <TabsContent value="Networks">
          <Networks id={id} titleOther={tabsList} _search={_search} />
        </TabsContent>
        <TabsContent value="Volumes">
          <Volumes id={id} titleOther={tabsList} _search={_search} />
        </TabsContent>
        <TabsContent value="Images">
          <Images id={id} titleOther={tabsList} _search={_search} />
        </TabsContent>
      </Tabs>
    </Section>
  );
};
