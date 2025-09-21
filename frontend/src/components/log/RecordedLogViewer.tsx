import React, { useMemo, useCallback, useState, useRef, useEffect } from 'react';
import { List, ListImperativeAPI } from 'react-window';
import { Button } from '@ui/button';
import { Badge } from '@ui/badge';
import { ChevronUp, ChevronDown, Download, Hash } from 'lucide-react';
import { cn } from '@lib/utils';

interface RecordedLogViewerProps {
  logs: string;
  searchTerms: string[];
  caseSensitive: boolean;
  regex: boolean;
  invert: boolean;
  showLineNumbers: boolean;
  onExport?: (content: string, filtered: boolean) => void;
  className?: string;
}

interface ProcessedLog {
  lines: string[];
  matchedLines: Set<number>;
  matchPositions: Map<number, number[]>;
  totalMatches: number;
}

interface LogLineData {
  lines: string[];
  matchedLines: Set<number>;
  matchPositions: Map<number, number[]>;
  searchTerms: string[];
  caseSensitive: boolean;
  regex: boolean;
  showLineNumbers: boolean;
  currentMatch: number | null;
}

interface LogLineProps {
  index: number;
  data: LogLineData;
}

// Helper function to escape HTML but preserve ANSI colors
function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

// Highlight search terms in a line
function highlightLine(
  line: string,
  searchTerms: string[],
  caseSensitive: boolean,
  regex: boolean,
  positions?: number[]
): string {
  if (!searchTerms.length || !positions?.length) {
    return escapeHtml(line);
  }

  let highlighted = escapeHtml(line);

  searchTerms.forEach(term => {
    if (regex) {
      try {
        const re = new RegExp(`(${term})`, caseSensitive ? 'g' : 'gi');
        highlighted = highlighted.replace(re, '<mark class="bg-yellow-300 text-black font-semibold">$1</mark>');
      } catch {
        // Invalid regex, treat as literal
        const escapedTerm = term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        const re = new RegExp(`(${escapedTerm})`, caseSensitive ? 'g' : 'gi');
        highlighted = highlighted.replace(re, '<mark class="bg-yellow-300 text-black font-semibold">$1</mark>');
      }
    } else {
      const escapedTerm = term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      const re = new RegExp(`(${escapedTerm})`, caseSensitive ? 'g' : 'gi');
      highlighted = highlighted.replace(re, '<mark class="bg-yellow-300 text-black font-semibold">$1</mark>');
    }
  });

  return highlighted;
}

// Component for rendering individual log lines
const LogLine = React.memo(({ index, data }: LogLineProps) => {
  const {
    lines,
    matchedLines,
    matchPositions,
    searchTerms,
    caseSensitive,
    regex,
    showLineNumbers,
    currentMatch
  } = data;

  const line = lines[index];
  const isMatched = matchedLines.has(index);
  const isCurrentMatch = currentMatch === index;
  const positions = matchPositions.get(index);

  const highlightedContent = useMemo(() => {
    if (searchTerms.length > 0 && isMatched) {
      return highlightLine(line, searchTerms, caseSensitive, regex, positions);
    }
    return escapeHtml(line);
  }, [line, searchTerms, caseSensitive, regex, isMatched, positions]);

  return (
    <div
      className={cn(
        "flex font-mono text-sm hover:bg-accent/20 h-5",
        isCurrentMatch && "bg-yellow-100 dark:bg-yellow-900/30"
      )}
    >
      {showLineNumbers && (
        <span className="px-2 text-muted-foreground select-none border-r min-w-[4rem] text-right">
          {index + 1}
        </span>
      )}
      <pre
        className="flex-1 px-2 whitespace-pre-wrap break-all"
        dangerouslySetInnerHTML={{ __html: highlightedContent }}
      />
    </div>
  );
});

LogLine.displayName = 'LogLine';

