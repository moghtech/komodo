import React, { useMemo, useCallback } from 'react';
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

      {/* Log Viewer - Using LazyLog properly */}
      <div className="flex-1 overflow-hidden" style={{ position: 'relative' }}>
        <LazyLog
          text={displayText}
          caseInsensitive={!caseSensitive}
          enableLineNumbers={showLineNumbers}
          extraLines={1}
          follow={false}
          selectableLines={true}
          enableLinks={false}
          enableHotKeys={false}
          // Remove height/width as they might be causing issues
          // Use style instead
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            width: '100%',
            height: '100%',
            background: 'var(--background)',
            color: 'var(--foreground)',
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