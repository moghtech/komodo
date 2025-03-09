import { useRead } from "@lib/hooks";
import { Types } from "komodo_client";
import { ReactNode } from "react";
import { useDeployment } from ".";
import { Log, LogSection } from "@components/log";

export const DeploymentLogs = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const state = useDeployment(id)?.info.state;
  if (
    state === undefined ||
    state === Types.DeploymentState.Unknown ||
    state === Types.DeploymentState.NotDeployed
  ) {
    return null;
  }
  return <DeploymentLogsInner id={id} titleOther={titleOther} />;
};

const DeploymentLogsInner = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  return (
    <LogSection
      regular_logs={(timestamps, stream, tail) =>
        NoSearchLogs(id, tail, timestamps, stream)
      }
      search_logs={(timestamps, terms, invert) =>
        SearchLogs(id, terms, invert, timestamps)
      }
      titleOther={titleOther}
    />
  );
};

const NoSearchLogs = (
  id: string,
  tail: number,
  timestamps: boolean,
  stream: string
) => {
  const { data: log, refetch } = useRead("GetDeploymentLog", {
    deployment: id,
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
  terms: string[],
  invert: boolean,
  timestamps: boolean
) => {
  const { data: log, refetch } = useRead("SearchDeploymentLog", {
    deployment: id,
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
