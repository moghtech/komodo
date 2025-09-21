import React, { useMemo, useCallback, useRef } from 'react';
import { LazyLog } from '@melloware/react-logviewer';
import { Button } from '@ui/button';
import { Badge } from '@ui/badge';
import { Download, Hash } from 'lucide-react';
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
  const logViewerRef = useRef<any>(null);

  // Split logs into lines for processing
  const logLines = useMemo(() => logs.split('\n'), [logs]);

  // Filter logs based on search and invert
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
            // Invalid regex, treat as literal
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

  // Get the text to display (filtered or original)
  const displayText = searchTerms.length > 0 ? filteredText : logs;

  // Export functionality
  const handleExport = useCallback((filtered: boolean) => {
    if (!onExport) return;
    const content = filtered && searchTerms.length > 0 ? filteredText : logs;
    onExport(content, filtered);
  }, [logs, filteredText, searchTerms, onExport]);

  // Build search text for LazyLog
  const searchText = useMemo(() => {
    if (searchTerms.length === 0) return '';

    // LazyLog doesn't support multiple search terms, so join them with |
    if (regex) {
      return searchTerms.join('|');
    }

    // For literal search, escape regex special characters and join
    const escaped = searchTerms.map(term =>
      term.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
    );
    return escaped.join('|');
  }, [searchTerms, regex]);

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
              // This is just for UI consistency
            }}
            title="Line numbers controlled by parent settings"
            disabled
          >
            <Hash className="w-4 h-4" />
          </Button>

          {/* Match count */}
          {searchTerms.length > 0 && (
            <Badge variant="secondary" className="text-xs">
              {filteredText ? filteredText.split('\n').filter(l => l).length : 0} lines
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

      {/* Log Viewer */}
      <div className="flex-1 overflow-hidden bg-background">
        <LazyLog
          ref={logViewerRef}
          text={displayText}
          enableSearch={searchTerms.length > 0}
          searchWords={searchText}
          caseInsensitive={!caseSensitive}
          enableLineNumbers={showLineNumbers}
          extraLines={1}
          follow={false}
          selectableLines
          highlight={searchTerms.length > 0 ? [0] : undefined}
          lineClassName="hover:bg-accent/20"
          highlightLineClassName="bg-yellow-100 dark:bg-yellow-900/30"
          style={{
            height: '100%',
            width: '100%',
            fontSize: '14px',
            fontFamily: 'ui-monospace, SFMono-Regular, "SF Mono", Consolas, "Liberation Mono", Menlo, monospace'
          }}
        />
      </div>

      {/* Status Bar */}
      <div className="flex items-center justify-between px-3 py-1 text-xs text-muted-foreground border-t bg-background/95">
        <span>{displayText.split('\n').length} lines</span>
        {searchTerms.length > 0 && (
          <span>{filteredText.split('\n').filter(l => l).length} matches</span>
        )}
      </div>
    </div>
  );
};