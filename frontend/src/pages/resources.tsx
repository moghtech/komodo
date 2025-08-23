import { ExportButton } from "@components/export";
import { Page } from "@components/layouts";
import { ResourceComponents } from "@components/resources";
import { TagsFilter } from "@components/tags";
import {
  useFilterByUpdateAvailable,
  useFilterResources,
  useRead,
  useResourceParamType,
  useSetTitle,
  useTemplatesQueryBehavior,
  useUser,
  useLocalStorage,
} from "@lib/hooks";
import { Input } from "@ui/input";
import { Button } from "@ui/button";
import { useState } from "react";
import { Search, Eye, EyeOff } from "lucide-react";
import { NotFound, TemplateQueryBehaviorSelector } from "@components/util";
import { Switch } from "@ui/switch";
import { UsableResource } from "@types";
import { ServerMonitoringTable } from "@components/resources/server/monitoring-table";

export default function Resources({ _type }: { _type?: UsableResource }) {
  const is_admin = useUser().data?.admin ?? false;
  const disable_non_admin_create =
    useRead("GetCoreInfo", {}).data?.disable_non_admin_create ?? true;
  const __type = useResourceParamType()!;
  const type = _type ? _type : __type;
  const name = type === "ResourceSync" ? "Resource Sync" : type;
  useSetTitle(name + "s");
  const [search, set] = useState("");
  const [monitoring, setMonitoring] = useLocalStorage<boolean>(
    "servers-monitoring-toggle-v1",
    false
  );
  const [filter_update_available, toggle_filter_update_available] =
    useFilterByUpdateAvailable();
  const query =
    type === "Stack" || type === "Deployment"
      ? {
          query: {
            specific: { update_available: filter_update_available },
          },
        }
      : {};
  const [templatesQueryBehavior] = useTemplatesQueryBehavior();
  const resources = useRead(`List${type}s`, query).data;
  const templatesFilterFn =
    String(templatesQueryBehavior) === "Exclude"
      ? (resource: any) => !resource.template
      : String(templatesQueryBehavior) === "Only"
        ? (resource: any) => resource.template
        : () => true;
  const filtered = useFilterResources(resources as any, search).filter(
    templatesFilterFn
  );

  const Components = ResourceComponents[type];

  if (!Components) {
    return <NotFound type={undefined} />;
  }

  const targets = filtered?.map((resource) => ({ type, id: resource.id }));

  return (
    <Page
      title={`${name}s`}
      subtitle={
        <div className="text-muted-foreground">
          <Components.Description />
        </div>
      }
      icon={<Components.BigIcon />}
      actions={
        <div className="flex items-center gap-2">
          {type === "Server" && (
            <Button
              variant="outline"
              className="flex items-center gap-2"
              onClick={() => setMonitoring(!monitoring)}
            >
              {monitoring ? (
                <>
                  <EyeOff className="w-4 h-4" />
                  Hide Server Stats
                </>
              ) : (
                <>
                  <Eye className="w-4 h-4" />
                  Show Server Stats
                </>
              )}
            </Button>
          )}
          <ExportButton targets={targets} />
        </div>
      }
    >
      <div className="flex flex-col gap-4">
        <div className="flex flex-wrap gap-4 items-center justify-between">
          <div className="flex gap-4">
            {(is_admin || !disable_non_admin_create) && <Components.New />}
            {!(type === "Server" && monitoring) && <Components.GroupActions />}
          </div>
          <div className="flex items-center gap-4 flex-wrap">
            {(type === "Stack" || type === "Deployment") && (
              <div
                className="flex gap-2 items-center cursor-pointer px-3 py-2 text-sm text-muted-foreground"
                onClick={() => toggle_filter_update_available()}
              >
                Pending Update
                <Switch checked={filter_update_available} />
              </div>
            )}
            {!(type === "Server" && monitoring) && (
              <TemplateQueryBehaviorSelector />
            )}
            {!(type === "Server" && monitoring) && <TagsFilter />}
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
        {type === "Server" && monitoring ? (
          <ServerMonitoringTable search={search} />
        ) : (
          <Components.Table resources={filtered ?? []} />
        )}
      </div>
    </Page>
  );
}
