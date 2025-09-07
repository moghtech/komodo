import { cn } from "@lib/utils";
import {
  Column,
  ColumnDef,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  Row,
  RowSelectionState,
  SortingState,
  useReactTable,
} from "@tanstack/react-table";

import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@ui/table";
import { ArrowDown, ArrowUp, Minus } from "lucide-react";
import { ReactNode, useEffect, useState } from "react";
import { Checkbox } from "./checkbox";

interface DataTableProps<TData, TValue> {
  /** Unique key given to table so sorting can be remembered on local storage */
  tableKey: string;
  columns: (ColumnDef<TData, TValue> | false | undefined)[];
  data: TData[];
  onRowClick?: (row: TData) => void;
  noResults?: ReactNode;
  defaultSort?: SortingState;
  sortDescFirst?: boolean;
  selectOptions?: {
    selectKey: (row: TData) => string;
    onSelect: (selected: string[]) => void;
    disableRow?: boolean | ((row: Row<TData>) => boolean);
  };
  containerClassName?: string;
}

export function DataTable<TData, TValue>({
  tableKey,
  columns,
  data,
  onRowClick,
  noResults,
  sortDescFirst = false,
  defaultSort = [],
  selectOptions,
  containerClassName,
}: DataTableProps<TData, TValue>) {
  const [sorting, setSorting] = useState<SortingState>(defaultSort);

  // intentionally not initialized to clear selected values on table mount
  // could add some prop for adding default selected state to preserve between mounts
  const [rowSelection, setRowSelection] = useState<RowSelectionState>({});

  const table = useReactTable({
    data,
    columns: columns.filter((c) => c) as any,
    getCoreRowModel: getCoreRowModel(),
    onSortingChange: setSorting,
    getSortedRowModel: getSortedRowModel(),
    state: {
      sorting,
      rowSelection,
    },
    sortDescFirst,
    onRowSelectionChange: setRowSelection,
    getRowId: selectOptions?.selectKey,
    enableRowSelection: selectOptions?.disableRow,
  });

  useEffect(() => {
    const stored = localStorage.getItem("data-table-" + tableKey);
    const sorting = stored ? (JSON.parse(stored) as SortingState) : null;
    if (sorting) setSorting(sorting);
  }, [tableKey]);

  useEffect(() => {
    localStorage.setItem("data-table-" + tableKey, JSON.stringify(sorting));
  }, [tableKey, sorting]);

  useEffect(() => {
    selectOptions?.onSelect(Object.keys(rowSelection));
  }, [rowSelection]);

  return (
    <div
      className={cn(
        "rounded-md border bg-card text-card-foreground shadow py-1 px-1",
        containerClassName
      )}
    >
      <Table className="xl:table-fixed border-separate border-spacing-0">
        <TableHeader className="sticky top-0 z-40">
          {table.getHeaderGroups().map((headerGroup, i) => (
            <TableRow key={headerGroup.id}>
              {/* placeholder header */}
              {i === 0 && selectOptions && (
                <TableHead className="w-8 relative whitespace-nowrap bg-background border-b border-r last:border-r-0">
                  <Checkbox
                    className="ml-2"
                    disabled={selectOptions.disableRow === true}
                    checked={
                      table.getIsSomeRowsSelected()
                        ? "indeterminate"
                        : table.getIsAllRowsSelected()
                    }
                    onCheckedChange={() => table.toggleAllRowsSelected()}
                  />
                </TableHead>
              )}
              {headerGroup.headers.map((header) => {
                const size = header.column.getSize();
                return (
                  <TableHead
                    key={header.id}
                    colSpan={header.colSpan}
                    className="relative whitespace-nowrap bg-background border-b border-r last:border-r-0"
                    style={{ width: `${size}px` }}
                  >
                    {header.isPlaceholder
                      ? null
                      : flexRender(
                          header.column.columnDef.header,
                          header.getContext()
                        )}
                  </TableHead>
                );
              })}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {table.getRowModel().rows?.length ? (
            table.getRowModel().rows.map((row) => (
              <TableRow
                key={row.id}
                data-state={row.getIsSelected() && "selected"}
                onClick={() => onRowClick && onRowClick(row.original)}
                className={cn(
                  "even:bg-accent/25",
                  onRowClick && "cursor-pointer"
                )}
              >
                {selectOptions && (
                  <TableCell>
                    <Checkbox
                      disabled={!row.getCanSelect()}
                      className="ml-2"
                      checked={row.getIsSelected()}
                      onCheckedChange={(c) =>
                        c !== "indeterminate" && row.toggleSelected()
                      }
                    />
                  </TableCell>
                )}
                {row.getVisibleCells().map((cell) => {
                  const size = cell.column.getSize();
                  return (
                    <TableCell
                      key={cell.id}
                      className="p-4 overflow-hidden overflow-ellipsis"
                      style={{ width: `${size}px` }}
                    >
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext()
                      )}
                    </TableCell>
                  );
                })}
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={columns.length} className="p-4 text-center">
                {noResults ?? "No results."}
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}

export const SortableHeader = <T, V>({
  column,
  title,
  sortDescFirst,
}: {
  column: Column<T, V>;
  title: string;
  sortDescFirst?: boolean;
}) => (
  <div
    className="flex items-center justify-between"
    onClick={() => column.toggleSorting()}
  >
    {title}
    {column.getIsSorted() === "asc" ? (
      sortDescFirst ? (
        <ArrowUp className="w-4" />
      ) : (
        <ArrowDown className="w-4" />
      )
    ) : column.getIsSorted() === "desc" ? (
      sortDescFirst ? (
        <ArrowDown className="w-4" />
      ) : (
        <ArrowUp className="w-4" />
      )
    ) : (
      <Minus className="w-4" />
    )}
  </div>
);
