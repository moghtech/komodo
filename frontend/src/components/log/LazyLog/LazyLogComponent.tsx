import React, { useRef, useEffect, useCallback, useMemo, useState, forwardRef, useImperativeHandle } from 'react';
import { List } from 'react-window';
import { cn } from '@lib/utils';
import { Search } from 'lucide-react';
import { Input } from '@ui/input';
import { Button } from '@ui/button';

interface LazyLogProps {
  text: string;
  height?: string | number;
  width?: string | number;
  follow?: boolean;
  scrollToLine?: number;
  highlight?: number | number[];
  selectableLines?: boolean;
  enableSearch?: boolean;
  formatPart?: (text: string, lineIndex?: number) => string;
  // onScroll?: (event: any) => void; // Not used in react-window v2
  onHighlight?: (range: [number, number] | null) => void;
  onLineSelect?: (lines: Set<number>) => void;
  rowHeight?: number;
  overscanRowCount?: number;
  style?: React.CSSProperties;
  lineClassName?: string;
  highlightLineClassName?: string;
  extraLines?: number;
  caseInsensitive?: boolean;
  searchWords?: string;
  isRegex?: boolean;
  wrapLines?: boolean;
  getLogLevel?: (line: string) => { color: string; bg: string } | null;
}

export interface LazyLogHandle {
  scrollToLine: (lineNumber: number) => void;
  scrollToBottom: () => void;
}

