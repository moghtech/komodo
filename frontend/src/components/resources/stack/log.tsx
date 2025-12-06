import { LocalStorageSetter, useLocalStorage, useRead } from "@lib/hooks";
import { Types } from "komodo_client";
import { ReactNode } from "react";
import { useStack } from ".";
import { Log, LogSection } from "@components/log";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@ui/dropdown-menu";
import { CaretSortIcon } from "@radix-ui/react-icons";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";

export const StackLogs = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const stackInfo = useStack(id)?.info;
  const [selectedServices, setServices] = useLocalStorage<string[]>(
    `stack-${id}-log-services`,
    []
  );
  if (
    stackInfo === undefined ||
    stackInfo.state === Types.StackState.Unknown ||
    stackInfo.state === Types.StackState.Down
  ) {
    return null;
  }
  return (
    <StackLogsInner
      id={id}
      titleOther={titleOther}
      services={stackInfo.services.map((s) => ({
        service: s.service,
        selected: selectedServices.includes(s.service),
      }))}
      setServices={setServices}
      useSwarm={!!stackInfo.swarm_id}
    />
  );
};

const StackLogsInner = ({
  id,
  titleOther,
  services,
  setServices,
  useSwarm,
}: {
  id: string;
  titleOther: ReactNode;
  services: Array<{ service: string; selected: boolean }>;
  setServices: (state: string[] | LocalStorageSetter<string[]>) => void;
  useSwarm: boolean;
}) => {
  return (
    <LogSection
      regular_logs={(timestamps, stream, tail, poll) =>
        NoSearchLogs(
          id,
          services.filter((s) => s.selected).map((s) => s.service),
          tail,
          timestamps,
          stream,
          poll
        )
      }
      search_logs={(timestamps, terms, invert, poll) =>
        SearchLogs(
          id,
          services.filter((s) => s.selected).map((s) => s.service),
          terms,
          invert,
          timestamps,
          poll
        )
      }
      titleOther={titleOther}
      extraParams={
        useSwarm ? (
          <StackLogsSingleServiceSelector
            services={services}
            setServices={setServices}
          />
        ) : (
          <StackLogsMultiServiceSelector
            services={services}
            setServices={setServices}
          />
        )
      }
    />
  );
};

// Used for Swarm logs, where selecting single service is necessary
const StackLogsSingleServiceSelector = ({
  services,
  setServices,
}: {
  services: Array<{ service: string; selected: boolean }>;
  setServices: (state: string[] | LocalStorageSetter<string[]>) => void;
}) => {
  const service =
    services.find((s) => s.selected)?.service ?? services[0]?.service;
  return (
    <Select value={service} onValueChange={(service) => setServices([service])}>
      <SelectTrigger className="w-fit">
        <div className="flex items-center gap-2 pr-2">
          <div className="text-xs text-muted-foreground">Service:</div>
          <SelectValue placeholder="Select Service" />
        </div>
      </SelectTrigger>
      <SelectContent>
        {services.map(({ service }) => (
          <SelectItem key={service} value={service}>
            {service}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
};

// Used for compose logs, where multi service selection is possible
const StackLogsMultiServiceSelector = ({
  services,
  setServices,
}: {
  services: Array<{ service: string; selected: boolean }>;
  setServices: (state: string[] | LocalStorageSetter<string[]>) => void;
}) => {
  const selected = services.filter((s) => s.selected);
  return (
    <DropdownMenu>
      <DropdownMenuTrigger>
        <div className="px-3 py-2 border rounded-md flex items-center gap-2 hover:bg-accent/70 text-sm">
          <div className="text-muted-foreground">Services:</div>{" "}
          {selected.length === 0
            ? "All"
            : selected.map((s) => s.service).join(", ")}
          <CaretSortIcon className="h-4 w-4 opacity-50" />
        </div>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start">
        {services.map((s) => {
          return (
            <DropdownMenuCheckboxItem
              key={s.service}
              checked={s.selected}
              onClick={(e) => {
                e.preventDefault();
                if (s.selected) {
                  setServices((services) =>
                    services.filter((service) => service !== s.service)
                  );
                } else {
                  setServices((services) => [...services, s.service]);
                }
              }}
            >
              {s.service}
            </DropdownMenuCheckboxItem>
          );
        })}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

const NoSearchLogs = (
  id: string,
  services: string[],
  tail: number,
  timestamps: boolean,
  stream: string,
  poll: boolean
) => {
  const { data: log, refetch } = useRead(
    "GetStackLog",
    {
      stack: id,
      services,
      tail,
      timestamps,
    },
    { refetchInterval: poll ? 3000 : false }
  );
  return {
    Log: (
      <div className="relative">
        <Log log={log} stream={stream as "stdout" | "stderr"} />
      </div>
    ),
    refetch,
    stderr: !!log?.stderr,
  };
};

const SearchLogs = (
  id: string,
  services: string[],
  terms: string[],
  invert: boolean,
  timestamps: boolean,
  poll: boolean
) => {
  const { data: log, refetch } = useRead(
    "SearchStackLog",
    {
      stack: id,
      services,
      terms,
      combinator: Types.SearchCombinator.And,
      invert,
      timestamps,
    },
    { refetchInterval: poll ? 10000 : false }
  );
  return {
    Log: (
      <div className="h-full relative">
        <Log log={log} stream="stdout" />
      </div>
    ),
    refetch,
    stderr: !!log?.stderr,
  };
};
