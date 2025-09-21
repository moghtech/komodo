import React, { useMemo, useCallback, useState, useRef, useEffect } from 'react';
import { LazyLog, ScrollFollow } from './LazyLog';
import type { LazyLogHandle } from './LazyLog';
import { Button } from '@ui/button';
import { Badge } from '@ui/badge';
import { Download, ChevronUp, ChevronDown } from 'lucide-react';
import { cn } from '@lib/utils';

interface LogViewerProps {
  logs: string;
  searchTerms: string[];
  caseSensitive: boolean;
  regex: boolean;
  invert: boolean;
  showLineNumbers: boolean;
  onExport?: (content: string, filtered: boolean, format?: 'text' | 'json' | 'csv') => void;
  className?: string;
  wrapLines?: boolean;
  showLogLevels?: boolean;
  onCopyLines?: (lines: string) => void;
}

const LOG_LEVELS = {
  ERROR: { pattern: /\b(ERROR|ERR|FATAL|FAIL)\b/i, color: 'text-red-500', bg: 'bg-red-50 dark:bg-red-950/20' },
  WARN: { pattern: /\b(WARN|WARNING)\b/i, color: 'text-yellow-600', bg: 'bg-yellow-50 dark:bg-yellow-950/20' },
  INFO: { pattern: /\b(INFO|INFORMATION)\b/i, color: 'text-blue-500', bg: 'bg-blue-50 dark:bg-blue-950/20' },
  DEBUG: { pattern: /\b(DEBUG|TRACE|VERBOSE)\b/i, color: 'text-gray-500', bg: 'bg-gray-50 dark:bg-gray-950/20' },
  SUCCESS: { pattern: /\b(SUCCESS|OK|DONE|COMPLETE)\b/i, color: 'text-green-500', bg: 'bg-green-50 dark:bg-green-950/20' },
};

