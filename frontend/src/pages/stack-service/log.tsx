import { useRead } from "@lib/hooks";
import { Types } from "komodo_client";
import { Log, LogSection } from "@components/log";

export const StackServiceLogs = ({
  id,
  service,
}: {
  /// Stack id
  id: string;
  service: string;
}) => {
  // const stack = useStack(id);
  const services = useRead("ListStackServices", { stack: id }).data;
  const container = services?.find((s) => s.service === service)?.container;
  const state = container?.state ?? Types.ContainerStateStatusEnum.Empty;

  if (state === undefined || state === Types.ContainerStateStatusEnum.Empty) {
    return null;
  }

  return <StackLogsInner id={id} service={service} />;
};

const StackLogsInner = ({
  id,
  service,
}: {
  /// Stack id
  id: string;
  service: string;
}) => {
  return (
    <LogSection
      regular_logs={(timestamps, stream, tail) =>
        NoSearchLogs(id, service, tail, timestamps, stream)
      }
      search_logs={(timestamps, terms, invert) =>
        SearchLogs(id, service, terms, invert, timestamps)
      }
    />
  );
};

const NoSearchLogs = (
  id: string,
  service: string,
  tail: number,
  timestamps: boolean,
  stream: string
) => {
  const { data: log, refetch } = useRead("GetStackLog", {
    stack: id,
    services: [service],
    tail,
    timestamps,
  });
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
  service: string,
  terms: string[],
  invert: boolean,
  timestamps: boolean
) => {
  const { data: log, refetch } = useRead("SearchStackLog", {
    stack: id,
    services: [service],
    terms,
    combinator: Types.SearchCombinator.And,
    invert,
    timestamps,
  });
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
