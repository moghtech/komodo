import { useRead, useWrite } from "@lib/hooks";
import { RequiredResourceComponents } from "@types";
import { DeleteResource } from "../common";
import { Bot, Cloud, ServerCog } from "lucide-react";
import { ServerTemplateConfig } from "./config";
import { Link, useNavigate } from "react-router-dom";
import { Card, CardDescription, CardHeader, CardTitle } from "@ui/card";
import { useState } from "react";
import { Types } from "komodo_client";
import { NewLayout } from "@components/layouts";
import { Input } from "@ui/input";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { ServerTemplateTable } from "./table";
import { LaunchServer } from "./actions";
import { ResourcePageHeader } from "@components/util";
import { GroupActions } from "@components/group-actions";

export const useServerTemplate = (id?: string) =>
  useRead("ListServerTemplates", {}).data?.find((d) => d.id === id);

export const ServerTemplateComponents: RequiredResourceComponents = {
  list_item: (id) => useServerTemplate(id),
  resource_links: () => undefined,

  Description: () => <>Deploy more cloud-based servers.</>,

  Dashboard: () => {
    const count = useRead("ListServerTemplates", {}).data?.length;
    return (
      <Link to="/server-templates/" className="w-full">
        <Card className="hover:bg-accent/50 transition-colors cursor-pointer">
          <CardHeader>
            <div className="flex justify-between">
              <div>
                <CardTitle>Server Templates</CardTitle>
                <CardDescription>{count} Total</CardDescription>
              </div>
              <ServerCog className="w-4 h-4" />
            </div>
          </CardHeader>
        </Card>
      </Link>
    );
  },

  New: () => {
    const nav = useNavigate();
    const { mutateAsync } = useWrite("CreateServerTemplate");
    const [name, setName] = useState("");
    const [type, setType] = useState<Types.ServerTemplateConfig["type"]>("Aws");

    return (
      <NewLayout
        entityType="Server Template"
        onConfirm={async () => {
          if (!type) return;
          const id = (await mutateAsync({ name, config: { type, params: {} } }))
            ._id?.$oid!;
          nav(`/server-templates/${id}`);
        }}
        enabled={!!name && !!type}
      >
        <div className="grid md:grid-cols-2 items-center">
          Name
          <Input
            placeholder="server-template-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>
        <div className="grid md:grid-cols-2 items-center">
          Cloud Provider
          <Select
            value={type}
            onValueChange={(value) => setType(value as typeof type)}
          >
            <SelectTrigger>
              <SelectValue placeholder="Select Type" />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                <SelectItem value="Aws">Aws</SelectItem>
                <SelectItem value="Hetzner">Hetzner</SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
        </div>
      </NewLayout>
    );
  },

  GroupActions: () => <GroupActions type="ServerTemplate" actions={[]} />,

  Table: ({ resources }) => (
    <ServerTemplateTable
      serverTemplates={resources as Types.ServerTemplateListItem[]}
    />
  ),

  Icon: () => <ServerCog className="w-4 h-4" />,
  BigIcon: () => <ServerCog className="w-8 h-8" />,

  State: () => null,

  Status: {},

  Info: {
    Provider: ({ id }) => {
      const provider = useServerTemplate(id)?.info.provider;
      return (
        <div className="flex items-center gap-2">
          <Cloud className="w-4 h-4" />
          {provider}
        </div>
      );
    },
    InstanceType: ({ id }) => {
      const instanceType = useServerTemplate(id)?.info.instance_type;
      return (
        <div className="flex items-center gap-2">
          <Bot className="w-4 h-4" />
          {instanceType}
        </div>
      );
    },
  },

  Actions: {
    LaunchServer,
  },

  Page: {},

  Config: ServerTemplateConfig,

  DangerZone: ({ id }) => <DeleteResource type="ServerTemplate" id={id} />,

  ResourcePageHeader: ({ id }) => {
    const template = useServerTemplate(id);
    return (
      <ResourcePageHeader
        intent="None"
        icon={<ServerCog className="w-8" />}
        type="ServerTemplate"
        id={id}
        name={template?.name}
        state={undefined}
        status={undefined}
      />
    );
  },
};
