import {
  Dispatch,
  ReactNode,
  SetStateAction,
  useEffect,
  useState,
} from "react";
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
  Box,
  BoxProps,
  Center,
  Checkbox,
  Flex,
  Group,
  HoverCard,
  Loader,
  Table,
  TableProps,
  Text,
  UnstyledButton,
} from "@mantine/core";
import { ArrowDown, ArrowUp, Info, Minus } from "lucide-react";

export interface DataTableProps<TData, TValue = unknown> extends BoxProps {
  /** Unique key given to table so sorting can be remembered on local storage */
  tableKey: string;
  columns: (ColumnDef<TData, TValue> | false | undefined)[];
  data: TData[];
  loading?: boolean;
  onRowClick?: (row: TData) => void;
  noResults?: ReactNode;
  defaultSort?: SortingState;
  sortDescFirst?: boolean;
  selectOptions?: {
    selectKey: (row: TData) => string;
    onSelect?: (selected: string[]) => void;
    state?: [RowSelectionState, Dispatch<SetStateAction<RowSelectionState>>];
    disableRow?: boolean | ((row: Row<TData>) => boolean);
  };
  caption?: string;
  tableProps?: TableProps;
  noBox?: boolean;
}

export function DataTable<TData, TValue>({
  tableKey,
  columns,
  data,
  loading,
  onRowClick,
  noResults = <Text c="dimmed">No results</Text>,
  sortDescFirst = false,
  defaultSort = [],
  selectOptions,
  caption,
  tableProps,
  noBox,
  ...boxProps
}: DataTableProps<TData, TValue>) {
  const [sorting, setSorting] = useState<SortingState>(defaultSort);

  // intentionally not initialized to clear selected values on table mount
  // could add some prop for adding default selected state to preserve between mounts
  const _internalState = useState<RowSelectionState>({});
  const [rowSelection, setRowSelection] = selectOptions?.state
    ? selectOptions.state
    : _internalState;

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
    selectOptions?.onSelect?.(Object.keys(rowSelection));
  }, [rowSelection]);

  const rows = table.getPrePaginationRowModel().rows;

  const tableNode = (
    <Table
      borderColor="accent-border"
      captionSide="top"
      stickyHeader
      {...tableProps}
    >
      {caption ? <Table.Caption>{caption}</Table.Caption> : null}

      <Table.Thead>
        {table.getHeaderGroups().map((hg, i) => (
          <Table.Tr key={hg.id}>
            {i === 0 && selectOptions && (
              <Table.Th
                onClick={() =>
                  selectOptions.disableRow !== true &&
                  table.toggleAllRowsSelected()
                }
                style={{
                  cursor: "pointer",
                  borderColor: "var(--mantine-color-accent-border-0)",
                  borderWidth: 0,
                  borderRightWidth: 1,
                  borderStyle: "solid",
                }}
              >
                <Checkbox
                  disabled={selectOptions.disableRow === true}
                  checked={table.getIsAllRowsSelected()}
                  indeterminate={table.getIsSomeRowsSelected()}
                />
              </Table.Th>
            )}
            {hg.headers.map((header, i) => {
              // const canSort = header.column.getCanSort();
              // const sortState = header.column.getIsSorted();
              return (
                <Table.Th
                  key={header.id}
                  px="md"
                  style={{
                    cursor: "pointer",
                    borderColor: "var(--mantine-color-accent-border-0)",
                    borderWidth: 0,
                    borderRightWidth: i < hg.headers.length - 1 ? 1 : 0,
                    borderStyle: "solid",
                  }}
                >
                  {header.isPlaceholder ? null : (
                    <Text fw={600} size="sm" lineClamp={1}>
                      {flexRender(
                        header.column.columnDef.header,
                        header.getContext(),
                      )}
                    </Text>
                  )}
                </Table.Th>
              );
            })}
          </Table.Tr>
        ))}
      </Table.Thead>

      <Table.Tbody>
        {loading ? (
          <Table.Tr>
            <Table.Td
              colSpan={
                table.getAllLeafColumns().length + (selectOptions ? 1 : 0)
              }
            >
              <Group justify="center" py="lg">
                <Loader size="sm" />
              </Group>
            </Table.Td>
          </Table.Tr>
        ) : rows.length === 0 ? (
          <Table.Tr>
            <Table.Td
              colSpan={
                table.getAllLeafColumns().length + (selectOptions ? 1 : 0)
              }
            >
              <Group justify="center" py="lg">
                {noResults}
              </Group>
            </Table.Td>
          </Table.Tr>
        ) : (
          rows.map((row) => (
            <Table.Tr
              key={row.id}
              style={onRowClick ? { cursor: "pointer" } : undefined}
            >
              {selectOptions && (
                <Table.Td onClick={() => row.toggleSelected()}>
                  <Checkbox
                    aria-label="Select row"
                    disabled={!row.getCanSelect()}
                    checked={row.getIsSelected()}
                  />
                </Table.Td>
              )}
              {row.getVisibleCells().map((cell) => (
                <Table.Td
                  key={cell.id}
                  onClick={
                    onRowClick ? () => onRowClick(row.original) : undefined
                  }
                  style={{ flexWrap: "nowrap", textWrap: "nowrap" }}
                >
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </Table.Td>
              ))}
            </Table.Tr>
          ))
        )}
      </Table.Tbody>
    </Table>
  );

  if (noBox) {
    return tableNode;
  } else {
    return (
      <Box
        p="lg"
        pt="0"
        bd="1px solid var(--mantine-color-accent-border-0)"
        bdrs="md"
        w="100%"
        mah="50vh"
        style={{ overflow: "auto" }}
        {...boxProps}
      >
        {tableNode}
      </Box>
    );
  }
}

export const SortableHeader = <T, V>({
  column,
  title,
  description,
}: {
  column: Column<T, V>;
  title: string;
  description?: ReactNode;
}) => {
  return (
    <UnstyledButton
      onClick={column.getToggleSortingHandler()}
      style={{ width: "100%" }}
    >
      <Group justify="space-between" gap="sm" wrap="nowrap">
        <Flex align="center" gap="sm">
          <Text fw={600} size="sm" lineClamp={1}>
            {title}
          </Text>
          {description && (
            <HoverCard>
              <HoverCard.Target>
                <Info className="w-4 h-4" />
              </HoverCard.Target>
              <HoverCard.Dropdown>
                <Text>{description}</Text>
              </HoverCard.Dropdown>
            </HoverCard>
          )}
        </Flex>
        <Center>
          <SortIcon state={column.getIsSorted()} />
        </Center>
      </Group>
    </UnstyledButton>
  );
};

function SortIcon({ state }: { state: false | "asc" | "desc" }) {
  if (state === "asc") return <ArrowUp size={14} />;
  if (state === "desc") return <ArrowDown size={14} />;
  return <Minus size={14} />;
}
