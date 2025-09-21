import React, { useMemo, useCallback, useRef, useEffect, useState } from 'react';
import { Button } from '@ui/button';
import { Badge } from '@ui/badge';
import { Download, Hash, ChevronUp, ChevronDown } from 'lucide-react';
import { cn } from '@lib/utils';

interface SimpleLogViewerProps {
  logs: string;
  searchTerms: string[];
  caseSensitive: boolean;
  regex: boolean;
  invert: boolean;
  showLineNumbers: boolean;
  onExport?: (content: string, filtered: boolean) => void;
  className?: string;
}

export const SimpleLogViewer: React.FC<SimpleLogViewerProps> = ({
  logs,
  searchTerms,
  caseSensitive,
  regex,
  invert,
  showLineNumbers,
  onExport,
  className
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [currentMatchIndex, setCurrentMatchIndex] = useState(0);

  // Split logs into lines for processing
  const logLines = useMemo(() => logs.split('\n'), [logs]);

  // Process logs for search/filter
  const processedData = useMemo(() => {
    const matchedLineIndices: number[] = [];
    let displayLines: string[] = logLines;

    if (searchTerms.length > 0) {
      const matchingIndices = new Set<number>();

      logLines.forEach((line, index) => {
        for (const term of searchTerms) {
          if (regex) {
            try {
              const re = new RegExp(term, caseSensitive ? '' : 'i');
              if (re.test(line)) {
                matchingIndices.add(index);
                break;
              }
            } catch {
              // Invalid regex, treat as literal
              if (caseSensitive ? line.includes(term) : line.toLowerCase().includes(term.toLowerCase())) {
                matchingIndices.add(index);
                break;
              }
            }
          } else {
            if (caseSensitive ? line.includes(term) : line.toLowerCase().includes(term.toLowerCase())) {
              matchingIndices.add(index);
              break;
            }
          }
        }
      });

      // Apply invert filter
      if (invert) {
        displayLines = logLines.filter((_, i) => !matchingIndices.has(i));
        logLines.forEach((_, i) => {
          if (!matchingIndices.has(i)) matchedLineIndices.push(i);
        });
      } else {
        displayLines = logLines.filter((_, i) => matchingIndices.has(i));
        matchingIndices.forEach(i => matchedLineIndices.push(i));
      }
    }

    return {
      displayLines,
      matchedLineIndices: matchedLineIndices.sort((a, b) => a - b),
      totalMatches: matchedLineIndices.length
    };
  }, [logLines, searchTerms, caseSensitive, regex, invert]);

  // Highlight search terms in a line
  const highlightLine = useCallback((line: string, lineIndex: number) => {
    if (searchTerms.length === 0 || invert) {
      return <span>{line}</span>;
    }

    const parts: React.ReactNode[] = [];
    let lastIndex = 0;

    searchTerms.forEach(term => {
      const searchStr = caseSensitive ? line : line.toLowerCase();
      const searchTerm = caseSensitive ? term : term.toLowerCase();

      if (!regex) {
        let index = searchStr.indexOf(searchTerm);
        while (index !== -1) {
          if (index > lastIndex) {
            parts.push(<span key={`${lineIndex}-${lastIndex}`}>{line.substring(lastIndex, index)}</span>);
          }
          parts.push(
            <mark key={`${lineIndex}-${index}`} className="bg-yellow-300 dark:bg-yellow-700 text-black dark:text-white">
              {line.substring(index, index + term.length)}
            </mark>
          );
          lastIndex = index + term.length;
          index = searchStr.indexOf(searchTerm, lastIndex);
        }
      }
    });

    if (lastIndex < line.length) {
      parts.push(<span key={`${lineIndex}-end`}>{line.substring(lastIndex)}</span>);
    }

    return parts.length > 0 ? <>{parts}</> : <span>{line}</span>;
  }, [searchTerms, caseSensitive, regex, invert]);

  // Navigate to next match
  const goToNextMatch = useCallback(() => {
    if (processedData.matchedLineIndices.length === 0) return;

    const nextIndex = (currentMatchIndex + 1) % processedData.matchedLineIndices.length;
    setCurrentMatchIndex(nextIndex);

    const lineElement = document.getElementById(`log-line-${processedData.matchedLineIndices[nextIndex]}`);
    lineElement?.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }, [currentMatchIndex, processedData.matchedLineIndices]);

  // Navigate to previous match
  const goToPrevMatch = useCallback(() => {
    if (processedData.matchedLineIndices.length === 0) return;

    const prevIndex = currentMatchIndex === 0
      ? processedData.matchedLineIndices.length - 1
      : currentMatchIndex - 1;
    setCurrentMatchIndex(prevIndex);

    const lineElement = document.getElementById(`log-line-${processedData.matchedLineIndices[prevIndex]}`);
    lineElement?.scrollIntoView({ behavior: 'smooth', block: 'center' });
  }, [currentMatchIndex, processedData.matchedLineIndices]);

  // Export functionality
  const handleExport = useCallback((filtered: boolean) => {
    if (!onExport) return;
    const content = filtered && searchTerms.length > 0
      ? processedData.displayLines.join('\n')
      : logs;
    onExport(content, filtered);
  }, [logs, processedData.displayLines, searchTerms, onExport]);

  // Reset match navigation when search changes
  useEffect(() => {
    setCurrentMatchIndex(0);
  }, [searchTerms, caseSensitive, regex, invert]);

  const displayText = searchTerms.length > 0
    ? processedData.displayLines
    : logLines;

  return (
    <div className={cn("flex flex-col h-full", className)}>
      {/* Controls Bar */}
      <div className="flex items-center justify-between gap-2 p-2 border-b bg-background/95 backdrop-blur">
        <div className="flex items-center gap-2">
          {/* Line Numbers Toggle */}
          <Button
            variant={showLineNumbers ? "secondary" : "ghost"}
            size="sm"
            onClick={() => {
              // Note: Line numbers are controlled by parent
            }}
            title="Line numbers controlled by parent settings"
            disabled
          >
            <Hash className="w-4 h-4" />
          </Button>

          {/* Search Navigation */}
          {searchTerms.length > 0 && !invert && (
            <>
              <div className="flex items-center gap-1">
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={goToPrevMatch}
                  disabled={processedData.matchedLineIndices.length === 0}
                >
                  <ChevronUp className="w-4 h-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={goToNextMatch}
                  disabled={processedData.matchedLineIndices.length === 0}
                >
                  <ChevronDown className="w-4 h-4" />
                </Button>
              </div>
              <Badge variant="secondary" className="text-xs">
                {processedData.matchedLineIndices.length > 0
                  ? `${currentMatchIndex + 1} of ${processedData.matchedLineIndices.length} matches`
                  : 'No matches'}
              </Badge>
            </>
          )}

          {/* Filtered count */}
          {searchTerms.length > 0 && invert && (
            <Badge variant="secondary" className="text-xs">
              {processedData.displayLines.length} lines
            </Badge>
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

      {/* Simple Log Display */}
      <div
        ref={containerRef}
        className="flex-1 overflow-auto bg-background font-mono text-sm"
        style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}
      >
        {displayText.map((line, index) => {
          const originalIndex = searchTerms.length > 0 && !invert
            ? processedData.matchedLineIndices[index]
            : index;
          const isCurrentMatch = searchTerms.length > 0 && !invert &&
            processedData.matchedLineIndices[currentMatchIndex] === originalIndex;

          return (
            <div
              key={index}
              id={`log-line-${originalIndex}`}
              className={cn(
                "flex hover:bg-accent/20",
                isCurrentMatch && "bg-yellow-100 dark:bg-yellow-900/30"
              )}
              style={{ minHeight: '20px' }}
            >
              {showLineNumbers && (
                <span className="px-2 text-muted-foreground select-none border-r min-w-[4rem] text-right">
                  {originalIndex + 1}
                </span>
              )}
              <div className="flex-1 px-2">
                {highlightLine(line, index)}
              </div>
            </div>
          );
        })}
      </div>

      {/* Status Bar */}
      <div className="flex items-center justify-between px-3 py-1 text-xs text-muted-foreground border-t bg-background/95">
        <span>{displayText.length} lines</span>
        {searchTerms.length > 0 && (
          <span>{processedData.totalMatches} matches</span>
        )}
      </div>
    </div>
  );
};