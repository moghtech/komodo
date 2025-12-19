import {
  ResourceDescription,
  ResourceLink,
  ResourcePageHeader,
} from "@components/resources/common";
import {
  useLocalStorage,
  usePermissions,
  useRead,
  useSetTitle,
} from "@lib/hooks";
import { Button } from "@ui/button";
import { ChevronLeft, Loader2 } from "lucide-react";
import { Link, useParams } from "react-router-dom";
import { MonacoEditor } from "@components/monaco";
import { SWARM_ICONS, useSwarm } from "@components/resources/swarm";
import { Section } from "@components/layouts";
import { ExportButton } from "@components/export";
import {
  stroke_color_class_by_intention,
  swarm_node_state_intention,
} from "@lib/color";
import { ResourceNotifications } from "@pages/resource-notifications";
import { Types } from "komodo_client";
import { Dispatch, ReactNode, SetStateAction, useMemo, useState } from "react";
import { MobileFriendlyTabsSelector } from "@ui/mobile-friendly-tabs";
import { RemoveSwarmResource } from "./remove";
import { SwarmTasksTable } from "@components/resources/swarm/table";

export default function SwarmNodePage() {
  const { id, node: __node } = useParams() as {
    id: string;
    node: string;
  };
  const _node = decodeURIComponent(__node);
  const swarm = useSwarm(id);
  const { data: node, isPending } = useRead("InspectSwarmNode", {
    swarm: id,
    node: _node,
  });
  const state = node?.Status?.State;
  const { canWrite, canExecute } = usePermissions({
    type: "Swarm",
    id,
  });
  useSetTitle(
    `${swarm?.name} | Node | ${node?.Description?.Hostname ?? "Unknown"}`
  );

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (!node) {
    return <div className="flex w-full py-4">Failed to inspect node.</div>;
  }

  const Icon = SWARM_ICONS.Node;
  const intent = swarm_node_state_intention(state);

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
              name={node.Description?.Hostname}
              state={state}
              status={node.Spec?.Role}
            />
            <div className="flex flex-col pb-2 px-4">
              <div className="flex items-center gap-x-4 gap-y-0 flex-wrap text-muted-foreground">
                <div>Swarm Node</div>
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
        {canExecute && node.ID && (
          <RemoveSwarmResource
            id={id}
            type="Node"
            resource_id={node.ID}
            resource_name={node.Description?.Hostname}
          />
        )}

        {/* Tabs */}
        <div className="pt-4">
          {swarm && <SwarmNodeTabs swarm={swarm} _node={_node} node={node} />}
        </div>
      </div>
    </div>
  );
}

type SwarmNodeTabsView = "Tasks" | "Inspect";

const SwarmNodeTabs = ({
  swarm,
  _node,
  node,
}: {
  swarm: Types.SwarmListItem;
  _node: string;
  node: Types.SwarmNode;
}) => {
  const [_view, setView] = useLocalStorage<SwarmNodeTabsView>(
    `swarm-${swarm.id}-node-${node}-tabs-v1`,
    "Tasks"
  );
  const { specificInspect } = usePermissions({
    type: "Swarm",
    id: swarm.id,
  });
  const _search = useState("");

  const view = !specificInspect && _view === "Inspect" ? "Tasks" : _view;

  const tabs = useMemo(
    () => [
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
    case "Tasks":
      return (
        <SwarmNodeTasks
          id={swarm.id}
          node_id={node.ID}
          Selector={Selector}
          _search={_search}
        />
      );
    case "Inspect":
      return (
        <SwarmNodeInspect swarm={swarm.id} node={_node} titleOther={Selector} />
      );
  }
};

const SwarmNodeTasks = ({
  id,
  node_id,
  Selector,
  _search,
}: {
  id: string;
  node_id: string | undefined;
  Selector: ReactNode;
  _search: [string, Dispatch<SetStateAction<string>>];
}) => {
  const tasks =
    useRead("ListSwarmTasks", { swarm: id }).data?.filter(
      (service) => node_id && service.NodeID === node_id
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

const SwarmNodeInspect = ({
  swarm,
  node,
  titleOther,
}: {
  swarm: string;
  node: string;
  titleOther: ReactNode;
}) => {
  const { data: inspect, isPending } = useRead("InspectSwarmNode", {
    swarm,
    node,
  });

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (!node) {
    return <div className="flex w-full py-4">Failed to inspect node.</div>;
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
