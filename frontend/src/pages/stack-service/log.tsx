import { useRead } from "@lib/hooks";
import { Types } from "komodo_client";
import { Log, LogSection } from "@components/log";
import { ReactNode } from "react";
import { Section } from "@components/layouts";

export const StackServiceLogs = ({
  id,
  service: _service,
  titleOther,
  disabled,
}: {
  /// Stack id
  id: string;
  service: string;
  titleOther?: ReactNode;
  disabled: boolean;
}) => {
  // const stack = useStack(id);
  const services = useRead("ListStackServices", { stack: id }).data;
  const service = services?.find((s) => s.service === _service);
  const logsAvailable = service?.swarm_service
    ? service.swarm_service.State !== Types.SwarmState.Unknown
    : (service?.container?.state ?? Types.ContainerStateStatusEnum.Empty) !==
      Types.ContainerStateStatusEnum.Empty;

  if (disabled || !logsAvailable) {
    return (
      <Section titleOther={titleOther}>
        <h1>Logs are not available or disabled.</h1>
      </Section>
    );
  }

  return <StackLogsInner titleOther={titleOther} id={id} service={_service} />;
};

const StackLogsInner = ({
  id,
  service,
  titleOther,
}: {
  /// Stack id
  id: string;
  service: string;
  titleOther?: ReactNode;
}) => {
  return (
    <LogSection
      titleOther={titleOther}
      regular_logs={(timestamps, stream, tail, poll) =>
        NoSearchLogs(id, service, tail, timestamps, stream, poll)
      }
      search_logs={(timestamps, terms, invert, poll) =>
        SearchLogs(id, service, terms, invert, timestamps, poll)
      }
    />
  );
};

const NoSearchLogs = (
  id: string,
  service: string,
  tail: number,
  timestamps: boolean,
  stream: string,
  poll: boolean
) => {
  const { data: log, refetch } = useRead(
    "GetStackLog",
    {
      stack: id,
      services: [service],
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
  service: string,
  terms: string[],
  invert: boolean,
  timestamps: boolean,
  poll: boolean
) => {
  const { data: log, refetch } = useRead(
    "SearchStackLog",
    {
      stack: id,
      services: [service],
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
