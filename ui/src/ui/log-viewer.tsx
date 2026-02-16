import {
  ActionIcon,
  Box,
  ScrollArea,
  ScrollAreaProps,
  Text,
} from "@mantine/core";
import { useEffect, useRef, useState, useMemo } from "react";
import Convert from "ansi-to-html";
import { ChevronsDown } from "lucide-react";

export interface LogViewerProps extends ScrollAreaProps {
  log: string | undefined;
  autoScroll?: boolean;
}

const convert = new Convert({
  fg: "#d4d4d4",
  bg: "#1e1e1e",
  newline: false,
  escapeXML: true,
  stream: false,
});

export default function LogViewer({
  log,
  autoScroll = true,
  h = "calc(100vh - 30rem)",
  ...props
}: LogViewerProps) {
  const viewportRef = useRef<HTMLDivElement>(null);
  const [isAtBottom, setIsAtBottom] = useState(true);

  // Split log into lines and memoize
  const lines = useMemo(() => {
    return log ? log.split("\n") : undefined;
  }, [log]);

  // Convert ANSI codes to HTML
  const renderLine = (line: string) => {
    try {
      return convert.toHtml(line);
    } catch {
      return line; // Fallback to plain text if conversion fails
    }
  };

  // Auto-scroll to bottom when new content arrives
  useEffect(() => {
    if (autoScroll && isAtBottom && viewportRef.current) {
      const viewport = viewportRef.current;
      viewport.scrollTop = viewport.scrollHeight;
    }
  }, [log, autoScroll, isAtBottom]);

  // Track if user is at bottom
  const handleScroll = () => {
    if (!viewportRef.current) return;
    const target = viewportRef.current;
    const threshold = 50; // pixels from bottom
    const atBottom =
      target.scrollHeight - target.scrollTop - target.clientHeight < threshold;
    setIsAtBottom(atBottom);
  };

  // Scroll to bottom manually
  const scrollToBottom = () => {
    if (viewportRef.current) {
      viewportRef.current.scrollTop = viewportRef.current.scrollHeight;
      setIsAtBottom(true);
    }
  };

  console.log(lines);

  return (
    <ScrollArea
      h={h}
      viewportRef={viewportRef}
      onScrollPositionChange={handleScroll}
      onScroll={handleScroll}
      bg="accent.0"
      className="bordered-light"
      pos="relative"
      {...props}
    >
      <Box
        component="pre"
        p="md"
        m={0}
        ff="monospace"
        fz="0.85rem"
        lh={1.5}
        style={{
          whiteSpace: "pre-wrap",
          wordBreak: "break-all",
        }}
      >
        {lines?.map((line, index) => (
          <Box
            key={index}
            component="div"
            style={{
              contentVisibility: "auto",
              containIntrinsicSize: "auto 1.5em",
            }}
            dangerouslySetInnerHTML={{
              __html: renderLine(line) || "&nbsp;",
            }}
          />
        ))}
        {!(lines?.length ?? 0) && <Text>No log.</Text>}
      </Box>
      <ActionIcon
        onClick={scrollToBottom}
        title="Scroll to bottom"
        size="lg"
        pos="absolute"
        top={12}
        right={12}
      >
        <ChevronsDown size="1.3rem" />
      </ActionIcon>
    </ScrollArea>
  );
}
