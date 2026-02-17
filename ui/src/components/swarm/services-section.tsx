import { filterBySplit } from "@/lib/utils";
import { ICONS } from "@/theme/icons";
import { DataTable, SortableHeader } from "@/ui/data-table";
import Section, { SectionProps } from "@/ui/section";
import ShowHideButton from "@/ui/show-hide-button";
import { Group, TextInput } from "@mantine/core";
import { Types } from "komodo_client";
import SwarmResourceLink from "./link";

export interface SwarmServicesSectionProps extends SectionProps {
  id: string;
  services: Types.SwarmServiceListItem[];
  show?: boolean;
  setShow?: (show: boolean) => void;
  _search: [string, (search: string) => void];
}

export default function SwarmServicesSection({
  id,
  services,
  show = true,
  setShow,
  titleOther,
  _search,
  ...sectionProps
}: SwarmServicesSectionProps) {
  const filtered = filterBySplit(
    services,
    _search[0],
    (service) => service.Name ?? service.ID ?? "Unknown",
  );

  return (
    <Section
      titleOther={titleOther}
      title={!titleOther ? "Services" : undefined}
      icon={!titleOther ? <ICONS.SwarmService size="1.3rem" /> : undefined}
      actions={
        _search || setShow ? (
          <Group>
            {_search && (
              <TextInput
                placeholder="search..."
                leftSection={<ICONS.Search size="1rem" />}
                value={_search[0]}
                onChange={(e) => _search[1](e.target.value)}
                w={{ base: 200, lg: 300 }}
              />
            )}
            {setShow && <ShowHideButton show={show} setShow={setShow} />}
          </Group>
        ) : undefined
      }
      {...sectionProps}
    >
      {show && (
        <DataTable
          tableKey="swarm-services"
          data={filtered}
          columns={[
            {
              accessorKey: "Name",
              header: ({ column }) => (
                <SortableHeader column={column} title="Name" />
              ),
              cell: ({ row }) => (
                <SwarmResourceLink
                  type="Service"
                  swarmId={id}
                  resourceId={row.original.Name}
                  name={row.original.Name}
                />
              ),
              size: 200,
            },
            {
              accessorKey: "ID",
              header: ({ column }) => (
                <SortableHeader column={column} title="Id" />
              ),
              cell: ({ row }) => row.original.ID ?? "Unknown",
              size: 200,
            },
            {
              accessorKey: "UpdatedAt",
              header: ({ column }) => (
                <SortableHeader column={column} title="Updated" />
              ),
              cell: ({ row }) =>
                row.original.UpdatedAt
                  ? new Date(row.original.UpdatedAt).toLocaleString()
                  : "Unknown",
              size: 200,
            },
            {
              accessorKey: "CreatedAt",
              header: ({ column }) => (
                <SortableHeader column={column} title="Created" />
              ),
              cell: ({ row }) =>
                row.original.CreatedAt
                  ? new Date(row.original.CreatedAt).toLocaleString()
                  : "Unknown",
              size: 200,
            },
          ]}
        />
      )}
    </Section>
  );
}
