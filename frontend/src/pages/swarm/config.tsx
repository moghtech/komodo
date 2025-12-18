import {
  ResourceDescription,
  ResourceLink,
  ResourcePageHeader,
} from "@components/resources/common";
import {
  useExecute,
  useLocalStorage,
  usePermissions,
  useRead,
  useSetTitle,
} from "@lib/hooks";
import { Button } from "@ui/button";
import { ChevronLeft, History, Loader2 } from "lucide-react";
import { Link, useParams } from "react-router-dom";
import { language_from_path, MonacoEditor } from "@components/monaco";
import { SWARM_ICONS, useSwarm } from "@components/resources/swarm";
import { Section } from "@components/layouts";
import { ExportButton } from "@components/export";
import { stroke_color_class_by_intention } from "@lib/color";
import { ResourceNotifications } from "@pages/resource-notifications";
import { Types } from "komodo_client";
import { Dispatch, ReactNode, SetStateAction, useMemo, useState } from "react";
import { MobileFriendlyTabsSelector } from "@ui/mobile-friendly-tabs";
import { RemoveSwarmResource } from "./remove";
import { ConfirmUpdate } from "@components/config/util";
import { useToast } from "@ui/use-toast";
import { Badge } from "@ui/badge";
import {
  SwarmServicesTable,
  SwarmTasksTable,
} from "@components/resources/swarm/table";

export default function SwarmConfigPage() {
  const { id, config: __config } = useParams() as {
    id: string;
    config: string;
  };
  const _config = decodeURIComponent(__config);
  const swarm = useSwarm(id);
  const inUse = useRead("ListSwarmConfigs", { swarm: id }).data?.find(
    (config) => config.ID === _config || config.Name === _config
  )?.InUse;
  const { data: config, isPending } = useRead("InspectSwarmConfig", {
    swarm: id,
    config: _config,
  });
  const { canWrite, canExecute } = usePermissions({
    type: "Swarm",
    id,
  });
  useSetTitle(`${swarm?.name} | Config | ${config?.Spec?.Name ?? "Unknown"}`);

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (!config) {
    return <div className="flex w-full py-4">Failed to inspect config.</div>;
  }

  const Icon = SWARM_ICONS.Config;
  const intent = inUse ? "Good" : "Critical";

  return (
    <div>
      <div className="w-full flex items-center justify-between mb-12">
        <Link to={"/swarms/" + id}>
          <Button className="gap-2" variant="secondary">
            <ChevronLeft className="w-4" />
            Back
          </Button>
        </Link>
        <div className="flex items-center gap-4">
          <ExportButton targets={[{ type: "Swarm", id }]} />
        </div>
      </div>
      <div className="flex flex-col xl:flex-row gap-4">
        {/* HEADER */}
        <div className="w-full flex flex-col gap-4">
          <div className="flex flex-col gap-2 border rounded-md">
            <ResourcePageHeader
              type={undefined}
              id={undefined}
              intent={intent}
              icon={
                <Icon
                  size={8}
                  className={stroke_color_class_by_intention(intent)}
                />
              }
              resource={undefined}
              name={config.Spec?.Name}
              state={!inUse && <Badge variant="destructive">Unused</Badge>}
              status={undefined}
            />
            <div className="flex flex-col pb-2 px-4">
              <div className="flex items-center gap-x-4 gap-y-0 flex-wrap text-muted-foreground">
                <div>Swarm Config</div>
                |
                <ResourceLink type="Swarm" id={id} />
              </div>
            </div>
          </div>
          <ResourceDescription type="Swarm" id={id} disabled={!canWrite} />
        </div>
        {/** NOTIFICATIONS */}
        <ResourceNotifications type="Swarm" id={id} />
      </div>

      <div className="mt-8 flex flex-col gap-12">
        {/* Actions */}
        {canExecute && config.ID && (
          <RemoveSwarmResource
            id={id}
            type="Config"
            resource_id={config.ID}
            resource_name={config.Spec?.Name}
            disabled={inUse}
          />
        )}

        {/* Tabs */}
        <div className="pt-4">
          {swarm && <SwarmConfigTabs swarm={swarm} config={_config} />}
        </div>
      </div>
    </div>
  );
}

type SwarmConfigTabsView = "Edit" | "Services" | "Tasks" | "Inspect";