const LazyLog = forwardRef<LazyLogHandle, LazyLogProps>(({
  text = '',
  height = 500,
  width = '100%',
  follow = false,
  scrollToLine = 0,
  highlight,
  selectableLines = false,
  enableSearch = false,
  formatPart,
  // onScroll, // removed - not used in react-window v2
  onHighlight,
  onLineSelect,
  rowHeight = 19,
  overscanRowCount = 100,
  style = {},
  lineClassName = '',
  highlightLineClassName = '',
  extraLines = 0,
  caseInsensitive = false,
  searchWords = '',
  isRegex = false,
  wrapLines = false,
  getLogLevel,
}, ref) => {
  const listRef = useRef<any>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [localSearch, setLocalSearch] = useState('');
  const [highlightRange, setHighlightRange] = useState<[number, number] | null>(null);
  const [containerHeight, setContainerHeight] = useState(500);
  const [selectedLines, setSelectedLines] = useState<Set<number>>(new Set());
  const [lastClickedLine, setLastClickedLine] = useState<number | null>(null);

  // Handle dynamic height
  useEffect(() => {
    if (typeof height === 'string' && height.includes('%') && containerRef.current) {
      const resizeObserver = new ResizeObserver(entries => {
        for (const entry of entries) {
          setContainerHeight(entry.contentRect.height - (enableSearch ? 50 : 0));
        }
      });
      resizeObserver.observe(containerRef.current.parentElement || containerRef.current);
      return () => resizeObserver.disconnect();
    } else if (typeof height === 'number') {
      setContainerHeight(height - (enableSearch ? 50 : 0));
    }
  }, [height, enableSearch]);

  // Split text into lines
  const lines = useMemo(() => {
    if (!text) return [];
    const splitLines = text.split('\n');
    // Add extra empty lines if requested
    if (extraLines > 0) {
      for (let i = 0; i < extraLines; i++) {
        splitLines.push('');
      }
    }
    return splitLines;
  }, [text, extraLines]);

  // Calculate highlight range
  useEffect(() => {
    if (highlight) {
      if (typeof highlight === 'number') {
        setHighlightRange([highlight, highlight]);
      } else if (Array.isArray(highlight) && highlight.length === 2) {
        setHighlightRange([highlight[0], highlight[1]]);
      }
    } else {
      setHighlightRange(null);
    }
  }, [highlight]);

  // Expose methods via ref
  useImperativeHandle(ref, () => ({
    scrollToLine: (lineNumber: number) => {
      if (listRef.current && lineNumber > 0 && lineNumber <= lines.length) {
        listRef.current.scrollToRow({
          index: lineNumber - 1,
          align: 'start'
        });
      }
    },
    scrollToBottom: () => {
      if (listRef.current && lines.length > 0) {
        listRef.current.scrollToRow({
          index: lines.length - 1,
          align: 'end'
        });
      }
    }
  }), [lines.length]);

  // Auto-scroll to bottom when following
  useEffect(() => {
    if (follow && listRef.current && lines.length > 0) {
      listRef.current.scrollToRow({
        index: lines.length - 1,
        align: 'end'
      });
    }
  }, [follow, lines.length]);

  // Scroll to specific line
  useEffect(() => {
    if (scrollToLine > 0 && listRef.current && !follow) {
      listRef.current.scrollToRow({
        index: scrollToLine - 1,
        align: 'start'
      });
    }
  }, [scrollToLine, follow]);

  // Format and highlight text
  const formatLine = useCallback((line: string): string => {
    let formattedLine = line;

    // Apply custom formatting if provided
    if (formatPart) {
      formattedLine = formatPart(formattedLine);
    }

    // Apply search highlighting
    const searchText = localSearch || searchWords;
    if (searchText) {
      try {
        let pattern: string;
        if (isRegex) {
          pattern = searchText;
        } else {
          // Escape special regex characters
          pattern = searchText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
        }

        const flags = caseInsensitive ? 'gi' : 'g';
        const regex = new RegExp(`(${pattern})`, flags);
        const parts = formattedLine.split(regex);

        return parts.map((part, index) => {
          if (index % 2 === 1) {
            return `<mark class="log-search-highlight">${part}</mark>`;
          }
          return part;
        }).join('');
      } catch (e) {
        // Invalid regex, return original
        return formattedLine;
      }
    }

    return formattedLine;
  }, [formatPart, localSearch, searchWords, isRegex, caseInsensitive]);

  // Row renderer for react-window v2
  const Row = useCallback((props: any) => {
    const { index, style, ...restProps } = props;
    const lineNumber = index + 1;
    const line = lines[index];
    const isHighlighted = highlightRange && lineNumber >= highlightRange[0] && lineNumber <= highlightRange[1];
    const isSelected = selectedLines.has(lineNumber);
    const formattedLine = formatLine(line);
    const logLevel = getLogLevel ? getLogLevel(line) : null;

    const handleClick = (e: React.MouseEvent) => {
      if (!selectableLines) return;

      const newSelected = new Set(selectedLines);

      if (e.shiftKey && lastClickedLine !== null) {
        // Range selection
        const start = Math.min(lastClickedLine, lineNumber);
        const end = Math.max(lastClickedLine, lineNumber);
        for (let i = start; i <= end; i++) {
          newSelected.add(i);
        }
      } else if (e.ctrlKey || e.metaKey) {
        // Toggle selection
        if (newSelected.has(lineNumber)) {
          newSelected.delete(lineNumber);
        } else {
          newSelected.add(lineNumber);
        }
      } else {
        // Single selection
        newSelected.clear();
        newSelected.add(lineNumber);
        setHighlightRange([lineNumber, lineNumber]);
        onHighlight?.([lineNumber, lineNumber]);
      }

      setSelectedLines(newSelected);
      setLastClickedLine(lineNumber);
      onLineSelect?.(newSelected);
    };

    return (
      <div
        {...restProps}
        style={{
          ...style,
          lineHeight: `${rowHeight}px`,
          fontFamily: 'ui-monospace, SFMono-Regular, "SF Mono", Consolas, "Liberation Mono", Menlo, monospace',
          fontSize: '14px',
          wordBreak: 'break-all',
          padding: '0 8px',
          cursor: selectableLines ? 'pointer' : 'default',
          whiteSpace: wrapLines ? 'pre-wrap' : 'nowrap',
          overflow: wrapLines ? 'visible' : 'hidden',
          textOverflow: wrapLines ? 'clip' : 'ellipsis',
        }}
        className={cn(
          lineClassName,
          isHighlighted && highlightLineClassName,
          'log-line',
          isHighlighted && 'log-line-highlight',
          isSelected && 'bg-blue-100 dark:bg-blue-900/30',
          logLevel?.bg
        )}
        onClick={handleClick}
        dangerouslySetInnerHTML={{ __html: formattedLine || '&nbsp;' }}
      />
    );
  }, [lines, highlightRange, formatLine, lineClassName, highlightLineClassName, selectableLines, rowHeight, onHighlight]);

  const computedHeight = typeof height === 'string' && height.includes('%') ? '100%' : height;

  return (
    <div ref={containerRef} style={{ width, height: computedHeight, ...style }} className="react-lazylog">
      {enableSearch && (
        <div className="flex items-center gap-2 p-2 border-b">
          <Search className="w-4 h-4 text-muted-foreground" />
          <Input
            value={localSearch}
            onChange={(e) => setLocalSearch(e.target.value)}
            placeholder="Search..."
            className="flex-1"
          />
          {localSearch && (
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setLocalSearch('')}
            >
              Clear
            </Button>
          )}
        </div>
      )}
      <List
        listRef={listRef}
        rowCount={lines.length}
        rowHeight={rowHeight}
        overscanCount={overscanRowCount}
        style={{ height: containerHeight, width: typeof width === 'string' ? width : `${width}px` }}
        rowComponent={Row as any}
        rowProps={{}}
      />
    </div>
  );
});

LazyLog.displayName = 'LazyLog';

export default LazyLog;