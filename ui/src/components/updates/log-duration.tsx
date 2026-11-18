import { Loader } from "@mantine/core";
import { Types } from "komodo_client";
import { Clock } from "lucide-react";
import { fmtDuration } from "mogh_ui";
import { useEffect, useState } from "react";

/**
 * The duration of a log. While its command runs, shows a loader
 * and counts up in whole seconds.
 */
export default function LogDuration({
  log,
  inProgress,
}: {
  log: Types.Log;
  inProgress: boolean;
}) {
  // Logs of running commands don't have an end yet
  if (inProgress && !log.end_ts) {
    return (
      <>
        <Loader size="1rem" />
        <LiveDuration start={log.start_ts} />
      </>
    );
  }
  return (
    <>
      <Clock size="1rem" />
      {fmtDuration(log.start_ts, log.end_ts || log.start_ts)}
    </>
  );
}

function LiveDuration({ start }: { start: number }) {
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    const interval = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(interval);
  }, []);
  const seconds = Math.floor(Math.max(0, now - start) / 1000);
  return seconds < 60 ? `${seconds} seconds` : fmtDuration(0, seconds * 1000);
}
