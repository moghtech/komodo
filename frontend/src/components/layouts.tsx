import { Button } from "@ui/button";
import { PlusCircle } from "lucide-react";
import { ReactNode, useState } from "react";
import { Link, Outlet, useNavigate } from "react-router-dom";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@ui/dialog";
import { Types } from "komodo_client";
import { ResourceComponents } from "./resources";
import { Card, CardHeader, CardTitle, CardContent, CardFooter } from "@ui/card";
import { ResourceTags } from "./tags";
import { Topbar } from "./topbar";
import { cn, usableResourcePath } from "@lib/utils";
import { Sidebar } from "./sidebar";
import { ResourceNameSimple } from "./resources/common";
import { useSettingsView, useShiftKeyListener } from "@lib/hooks";

export const Layout = () => {
  const nav = useNavigate();
  const [_, setSettingsView] = useSettingsView();

  useShiftKeyListener("H", () => nav("/"));
  useShiftKeyListener("G", () => nav("/servers"));
  useShiftKeyListener("Z", () => nav("/stacks"));
  useShiftKeyListener("D", () => nav("/deployments"));
  useShiftKeyListener("B", () => nav("/builds"));
  useShiftKeyListener("R", () => nav("/repos"));
  useShiftKeyListener("P", () => nav("/procedures"));
  useShiftKeyListener("X", () => nav("/terminals"));
  useShiftKeyListener("C", () => nav("/schedules"));
  useShiftKeyListener("V", () => {
    setSettingsView("Variables");
    nav("/settings");
  });

  return (
    <>
      <Topbar />
      <div className="h-screen overflow-y-scroll">
        <div className="container px-[1.2rem]">
          <Sidebar />
          <div className="lg:ml-[200px] lg:pl-8 py-[88px]">
            <Outlet />
          </div>
        </div>
      </div>
    </>
  );
};

interface PageProps {
  title?: ReactNode;
  icon?: ReactNode;
  titleRight?: ReactNode;
  titleOther?: ReactNode;
  children?: ReactNode;
  subtitle?: ReactNode;
  actions?: ReactNode;
  superHeader?: ReactNode;
  className?: string;
}

export const Page = ({
  superHeader,
  title,
  icon,
  titleRight,
  titleOther,
  subtitle,
  actions,
  children,
  className,
}: PageProps) => {
  const Header = (
    <>
      {(title || icon || subtitle || actions) && (
        <div
          className={`flex flex-col gap-6 md:flex-row md:gap-0 md:justify-between`}
        >
          <div className="flex flex-col gap-4">
            <div className="flex flex-wrap gap-4 items-center">
              {icon}
              <h1 className="text-4xl">{title}</h1>
              {titleRight}
            </div>
            <div className="flex flex-col">{subtitle}</div>
          </div>
          {actions}
        </div>
      )}
    </>
  );
  return (
    <div className={cn("w-full flex flex-col gap-12", className)}>
      {superHeader ? (
        <div className="flex flex-col gap-4">
          {superHeader}
          {Header}
        </div>
      ) : (
        Header
      )}
      {titleOther}
      {children}
    </div>
  );
};

export const PageXlRow = ({
  superHeader,
  title,
  icon,
  titleRight,
  titleOther,
  subtitle,
  actions,
  children,
}: PageProps) => (
  <div className="flex flex-col gap-10 container py-8 pr-12">
    {superHeader ? (
      <div className="flex flex-col gap-4">
        {superHeader}
        {(title || icon || subtitle || actions) && (
          <div
            className={`flex flex-col gap-6 lg:flex-row lg:gap-0 lg:justify-between`}
          >
            <div className="flex flex-col gap-4">
              <div className="flex flex-wrap gap-4 items-center">
                {icon}
                <h1 className="text-4xl">{title}</h1>
                {titleRight}
              </div>
              <div className="flex flex-col">{subtitle}</div>
            </div>
            {actions}
          </div>
        )}
      </div>
    ) : (
      (title || icon || subtitle || actions) && (
        <div
          className={`flex flex-col gap-6 lg:flex-row lg:gap-0 lg:justify-between`}
        >
          <div className="flex flex-col gap-4">
            <div className="flex flex-wrap gap-4 items-center">
              {icon}
              <h1 className="text-4xl">{title}</h1>
              {titleRight}
            </div>
            <div className="flex flex-col">{subtitle}</div>
          </div>
          {actions}
        </div>
      )
    )}
    {titleOther}
    {children}
  </div>
);

interface SectionProps {
  title?: ReactNode;
  icon?: ReactNode;
  titleRight?: ReactNode;
  titleOther?: ReactNode;
  children?: ReactNode;
  actions?: ReactNode;
  // otherwise items-start
  itemsCenterTitleRow?: boolean;
  className?: string;
}

