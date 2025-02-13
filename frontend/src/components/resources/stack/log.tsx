import { useRead } from "@lib/hooks";
import { Types } from "komodo_client";
import { ReactNode } from "react";
import { useStack } from ".";
import { Log, LogSection } from "@components/log";

export const StackLogs = ({
  id,
  titleOther,
}: {
  id: string;
  titleOther: ReactNode;
}) => {
  const state = useStack(id)?.info.state;
  if (
    state === undefined ||
    state === Types.StackState.Unknown ||
    state === Types.StackState.Down
  ) {
    return null;
  }
  return <StackLogsInner id={id} titleOther={titleOther} />;
};

const StackLogsInner = ({
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
  const { data: log, refetch } = useRead("GetStackLog", {
    stack: id,
    services: [],
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
  const { data: log, refetch } = useRead("SearchStackLog", {
    stack: id,
    services: [],
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