export const LogViewer: React.FC<LogViewerProps> = ({
  logs,
  searchTerms,
  caseSensitive,
  regex,
  invert,
  showLineNumbers: _showLineNumbers,
  onExport,
  className,
  wrapLines = false,
  showLogLevels = true,
  onCopyLines
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const lazyLogRef = useRef<LazyLogHandle>(null);
  const [selectedLines, setSelectedLines] = useState<Set<number>>(new Set());
  const [currentSearchIndex, setCurrentSearchIndex] = useState(0);
  const [searchMatches, setSearchMatches] = useState<number[]>([]);

  const logLines = useMemo(() => logs.split('\n'), [logs]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.key === 'F3' || (e.ctrlKey && e.key === 'g')) && !e.shiftKey) {
        e.preventDefault();
        if (searchMatches.length > 0) {
          const nextIndex = currentSearchIndex < searchMatches.length - 1 ? currentSearchIndex + 1 : 0;
          setCurrentSearchIndex(nextIndex);
          if (lazyLogRef.current) {
            lazyLogRef.current.scrollToLine(searchMatches[nextIndex]);
          }
        }
      }
      else if ((e.key === 'F3' && e.shiftKey) || (e.ctrlKey && e.shiftKey && e.key === 'G')) {
        e.preventDefault();
        if (searchMatches.length > 0) {
          const prevIndex = currentSearchIndex > 0 ? currentSearchIndex - 1 : searchMatches.length - 1;
          setCurrentSearchIndex(prevIndex);
          if (lazyLogRef.current) {
            lazyLogRef.current.scrollToLine(searchMatches[prevIndex]);
          }
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [currentSearchIndex, searchMatches]);

  useEffect(() => {
    if (searchTerms.length === 0) {
      setSearchMatches([]);
      setCurrentSearchIndex(0);
      return;
    }

    const matches: number[] = [];
    logLines.forEach((line, index) => {
      let lineMatches = false;

      for (const term of searchTerms) {
        if (regex) {
          try {
            const re = new RegExp(term, caseSensitive ? '' : 'i');
            if (re.test(line)) {
              lineMatches = true;
              break;
            }
          } catch {
            if (caseSensitive ? line.includes(term) : line.toLowerCase().includes(term.toLowerCase())) {
              lineMatches = true;
              break;
            }
          }
        } else {
          if (caseSensitive ? line.includes(term) : line.toLowerCase().includes(term.toLowerCase())) {
            lineMatches = true;
            break;
          }
        }
      }

      if (invert ? !lineMatches : lineMatches) {
        matches.push(index + 1); // Line numbers are 1-based
      }
    });

    setSearchMatches(matches);
    setCurrentSearchIndex(0);

    if (matches.length > 0 && lazyLogRef.current) {
      setTimeout(() => {
        lazyLogRef.current?.scrollToLine(matches[0]);
      }, 100);
    }
  }, [searchTerms, logLines, caseSensitive, regex, invert]);

  const filteredText = useMemo(() => {
    if (searchTerms.length === 0) {
      return logs;
    }

    const filtered = logLines.filter((line) => {
      let matches = false;

      for (const term of searchTerms) {
        if (regex) {
          try {
            const re = new RegExp(term, caseSensitive ? '' : 'i');
            if (re.test(line)) {
              matches = true;
              break;
            }
          } catch {
            if (caseSensitive ? line.includes(term) : line.toLowerCase().includes(term.toLowerCase())) {
              matches = true;
              break;
            }
          }
        } else {
          if (caseSensitive ? line.includes(term) : line.toLowerCase().includes(term.toLowerCase())) {
            matches = true;
            break;
          }
        }
      }

      return invert ? !matches : matches;
    });

    return filtered.join('\n');
  }, [logLines, searchTerms, caseSensitive, regex, invert]);

  const displayText = searchTerms.length > 0 ? filteredText : logs;

  const handleExport = useCallback((filtered: boolean, format: 'text' | 'json' | 'csv' = 'text') => {
    if (!onExport) return;
    const content = filtered && searchTerms.length > 0 ? filteredText : logs;
    onExport(content, filtered, format);
  }, [logs, filteredText, searchTerms, onExport]);

  const formattedText = useMemo(() => {
    return displayText || '';
  }, [displayText]);

  const searchOptions = useMemo(() => {
    if (searchTerms.length === 0) return undefined;

    const searchText = searchTerms.join('|');

    return {
      searchWords: searchText,
      caseInsensitive: !caseSensitive,
      isRegex: regex
    };
  }, [searchTerms, caseSensitive, regex]);

  const getLogLevel = useCallback((line: string) => {
    if (!showLogLevels) return null;

    for (const [level, config] of Object.entries(LOG_LEVELS)) {
      if (config.pattern.test(line)) {
        return { level, ...config };
      }
    }
    return null;
  }, [showLogLevels]);

  const formatPart = useCallback((text: string) => {
    if (!searchOptions || !searchOptions.searchWords) return text;

    try {
      const pattern = searchOptions.isRegex
        ? searchOptions.searchWords
        : searchOptions.searchWords.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

      const flags = searchOptions.caseInsensitive ? 'gi' : 'g';
      const regex = new RegExp(`(${pattern})`, flags);

      const parts = text.split(regex);

      let result = parts.map((part, index) => {
        if (index % 2 === 1) {
          return `<mark class="log-search-highlight">${part}</mark>`;
        }
        return part;
      }).join('');

      return result;
    } catch {
      return text;
    }
  }, [searchOptions, showLogLevels, getLogLevel, logLines]);

  return (
    <div className={cn("flex flex-col h-full", className)}>
      <div className="flex items-center justify-between gap-2 p-2 border-b bg-background/95 backdrop-blur">
        <div className="flex items-center gap-2">
          {searchTerms.length > 0 ? (
            <>
              <Badge variant="secondary" className="text-xs">
                {searchMatches.length} {searchMatches.length === 1 ? 'match' : 'matches'}
              </Badge>
              {searchMatches.length > 0 && (
                <div className="flex items-center gap-1">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => {
                      const prevIndex = currentSearchIndex > 0 ? currentSearchIndex - 1 : searchMatches.length - 1;
                      setCurrentSearchIndex(prevIndex);
                      if (lazyLogRef.current) {
                        lazyLogRef.current.scrollToLine(searchMatches[prevIndex]);
                      }
                    }}
                    title="Previous match (Shift+F3)"
                  >
                    <ChevronUp className="w-4 h-4" />
                  </Button>
                  <span className="text-xs text-muted-foreground">
                    {currentSearchIndex + 1}/{searchMatches.length}
                  </span>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => {
                      const nextIndex = currentSearchIndex < searchMatches.length - 1 ? currentSearchIndex + 1 : 0;
                      setCurrentSearchIndex(nextIndex);
                      if (lazyLogRef.current) {
                        lazyLogRef.current.scrollToLine(searchMatches[nextIndex]);
                      }
                    }}
                    title="Next match (F3)"
                  >
                    <ChevronDown className="w-4 h-4" />
                  </Button>
                </div>
              )}
            </>
          ) : (
            <span className="text-xs text-muted-foreground">No active search</span>
          )}

          {selectedLines.size > 0 && (
            <Button
              variant="secondary"
              size="sm"
              onClick={() => {
                const linesToCopy = Array.from(selectedLines)
                  .sort((a, b) => a - b)
                  .map(lineNum => logLines[lineNum - 1])
                  .join('\n');
                navigator.clipboard.writeText(linesToCopy);
                if (onCopyLines) onCopyLines(linesToCopy);
                setSelectedLines(new Set());
              }}
            >
              Copy {selectedLines.size} lines
            </Button>
          )}
        </div>

        {onExport && (
          <div className="flex items-center gap-2">
            <div className="flex items-center gap-1 border rounded-md">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => handleExport(searchTerms.length > 0, 'text')}
                className="rounded-r-none"
              >
                <Download className="w-4 h-4 mr-1" />
                .log
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => handleExport(searchTerms.length > 0, 'json')}
                className="rounded-none border-x"
              >
                .json
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => handleExport(searchTerms.length > 0, 'csv')}
                className="rounded-l-none"
              >
                .csv
              </Button>
            </div>
          </div>
        )}
      </div>

      <div className="flex-1 overflow-hidden" ref={containerRef}>
        <ScrollFollow
          startFollowing={true}
          render={({ follow }) => (
            <LazyLog
              ref={lazyLogRef}
              text={formattedText}
              caseInsensitive={!caseSensitive}
              enableSearch={false}
              selectableLines={true}
              searchWords={searchOptions?.searchWords}
              isRegex={searchOptions?.isRegex}
              follow={follow}
              formatPart={formatPart}
              wrapLines={wrapLines}
              getLogLevel={getLogLevel}
              onLineSelect={setSelectedLines}
              lineClassName="log-line"
              highlightLineClassName="log-line-highlight"
              highlight={searchMatches.length > 0 ? searchMatches[currentSearchIndex] : undefined}
              extraLines={1}
              height="100%"
              width="100%"
              style={{
                background: 'var(--background)',
                color: 'var(--foreground)',
                fontSize: '14px',
                fontFamily: 'ui-monospace, SFMono-Regular, "SF Mono", Consolas, "Liberation Mono", Menlo, monospace'
              }}
              rowHeight={19}
              overscanRowCount={100}
            />
          )}
        />
      </div>

      <div className="flex items-center justify-between px-3 py-1 text-xs text-muted-foreground border-t bg-background/95">
        <div className="flex items-center gap-4">
          <span>{displayText.split('\n').length} lines</span>
          {showLogLevels && (
            <div className="flex items-center gap-2">
              <span className="text-red-500">● ERROR</span>
              <span className="text-yellow-600">● WARN</span>
              <span className="text-blue-500">● INFO</span>
              <span className="text-gray-500">● DEBUG</span>
            </div>
          )}
        </div>
        <div className="flex items-center gap-2">
          {selectedLines.size > 0 && (
            <span>{selectedLines.size} selected</span>
          )}
          <span className="text-muted-foreground/60">
            Press Shift+Click to select range
          </span>
        </div>
      </div>

    </div>
  );
};