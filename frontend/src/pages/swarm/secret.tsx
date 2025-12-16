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

export default function SwarmSecretPage() {
  const { id, secret: __secret } = useParams() as {
    id: string;
    secret: string;
  };
  const _secret = decodeURIComponent(__secret);
  const swarm = useSwarm(id);
  const { data: secret, isPending } = useRead("InspectSwarmSecret", {
    swarm: id,
    secret: _secret,
  });
  const { canWrite } = usePermissions({
    type: "Swarm",
    id,
  });
  useSetTitle(`${swarm?.name} | Secret | ${secret?.Spec?.Name ?? "Unknown"}`);

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (!secret) {
    return <div className="flex w-full py-4">Failed to inspect secret.</div>;
  }

  const Icon = SWARM_ICONS.Secret;

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
              name={secret.Spec?.Name}
              state={undefined}
              status={undefined}
            />
            <div className="flex flex-col pb-2 px-4">
              <div className="flex items-center gap-x-4 gap-y-0 flex-wrap text-muted-foreground">
                <div>Swarm Secret</div>
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
        {secret.ID && (
          <RemoveSwarmResource
            id={id}
            type="Secret"
            resource_id={secret.ID}
            resource_name={secret.Spec?.Name}
          />
        )}

        {/* Tabs */}
        <div className="pt-4">
          {swarm && <SwarmSecretTabs swarm={swarm} secret={_secret} />}
        </div>
      </div>
    </div>
  );
}

type SwarmSecretTabsView = "Edit" | "Inspect";

const SwarmSecretTabs = ({
  swarm,
  secret,
}: {
  swarm: Types.SwarmListItem;
  secret: string;
}) => {
  const [_view, setView] = useLocalStorage<SwarmSecretTabsView>(
    `swarm-${swarm.id}-secret-${secret}-tabs-v2`,
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
        <SwarmSecretEdit
          swarm={swarm.id}
          secret={secret}
          titleOther={Selector}
        />
      );
    case "Inspect":
      return (
        <SwarmSecretInspect
          swarm={swarm.id}
          secret={secret}
          titleOther={Selector}
        />
      );
  }
};

const SwarmSecretEdit = ({
  swarm,
  secret,
  titleOther,
}: {
  swarm: string;
  secret: string;
  titleOther: ReactNode;
}) => {
  const {
    data: inspect,
    isPending,
    refetch,
  } = useRead("InspectSwarmSecret", {
    swarm,
    secret,
  });
  const { canExecute } = usePermissions({
    type: "Swarm",
    id: swarm,
  });
  const { toast } = useToast();
  const { mutate: sendEdit } = useExecute("RotateSwarmSecret", {
    onSuccess: (res) => {
      toast({
        title: res.success ? "Secret updated." : "Failed to update secret.",
        variant: res.success ? undefined : "destructive",
      });
      setEdit({ edit: undefined });
      refetch();
    },
  });
  const [{ edit }, setEdit] = useLocalStorage<{ edit: string | undefined }>(
    `swarm-${swarm}-secret-${secret}-edit`,
    { edit: undefined }
  );

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (!secret) {
    return <div className="flex w-full py-4">Failed to inspect secret.</div>;
  }

  const name = inspect?.Spec?.Name;
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
                previous={{ contents: "" }}
                content={{ contents: edit }}
                onConfirm={async () => {
                  name && edit && sendEdit({ swarm, secret: name, data: edit });
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
        value={edit ?? "# Enter new secret and save to rotate"}
        language={language}
        onValueChange={(edit) => setEdit({ edit })}
        readOnly={!canExecute}
      />
    </Section>
  );
};

const SwarmSecretInspect = ({
  swarm,
  secret,
  titleOther,
}: {
  swarm: string;
  secret: string;
  titleOther: ReactNode;
}) => {
  const { data: inspect, isPending } = useRead("InspectSwarmSecret", {
    swarm,
    secret,
  });

  if (isPending) {
    return (
      <div className="flex justify-center w-full py-4">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (!secret) {
    return <div className="flex w-full py-4">Failed to inspect secret.</div>;
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
