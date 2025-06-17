import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@ui/sheet";
import {
  Calendar,
  Clock,
  Loader2,
  Milestone,
  Settings,
  User,
} from "lucide-react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@ui/card";
import { ReactNode, useEffect, useState } from "react";
import { useRead } from "@lib/hooks";
import { ResourceComponents } from "@components/resources";
import { Link } from "react-router-dom";
import { fmt_duration, fmt_operation, fmt_version } from "@lib/formatting";
import {
  cn,
  updateLogToHtml,
  usableResourcePath,
  version_is_none,
} from "@lib/utils";
import { UsableResource } from "@types";
import { UserAvatar } from "@components/util";
import { ResourceNameSimple } from "@components/resources/common";
import { useWebsocketMessages } from "@lib/socket";
import { MonacoDiffEditor } from "@components/monaco";

export const UpdateUser = ({
  user_id,
  className,
  iconSize = 4,
  defaultAvatar,
  muted,
}: {
  user_id: string;
  className?: string;
  iconSize?: number;
  defaultAvatar?: boolean;
  muted?: boolean;
}) => {
  const res = useRead("GetUsername", { user_id }).data;
  const username = res?.username;
  const avatar = res?.avatar;
  return (
    <div
      className={cn(
        "flex items-center gap-2 text-nowrap",
        muted && "text-muted-foreground",
        className
      )}
    >
      {defaultAvatar ? (
        <User className={`w-${iconSize} h-${iconSize}`} />
      ) : (
        <UserAvatar avatar={avatar} size={iconSize} />
      )}
      {username || "Unknown"}
    </div>
  );
};

export const UpdateDetails = ({
  id,
  children,
}: {
  id: string;
  children: ReactNode;
}) => {
  const [open, setOpen] = useState(false);
  return (
    <UpdateDetailsInner
      id={id}
      children={children}
      open={open}
      setOpen={setOpen}
    />
  );
};

export const UpdateDetailsInner = ({
  id,
  children,
  open,
  setOpen,
}: {
  id: string;
  children?: ReactNode;
  open: boolean;
  setOpen: React.Dispatch<React.SetStateAction<boolean>>;
}) => {
  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <SheetTrigger asChild>{children}</SheetTrigger>
      <SheetContent
        className="mx-auto w-[1200px] max-w-[100vw] rounded-b-md"
        side="top"
      >
        <UpdateDetailsContent id={id} open={open} setOpen={setOpen} />
      </SheetContent>
    </Sheet>
  );
};

const UpdateDetailsContent = ({
  id,
  open,
  setOpen,
}: {
  id: string;
  open: boolean;
  setOpen: React.Dispatch<React.SetStateAction<boolean>>;
}) => {
  const { data: update, refetch } = useRead(
    "GetUpdate",
    { id },
    { enabled: false }
  );
  useEffect(() => {
    // handle open state change loading
    if (open) {
      refetch();
    }
  }, [open]);
  // Since auto refetching is disabled, listen for updates on the update id and refetch
  useWebsocketMessages("update-details", (update) => {
    if (update.id === id) refetch();
  });
  if (!update)
    return (
      <div className="w-full flex justify-center">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  const Components =
    update.target.type === "System"
      ? null
      : ResourceComponents[update.target.type];
  return (
    <>
      <SheetHeader className="mb-4">
        <SheetTitle>
          {fmt_operation(update.operation)}{" "}
          {!version_is_none(update.version) && fmt_version(update.version)}
        </SheetTitle>
        <SheetDescription className="flex flex-col gap-2">
          <UpdateUser user_id={update.operator} />
          <div className="flex gap-4">
            {Components ? (
              <Link
                to={`/${usableResourcePath(
                  update.target.type as UsableResource
                )}/${update.target.id}`}
              >
                <div
                  className="flex items-center gap-2"
                  onClick={() => setOpen(false)}
                >
                  <Components.Icon id={update.target.id} />
                  <ResourceNameSimple
                    // will be UsableResource because Components exists in this branch
                    type={update.target.type as UsableResource}
                    id={update.target.id}
                  />
                </div>
              </Link>
            ) : (
              <div className="flex items-center gap-2">
                <Settings className="w-4 h-4" />
                System
              </div>
            )}
            {update.version && (
              <div className="flex items-center gap-2">
                <Milestone className="w-4 h-4" />
                {fmt_version(update.version)}
              </div>
            )}
          </div>
          <div className="flex gap-4">
            <div className="flex items-center gap-2">
              <Calendar className="w-4 h-4" />
              {new Date(update.start_ts).toLocaleString()}
            </div>
            <div className="flex items-center gap-2">
              <Clock className="w-4 h-4" />
              {update.end_ts
                ? fmt_duration(update.start_ts, update.end_ts)
                : "ongoing"}
            </div>
          </div>
        </SheetDescription>
      </SheetHeader>
      <div className="grid gap-2 max-h-[calc(85vh-110px)] overflow-y-auto">
        {update.prev_toml && update.current_toml && (
          <Card>
            <CardHeader>
              <CardTitle>Changes made</CardTitle>
            </CardHeader>
            <CardContent>
              <MonacoDiffEditor
                original={update.prev_toml}
                modified={update.current_toml}
                language="toml"
                readOnly
              />
            </CardContent>
          </Card>
        )}
        {update.logs?.map((log, i) => (
          <Card key={i}>
            <CardHeader className="flex-col">
              <CardTitle>{log.stage}</CardTitle>
              <CardDescription className="flex gap-2">
                <span>
                  Stage {i + 1} of {update.logs.length}
                </span>
                <span>|</span>
                <span className="flex items-center gap-2">
                  <Clock className="w-4 h-4" />
                  {fmt_duration(log.start_ts, log.end_ts)}
                </span>
              </CardDescription>
            </CardHeader>
            <CardContent className="flex flex-col gap-2">
              {log.command && (
                <div>
                  <CardDescription>command</CardDescription>
                  <pre className="max-h-[500px] overflow-y-auto">
                    {log.command}
                  </pre>
                </div>
              )}
              {log.stdout && (
                <div>
                  <CardDescription>stdout</CardDescription>
                  <pre
                    dangerouslySetInnerHTML={{
                      __html: updateLogToHtml(log.stdout),
                    }}
                    className="max-h-[500px] overflow-y-auto"
                  />
                </div>
              )}
              {log.stderr && (
                <div>
                  <CardDescription>stderr</CardDescription>
                  <pre
                    dangerouslySetInnerHTML={{
                      __html: updateLogToHtml(log.stderr),
                    }}
                    className="max-h-[500px] overflow-y-auto"
                  />
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </>
  );
};
