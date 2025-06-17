import { NewLayout } from "@components/layouts";
import { useRead, useUser, useWrite } from "@lib/hooks";
import { Types } from "komodo_client";
import { RequiredResourceComponents } from "@types";
import { Card, CardDescription, CardHeader, CardTitle } from "@ui/card";
import { Input } from "@ui/input";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { Cloud, Bot, Factory } from "lucide-react";
import { ReactNode, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { BuilderConfig } from "./config";
import { DeleteResource, ResourceLink, ResourcePageHeader } from "../common";
import { BuilderTable } from "./table";
import { GroupActions } from "@components/group-actions";
import { useServer } from "../server";
import { cn } from "@lib/utils";
import {
  ColorIntention,
  server_state_intention,
  stroke_color_class_by_intention,
} from "@lib/color";

export const useBuilder = (id?: string) =>
  useRead("ListBuilders", {}, { refetchInterval: 10_000 }).data?.find(
    (d) => d.id === id
  );

const Icon = ({ id, size }: { id?: string; size: number }) => {
  const info = useBuilder(id)?.info;
  if (info?.builder_type === "Server" && info.instance_type) {
    return <ServerIcon server_id={info.instance_type} size={size} />;
  } else {
    return <Factory className={`w-${size} h-${size}`} />;
  }
};

const ServerIcon = ({
  server_id,
  size,
}: {
  server_id: string;
  size: number;
}) => {
  const state = useServer(server_id)?.info.state;
  return (
    <Factory
      className={cn(
        `w-${size} h-${size}`,
        state && stroke_color_class_by_intention(server_state_intention(state))
      )}
    />
  );
};

export const BuilderInstanceType = ({ id }: { id: string }) => {
  let info = useBuilder(id)?.info;
  if (info?.builder_type === "Server") {
    return (
      info.instance_type && (
        <ResourceLink type="Server" id={info.instance_type} />
      )
    );
  } else {
    return (
      <div className="flex items-center gap-2">
        <Bot className="w-4 h-4" />
        {info?.instance_type}
      </div>
    );
  }
};

export const BuilderComponents: RequiredResourceComponents = {
  list_item: (id) => useBuilder(id),
  resource_links: () => undefined,

  Description: () => <>Build on your servers, or single-use AWS instances.</>,

  Dashboard: () => {
    const builders_count = useRead("ListBuilders", {}).data?.length;
    return (
      <Link to="/builders/" className="w-full">
        <Card className="hover:bg-accent/50 transition-colors cursor-pointer">
          <CardHeader>
            <div className="flex justify-between">
              <div>
                <CardTitle>Builders</CardTitle>
                <CardDescription>{builders_count} Total</CardDescription>
              </div>
              <Factory className="w-4 h-4" />
            </div>
          </CardHeader>
        </Card>
      </Link>
    );
  },

  New: () => {
    const is_admin = useUser().data?.admin;
    const nav = useNavigate();
    const { mutateAsync } = useWrite("CreateBuilder");
    const [name, setName] = useState("");
    const [type, setType] = useState<Types.BuilderConfig["type"]>();

    if (!is_admin) return null;

    return (
      <NewLayout
        entityType="Builder"
        onConfirm={async () => {
          if (!type) return;
          const id = (await mutateAsync({ name, config: { type, params: {} } }))
            ._id?.$oid!;
          nav(`/builders/${id}`);
        }}
        enabled={!!name && !!type}
      >
        <div className="grid md:grid-cols-2 items-center">
          Name
          <Input
            placeholder="builder-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </div>
        <div className="grid md:grid-cols-2 items-center">
          Builder Type
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
                <SelectItem value="Server">Server</SelectItem>
                <SelectItem value="Url">Url</SelectItem>
              </SelectGroup>
            </SelectContent>
          </Select>
        </div>
      </NewLayout>
    );
  },

  GroupActions: () => <GroupActions type="Builder" actions={[]} />,

  Table: ({ resources }) => (
    <BuilderTable builders={resources as Types.BuilderListItem[]} />
  ),

  Icon: ({ id }) => <Icon id={id} size={4} />,
  BigIcon: ({ id }) => <Icon id={id} size={8} />,

  State: () => null,
  Status: {},

  Info: {
    Provider: ({ id }) => {
      const builder_type = useBuilder(id)?.info.builder_type;
      return (
        <div className="flex items-center gap-2">
          <Cloud className="w-4 h-4" />
          {builder_type}
        </div>
      );
    },
    InstanceType: ({ id }) => <BuilderInstanceType id={id} />,
  },

  Actions: {},

  Page: {},

  Config: BuilderConfig,

  DangerZone: ({ id }) => <DeleteResource type="Builder" id={id} />,

  ResourcePageHeader: ({ id }) => {
    const builder = useBuilder(id);
    if (builder?.info.builder_type === "Server" && builder.info.instance_type) {
      return (
        <ServerInnerResourcePageHeader
          builder={builder}
          server_id={builder.info.instance_type}
        />
      );
    }
    return (
      <InnerResourcePageHeader
        id={id}
        builder={builder}
        intent="None"
        icon={<Factory className="w-8 h-8" />}
      />
    );
  },
};

const ServerInnerResourcePageHeader = ({
  builder,
  server_id,
}: {
  builder: Types.BuilderListItem;
  server_id: string;
}) => {
  const state = useServer(server_id)?.info.state;
  return (
    <InnerResourcePageHeader
      id={builder.id}
      builder={builder}
      intent={server_state_intention(state)}
      icon={<ServerIcon server_id={server_id} size={8} />}
    />
  );
};

const InnerResourcePageHeader = ({
  id,
  builder,
  intent,
  icon,
}: {
  id: string;
  builder: Types.BuilderListItem | undefined;
  intent: ColorIntention;
  icon: ReactNode;
}) => {
  return (
    <ResourcePageHeader
      intent={intent}
      icon={icon}
      type="Builder"
      id={id}
      resource={builder}
      state={builder?.info.builder_type}
      status={
        builder?.info.builder_type === "Aws"
          ? builder?.info.instance_type
          : undefined
      }
    />
  );
};
