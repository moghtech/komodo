import CopyText from "@/ui/copy-text";
import { DataTable } from "@/ui/data-table";
import { Text } from "@mantine/core";
import { Types } from "komodo_client";
import { ReactNode } from "react";

const ONE_DAY_MS = 1000 * 60 * 60 * 24;

export interface ApiKeysTableProps {
  keys: Types.ApiKey[];
  DeleteKey: (params: { key: string }) => ReactNode;
}

export default function ApiKeysTable({ keys, DeleteKey }: ApiKeysTableProps) {
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
            return <CopyText content={key} label="api key" />;
          },
        },
        {
          header: "Expires",
          cell: ({ row }) => {
            return (
              <Text>
                {row.original.expires ? (
                  <>
                    In{" "}
                    <b>
                      {(
                        (row.original.expires - Date.now()) /
                        ONE_DAY_MS
                      ).toFixed()}
                    </b>{" "}
                    Days
                  </>
                ) : (
                  <b>Never</b>
                )}
              </Text>
            );
          },
        },
        {
          header: "Delete",
          cell: ({ row }) => <DeleteKey key={row.original.key} />,
        },
      ]}
    />
  );
}