const SwarmConfigTabs = ({
  swarm,
  config,
}: {
  swarm: Types.SwarmListItem;
  config: string;
}) => {
  const [_view, setView] = useLocalStorage<SwarmConfigTabsView>(
    `swarm-${swarm.id}-config-${config}-tabs-v2`,
    "Edit"
  );
  const { specificInspect } = usePermissions({
    type: "Swarm",
    id: swarm.id,
  });
  const _search = useState("");

  const view = !specificInspect && _view === "Inspect" ? "Edit" : _view;

  const tabs = useMemo(
    () => [
      {
        value: "Edit",
      },
      {
        value: "Services",
      },
      {
        value: "Tasks",
      },
      {
        value: "Inspect",
        disabled: !specificInspect,
      },
    ],
    [specificInspect]
  );

  const Selector = (
    <MobileFriendlyTabsSelector
      tabs={tabs}
      value={view}
      onValueChange={setView as any}
      tabsTriggerClassname="w-[110px]"
    />
  );

  switch (view) {
    case "Edit":
      return (
        <SwarmConfigEdit
          swarm={swarm.id}
          config={config}
          titleOther={Selector}
        />
      );
    case "Services":
      return (
        <SwarmConfigServices
          id={swarm.id}
          config={config}
          Selector={Selector}
          _search={_search}
        />
      );
    case "Tasks":
      return (
        <SwarmConfigTasks
          id={swarm.id}
          config={config}
          Selector={Selector}
          _search={_search}
        />
      );
    case "Inspect":
      return (
        <SwarmConfigInspect
          swarm={swarm.id}
          config={config}
          titleOther={Selector}
        />
      );
  }
};

const SwarmConfigEdit = ({
  swarm,
  config,
  titleOther,
}: {
  swarm: string;
  config: string;
  titleOther: ReactNode;
}) => {
  const {
    data: inspect,
    isPending,
    refetch,
  } = useRead("InspectSwarmConfig", {
    swarm,
    config,
  });
  const { canExecute } = usePermissions({
    type: "Swarm",
    id: swarm,
  });
  const { toast } = useToast();
  const { mutate: sendEdit } = useExecute("RotateSwarmConfig", {
    onSuccess: (res) => {
      toast({
        title: res.success ? "Config updated." : "Failed to update config.",
        variant: res.success ? undefined : "destructive",
      });
      setEdit({ edit: undefined });
      refetch();
    },
  });
  const [{ edit }, setEdit] = useLocalStorage<{ edit: string | undefined }>(
    `swarm-${swarm}-config-${config}-edit-v2`,
    { edit: undefined }
  );

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (!config) {
    return <div className="flex w-full py-4">Failed to inspect config.</div>;
  }

  const name = inspect?.Spec?.Name;
  const data = inspect?.Spec?.Data;
  const language = name ? language_from_path(name) : undefined;

  return (
    <Section
      titleOther={titleOther}
      actions={
        canExecute && (
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              onClick={(e) => {
                e.stopPropagation();
                setEdit({ edit: undefined });
              }}
              className="flex items-center gap-2"
              disabled={edit === undefined}
            >
              <History className="w-4 h-4" />
              Reset
            </Button>
            <span onClick={(e) => e.stopPropagation()}>
              <ConfirmUpdate
                previous={{ contents: data }}
                content={{ contents: edit }}
                onConfirm={async () => {
                  name && edit && sendEdit({ swarm, config: name, data: edit });
                }}
                disabled={edit === undefined}
                language={language}
                loading={isPending}
              />
            </span>
          </div>
        )
      }
    >
      <MonacoEditor
        value={edit ?? data ?? "Failed to retrieve config data"}
        language={language}
        onValueChange={(edit) => setEdit({ edit })}
        readOnly={!canExecute}
      />
    </Section>
  );
};

const SwarmConfigServices = ({
  id,
  config,
  Selector,
  _search,
}: {
  id: string;
  config: string | undefined;
  Selector: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const services =
    useRead("ListSwarmServices", { swarm: id }).data?.filter(
      (service) => config && service.Configs.includes(config)
    ) ?? [];
  return (
    <SwarmServicesTable
      id={id}
      services={services}
      _search={_search}
      titleOther={Selector}
    />
  );
};

const SwarmConfigTasks = ({
  id,
  config,
  Selector,
  _search,
}: {
  id: string;
  config: string | undefined;
  Selector: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const tasks =
    useRead("ListSwarmTasks", { swarm: id }).data?.filter(
      (service) => config && service.Configs.includes(config)
    ) ?? [];
  return (
    <SwarmTasksTable
      id={id}
      tasks={tasks}
      _search={_search}
      titleOther={Selector}
    />
  );
};

const SwarmConfigInspect = ({
  swarm,
  config,
  titleOther,
}: {
  swarm: string;
  config: string;
  titleOther: ReactNode;
}) => {
  const { data: inspect, isPending } = useRead("InspectSwarmConfig", {
    swarm,
    config,
  });

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (!config) {
    return <div className="flex w-full py-4">Failed to inspect config.</div>;
  }

  return (
    <Section titleOther={titleOther}>
      <MonacoEditor
        value={JSON.stringify(inspect, null, 2)}
        language="json"
        readOnly
      />
    </Section>
  );
};
