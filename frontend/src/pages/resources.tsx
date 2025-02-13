import { ExportButton } from "@components/export";
import { Page } from "@components/layouts";
import { ResourceComponents } from "@components/resources";
import { TagsFilter } from "@components/tags";
import {
  useFilterResources,
  useRead,
  useResourceParamType,
  useSetTitle,
  useUser,
} from "@lib/hooks";
import { Types } from "komodo_client";
import { Input } from "@ui/input";
import { useState } from "react";
import { Search } from "lucide-react";
import { NotFound } from "@components/util";

export const Resources = () => {
  const is_admin = useUser().data?.admin ?? false;
  const disable_non_admin_create =
    useRead("GetCoreInfo", {}).data?.disable_non_admin_create ?? true;
  const type = useResourceParamType()!;
  const name =
    type === "ServerTemplate"
      ? "Server Template"
      : type === "ResourceSync"
        ? "Resource Sync"
        : type;
  useSetTitle(name + "s");
  const [search, set] = useState("");
  const resources = useRead(`List${type}s`, {}).data;
  const filtered = useFilterResources(resources as any, search);

  const Components = ResourceComponents[type];

  if (!Components) {
    return <NotFound type={undefined} />;
  }

  const targets = filtered?.map(
    (resource): Types.ResourceTarget => ({
      type,
      id: resource.id,
    })
  );

  return (
    <Page
      title={`${name}s`}
      subtitle={
        <div className="text-muted-foreground">
          <Components.Description />
        </div>
      }
      icon={<Components.BigIcon />}
      actions={<ExportButton targets={targets} />}
    >
      <div className="flex flex-col gap-4">
        <div className="flex flex-col md:flex-row gap4 items-center justify-between">
          <div className="flex gap-4">
            {(is_admin || !disable_non_admin_create) && <Components.New />}
            <Components.GroupActions />
          </div>
          <div className="flex items-center gap-4">
            <TagsFilter />
            <div className="relative">
              <Search className="w-4 absolute top-[50%] left-3 -translate-y-[50%] text-muted-foreground" />
              <Input
                value={search}
                onChange={(e) => set(e.target.value)}
                placeholder="search..."
                className="pl-8 w-[200px] lg:w-[300px]"
              />
            </div>
          </div>
        </div>
        <Components.Table resources={filtered ?? []} />
      </div>
    </Page>
  );
};
