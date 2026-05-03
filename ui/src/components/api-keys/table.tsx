import { ICONS } from "@/lib/icons";
import { ConfirmButton } from "mogh_ui";
import { CopyText } from "mogh_ui";
import { DataTable } from "mogh_ui";
import { Text } from "@mantine/core";
import { Types } from "komodo_client";

const ONE_DAY_MS = 1000 * 60 * 60 * 24;

export interface ApiKeysTableProps {
  keys: Types.ApiKey[];
  onDelete: (key: string) => void;
  deletePending: boolean;
  noBorder?: boolean;
}

export default function ApiKeysTable({
  keys,
  onDelete,
  deletePending,
  noBorder,
}: ApiKeysTableProps) {
  return (
    <DataTable
      noBorder={noBorder}
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
            return <CopyText content={key} label="API key" />;
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
          cell: ({ row }) => (
            <ConfirmButton
              icon={<ICONS.Delete size="1rem" />}
              onClick={() => onDelete(row.original.key)}
              loading={deletePending}
              confirmProps={{ variant: "filled", color: "red" }}
            >
              Delete
            </ConfirmButton>
          ),
        },
      ]}
    />
  );
}