export const Section = ({
  title,
  icon,
  titleRight,
  titleOther,
  actions,
  children,
  itemsCenterTitleRow,
  className,
}: SectionProps) => (
  <div className={cn("flex flex-col gap-4", className)}>
    {(title || icon || titleRight || titleOther || actions) && (
      <div
        className={cn(
          "flex flex-wrap gap-4 justify-between mb-3",
          itemsCenterTitleRow ? "items-center" : "items-start"
        )}
      >
        {title || icon ? (
          <div className="px-2 flex items-center gap-2 text-muted-foreground">
            {icon}
            {title && <h2 className="text-xl">{title}</h2>}
            {titleRight}
          </div>
        ) : (
          titleOther
        )}
        {actions}
      </div>
    )}
    {children}
  </div>
);

export const NewLayout = ({
  entityType,
  children,
  enabled,
  onConfirm,
  onOpenChange,
  configureLabel = "a unique name",
  open: _open,
  setOpen: _setOpen,
}: {
  entityType: string;
  children: ReactNode;
  enabled: boolean;
  onConfirm: () => Promise<unknown>;
  onOpenChange?: (open: boolean) => void;
  configureLabel?: string;
  open?: boolean;
  setOpen?: (open: boolean) => void;
}) => {
  const [__open, __setOpen] = useState(false);
  const open = _open ? _open : __open;
  const setOpen = _setOpen ? _setOpen : __setOpen;
  const [loading, setLoading] = useState(false);

  return (
    <Dialog
      open={open}
      onOpenChange={(open) => {
        setOpen(open);
        onOpenChange && onOpenChange(open);
      }}
    >
      <DialogTrigger asChild>
        <Button className="items-center gap-2" variant="secondary">
          New {entityType} <PlusCircle className="w-4 h-4" />
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>New {entityType}</DialogTitle>
          <DialogDescription>
            Enter {configureLabel} for the new {entityType}.
          </DialogDescription>
        </DialogHeader>

        <div className="flex flex-col gap-6 py-8">{children}</div>

        <DialogFooter>
          <Button
            variant="secondary"
            onClick={async () => {
              setLoading(true);
              try {
                await onConfirm();
                setOpen(false);
              } catch (error: any) {
                const status = error?.status || error?.response?.status;
                if (status !== 409 && status !== 400) {
                  setOpen(false);
                }
              } finally {
                setLoading(false);
              }
            }}
            disabled={!enabled || loading}
          >
            Create
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export const ResourceCard = ({
  target: { type, id },
}: {
  target: Exclude<Types.ResourceTarget, { type: "System" }>;
}) => {
  const Components = ResourceComponents[type];

  return (
    <Link
      to={`/${usableResourcePath(type)}/${id}`}
      className="group hover:translate-y-[-2.5%] focus:translate-y-[-2.5%] transition-transform"
    >
      <Card className="h-full hover:bg-accent/50 group-focus:bg-accent/50 transition-colors">
        <CardHeader className="flex-row justify-between">
          <div>
            <CardTitle>
              <ResourceNameSimple type={type} id={id} />
            </CardTitle>
            {/* <CardDescription>
              <Components.Description id={id} />
            </CardDescription> */}
          </div>
          <Components.Icon id={id} />
        </CardHeader>
        <CardContent className="text-sm text-muted-foreground">
          {Object.entries(Components.Info).map(([key, Info]) => (
            <Info key={key} id={id} />
          ))}
        </CardContent>
        <CardFooter className="flex items-center gap-2">
          <ResourceTags target={{ type, id }} />
        </CardFooter>
      </Card>
    </Link>
  );
};

export const ResourceRow = ({
  target: { type, id },
}: {
  target: Exclude<Types.ResourceTarget, { type: "System" }>;
}) => {
  const Components = ResourceComponents[type];

  return (
    <Link
      to={`/${usableResourcePath(type)}/${id}`}
      className="group hover:translate-y-[-2.5%] focus:translate-y-[-2.5%] transition-transform"
    >
      <Card className="h-full hover:bg-accent/50 group-focus:bg-accent/50 transition-colors">
        <CardHeader className="grid grid-cols-4 items-center">
          <CardTitle>
            <ResourceNameSimple type={type} id={id} />
          </CardTitle>
          {Object.entries(Components.Info).map(([key, Info]) => (
            <Info key={key} id={id} />
          ))}
          <div className="flex items-center gap-2">
            <Components.Icon id={id} />
            {/* <CardDescription>
              <Components.Description id={id} />
            </CardDescription> */}
          </div>
        </CardHeader>
      </Card>
    </Link>
  );
};
