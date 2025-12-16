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
import { ReactNode, useMemo } from "react";
import { MobileFriendlyTabsSelector } from "@ui/mobile-friendly-tabs";
import { RemoveSwarmResource } from "./remove";
import { ConfirmUpdate } from "@components/config/util";
import { useToast } from "@ui/use-toast";

export default function SwarmConfigPage() {
  const { id, config: __config } = useParams() as {
    id: string;
    config: string;
  };
  const _config = decodeURIComponent(__config);
  const swarm = useSwarm(id);
  const { data: config, isPending } = useRead("InspectSwarmConfig", {
    swarm: id,
    config: _config,
  });
  const { canWrite } = usePermissions({
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
              intent="Good"
              icon={
                <Icon
                  size={8}
                  className={stroke_color_class_by_intention("Good")}
                />
              }
              resource={undefined}
              name={config.Spec?.Name}
              state={undefined}
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
        {config.ID && (
          <RemoveSwarmResource
            id={id}
            type="Config"
            resource_id={config.ID}
            resource_name={config.Spec?.Name}
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

type SwarmConfigTabsView = "Edit" | "Inspect";

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

  const view = !specificInspect && _view === "Inspect" ? "Edit" : _view;

  const tabs = useMemo(
    () => [
      {
        value: "Edit",
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
      setEdit(undefined);
      refetch();
    },
  });
  const [edit, setEdit] = useLocalStorage<string | undefined>(
    `swarm-${swarm}-config-${inspect?.ID}-edit`,
    undefined
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
                setEdit(undefined);
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
        onValueChange={(edit) => setEdit(edit)}
        readOnly={!canExecute}
      />
    </Section>
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

// export default function SwarmConfigPage() {
//   const { id, config: __config } = useParams() as {
//     id: string;
//     config: string;
//   };
//   const _config = decodeURIComponent(__config);
//   const swarm = useSwarm(id);
//   const { data: config, isPending } = useRead("InspectSwarmConfig", {
//     swarm: id,
//     config: _config,
//   });
//   useSetTitle(
//     `${swarm?.name} | Config | ${config?.Spec?.Name ?? config?.ID ?? "Unknown"}`
//   );
//   const { canExecute } = usePermissions({
//     type: "Swarm",
//     id,
//   });
//   const nav = useNavigate();

//   if (isPending) {
//     return (
//       <div className="flex justify-center w-full py-4">
//         <Loader2 className="w-8 h-8 animate-spin" />
//       </div>
//     );
//   }

//   if (!config) {
//     return <div className="flex w-full py-4">Failed to inspect config.</div>;
//   }

//   const Icon = SWARM_ICONS.Config;

//   return (
//     <div className="flex flex-col gap-16 mb-24">
//       {/* HEADER */}
//       <div className="flex flex-col gap-4">
//         {/* BACK */}
//         <div className="flex items-center justify-between mb-4">
//           <Button
//             className="gap-2"
//             variant="secondary"
//             onClick={() => nav("/swarms/" + id)}
//           >
//             <ChevronLeft className="w-4" /> Back
//           </Button>
//         </div>

//         {/* TITLE */}
//         <div className="flex items-center gap-4">
//           <div className="mt-1">
//             <Icon size={8} />
//           </div>
//           <PageHeaderName name={config?.Spec?.Name ?? config?.ID} />
//         </div>

//         {/* INFO */}
//         <div className="flex flex-wrap gap-4 items-center text-muted-foreground">
//           Swarm Config
//           <ResourceLink type="Swarm" id={id} />
//         </div>
//       </div>

//       {canExecute && config.ID && (
//         <Section title="Execute" icon={<Zap className="w-4 h-4" />}>
//           <div className="flex gap-4 items-center flex-wrap">
//             <RemoveSwarmResource
//               id={id}
//               type="Config"
//               resource_id={config.ID}
//               resource_name={config.Spec?.Name}
//             />
//           </div>
//         </Section>
//       )}

//       <MonacoEditor
//         value={JSON.stringify(config, null, 2)}
//         language="json"
//         readOnly
//       />
//     </div>
//   );
// }
