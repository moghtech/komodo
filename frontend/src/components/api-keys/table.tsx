import { CopyButton } from "@components/util";
import { Types } from "komodo_client";
import { DataTable } from "@ui/data-table";
import { Input } from "@ui/input";
import { ReactNode } from "react";

const ONE_DAY_MS = 1000 * 60 * 60 * 24;

export const ApiKeysTable = ({
  keys,
  DeleteKey,
}: {
  keys: Types.ApiKey[];
  DeleteKey: (params: { api_key: string }) => ReactNode;
}) => {
  return (
    <DataTable
      tableKey="api-keys"
      data={keys}
      columns={[
        { header: "Name", accessorKey: "name" },
        {
          header: "Key",
          cell: ({
            row: {
              original: { key },
            },
          }) => {
            return (
              <div className="flex items-center gap-2">
                <Input
                  className="w-[100px] lg:w-[200px] overflow-ellipsis"
                  value={key}
                  disabled
                />
                <CopyButton content={key} />
              </div>
            );
          },
        },
        {
          header: "Expires",
          accessorFn: ({ expires }) =>
            expires
              ? "In " +
                ((expires - Date.now()) / ONE_DAY_MS).toFixed() +
                " Days"
              : "Never",
        },
        {
          header: "Delete",
          cell: ({ row }) => <DeleteKey api_key={row.original.key} />,
        },
      ]}
    />
  );
};
