import { useRead } from "@lib/hooks";
import { Types } from "komodo_client";
import { Log, LogSection } from "@components/log";
import { ReactNode } from "react";
import { Section } from "@components/layouts";

export const SwarmServiceLogs = ({
  id,
  service,
  titleOther,
  disabled,
  extraParams,
}: {
  /* Swarm id */
  id: string;
  /* Swarm service / task id */
  service: string;
  titleOther?: ReactNode;
  disabled: boolean;
  extraParams?: ReactNode;
}) => {
  if (disabled) {
    return (
      <Section titleOther={titleOther}>
        <h1>Logs are disabled.</h1>
      </Section>
    );
  }

  return (
    <SwarmServiceLogsInner
      titleOther={titleOther}
      id={id}
      service={service}
      extraParams={extraParams}
    />
  );
};

const SwarmServiceLogsInner = ({
  id,
  service,
  titleOther,
  extraParams,
}: {
  /// Swarm id
  id: string;
  service: string;
  titleOther?: ReactNode;
  extraParams?: ReactNode;
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
      extraParams={extraParams}
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
    "GetSwarmServiceLog",
    {
      swarm: id,
      service,
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
    "SearchSwarmServiceLog",
    {
      swarm: id,
      service,
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