export const RecordedLogViewer: React.FC<RecordedLogViewerProps> = ({
  logs,
  searchTerms,
  caseSensitive,
  regex,
  invert,
  showLineNumbers,
  onExport,
  className
}) => {
  const [currentMatchIndex, setCurrentMatchIndex] = useState<number>(0);
  const [showLineNumbersLocal, setShowLineNumbersLocal] = useState(showLineNumbers);
  const listRef = useRef<ListImperativeAPI>(null);

  // Process logs with memoization
  const processedLog = useMemo<ProcessedLog>(() => {
    const lines = logs.split('\n');
    const matchedLines = new Set<number>();
    const matchPositions = new Map<number, number[]>();
    let totalMatches = 0;

    if (searchTerms.length === 0) {
      return {
        lines,
        matchedLines,
        matchPositions,
        totalMatches: 0
      };
    }

    lines.forEach((line, index) => {
      let lineMatches = false;
      const positions: number[] = [];

      for (const term of searchTerms) {
        if (regex) {
          try {
            const re = new RegExp(term, caseSensitive ? 'g' : 'gi');
            const matches = [...line.matchAll(re)];
            if (matches.length > 0) {
              lineMatches = true;
              matches.forEach(match => {
                if (match.index !== undefined) {
                  positions.push(match.index);
                  totalMatches++;
                }
              });
            }
          } catch {
            // Invalid regex, treat as literal
            if (caseSensitive ? line.includes(term) : line.toLowerCase().includes(term.toLowerCase())) {
              lineMatches = true;
              totalMatches++;
              // Find all occurrences
              let searchStr = caseSensitive ? line : line.toLowerCase();
              let searchTerm = caseSensitive ? term : term.toLowerCase();
              let idx = searchStr.indexOf(searchTerm);
              while (idx !== -1) {
                positions.push(idx);
                idx = searchStr.indexOf(searchTerm, idx + 1);
              }
            }
          }
        } else {
          if (caseSensitive ? line.includes(term) : line.toLowerCase().includes(term.toLowerCase())) {
            lineMatches = true;
            totalMatches++;
            // Find all occurrences
            let searchStr = caseSensitive ? line : line.toLowerCase();
            let searchTerm = caseSensitive ? term : term.toLowerCase();
            let idx = searchStr.indexOf(searchTerm);
            while (idx !== -1) {
              positions.push(idx);
              idx = searchStr.indexOf(searchTerm, idx + 1);
            }
          }
        }
      }

      if ((invert && !lineMatches) || (!invert && lineMatches)) {
        matchedLines.add(index);
        if (positions.length > 0) {
          matchPositions.set(index, positions);
        }
      }
    });

    return {
      lines: invert && searchTerms.length > 0
        ? lines.filter((_, i) => matchedLines.has(i))
        : lines,
      matchedLines,
      matchPositions,
      totalMatches
    };
  }, [logs, searchTerms, caseSensitive, regex, invert]);

  // Get sorted match line indices for navigation
  const sortedMatchIndices = useMemo(() => {
    return Array.from(processedLog.matchedLines).sort((a, b) => a - b);
  }, [processedLog.matchedLines]);

  // Navigate to next match
  const goToNextMatch = useCallback(() => {
    if (sortedMatchIndices.length === 0) return;

    const nextIndex = (currentMatchIndex + 1) % sortedMatchIndices.length;
    setCurrentMatchIndex(nextIndex);

    const lineIndex = sortedMatchIndices[nextIndex];
    listRef.current?.scrollToRow(lineIndex, 'center');
  }, [currentMatchIndex, sortedMatchIndices]);

  // Navigate to previous match
  const goToPrevMatch = useCallback(() => {
    if (sortedMatchIndices.length === 0) return;

    const prevIndex = currentMatchIndex === 0
      ? sortedMatchIndices.length - 1
      : currentMatchIndex - 1;
    setCurrentMatchIndex(prevIndex);

    const lineIndex = sortedMatchIndices[prevIndex];
    listRef.current?.scrollToRow(lineIndex, 'center');
  }, [currentMatchIndex, sortedMatchIndices]);

  // Reset match navigation when search changes
  useEffect(() => {
    setCurrentMatchIndex(0);
  }, [searchTerms, caseSensitive, regex, invert]);

  // Export functionality
  const handleExport = useCallback((filtered: boolean) => {
    if (!onExport) return;

    const content = filtered && searchTerms.length > 0
      ? processedLog.lines.join('\n')
      : logs;

    onExport(content, filtered);
  }, [logs, processedLog.lines, searchTerms, onExport]);

  // Data passed to each row
  const itemData = useMemo<LogLineData>(() => ({
    lines: processedLog.lines,
    matchedLines: processedLog.matchedLines,
    matchPositions: processedLog.matchPositions,
    searchTerms,
    caseSensitive,
    regex,
    showLineNumbers: showLineNumbersLocal,
    currentMatch: sortedMatchIndices[currentMatchIndex] ?? null
  }), [
    processedLog,
    searchTerms,
    caseSensitive,
    regex,
    showLineNumbersLocal,
    sortedMatchIndices,
    currentMatchIndex
  ]);

  const lineHeight = 20; // Height of each line in pixels

  return (
    <div className={cn("flex flex-col h-full", className)}>
      {/* Controls Bar */}
      <div className="flex items-center justify-between gap-2 p-2 border-b bg-background/95 backdrop-blur">
        <div className="flex items-center gap-2">
          {/* Line Numbers Toggle */}
          <Button
            variant={showLineNumbersLocal ? "secondary" : "ghost"}
            size="sm"
            onClick={() => setShowLineNumbersLocal(!showLineNumbersLocal)}
            title="Toggle line numbers"
          >
            <Hash className="w-4 h-4" />
          </Button>

          {/* Search Navigation */}
          {searchTerms.length > 0 && (
            <>
              <div className="flex items-center gap-1">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={goToPrevMatch}
                  disabled={sortedMatchIndices.length === 0}
                >
                  <ChevronUp className="w-4 h-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={goToNextMatch}
                  disabled={sortedMatchIndices.length === 0}
                >
                  <ChevronDown className="w-4 h-4" />
                </Button>
              </div>
              <Badge variant="secondary" className="text-xs">
                {sortedMatchIndices.length > 0
                  ? `${currentMatchIndex + 1} of ${sortedMatchIndices.length} matches`
                  : 'No matches'}
              </Badge>
            </>
          )}
        </div>

        {/* Export Options */}
        {onExport && (
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => handleExport(false)}
            >
              <Download className="w-4 h-4 mr-1" />
              Export All
            </Button>
            {searchTerms.length > 0 && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleExport(true)}
              >
                <Download className="w-4 h-4 mr-1" />
                Export Filtered
              </Button>
            )}
          </div>
        )}
      </div>

      {/* Virtual Scrolling Log Display */}
      <div className="flex-1 bg-background overflow-hidden">
        {processedLog.lines.length > 0 ? (
          <List
            listRef={listRef}
            rowCount={processedLog.lines.length}
            rowHeight={lineHeight}
            rowComponent={LogLine}
            rowProps={itemData}
            className="scrollbar-thin scrollbar-thumb-gray-400 dark:scrollbar-thumb-gray-600"
            style={{ height: '100%', width: '100%' }}
          />
        ) : (
          <div className="flex items-center justify-center h-full text-muted-foreground">
            {searchTerms.length > 0 ? 'No matches found' : 'No logs to display'}
          </div>
        )}
      </div>

      {/* Status Bar */}
      <div className="flex items-center justify-between px-3 py-1 text-xs text-muted-foreground border-t bg-background/95">
        <span>{processedLog.lines.length} lines</span>
        {searchTerms.length > 0 && (
          <span>{processedLog.totalMatches} total matches</span>
        )}
      </div>
    </div>
  );
};