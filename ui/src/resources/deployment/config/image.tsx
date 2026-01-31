import { fmtDate, fmtVersion } from "@/lib/formatting";
import { useRead } from "@/lib/hooks";
import { ICONS } from "@/lib/icons";
import { filterBySplit } from "@/lib/utils";
import ResourceSelector from "@/resources/selector";
import {
  Button,
  Combobox,
  Group,
  Select,
  Text,
  TextInput,
  useCombobox,
} from "@mantine/core";
import { Types } from "komodo_client";
import { ChevronsUpDown } from "lucide-react";
import { useEffect, useState } from "react";

export interface DeploymentImageConfigProps {
  image: Types.DeploymentImage | undefined;
  setUpdate: (input: Partial<Types.DeploymentConfig>) => void;
  disabled: boolean;
}

export default function DeploymentImageConfig({
  image,
  setUpdate,
  disabled,
}: DeploymentImageConfigProps) {
  return (
    <Group>
      <ImageTypeSelector
        selected={image?.type}
        disabled={disabled}
        onSelect={(type) =>
          setUpdate({
            image: {
              type: type,
              params:
                type === "Image"
                  ? { image: "" }
                  : ({
                      build_id: "",
                      version: { major: 0, minor: 0, patch: 0 },
                    } as any),
            },
          })
        }
      />
      {image?.type === "Build" && (
        <>
          <ResourceSelector
            type="Build"
            selected={image.params.build_id}
            onSelect={(id) =>
              setUpdate({
                image: {
                  ...image,
                  params: { ...image.params, build_id: id },
                },
              })
            }
            disabled={disabled}
          />
          <BuildVersionSelector
            buildId={image.params.build_id}
            selected={image.params.version}
            onSelect={(version) =>
              setUpdate({
                image: {
                  ...image,
                  params: {
                    ...image.params,
                    version,
                  },
                },
              })
            }
            disabled={disabled}
          />
        </>
      )}
      {image?.type === "Image" && (
        <TextInput
          value={image.params.image}
          onChange={(e) =>
            setUpdate({
              image: {
                ...image,
                params: { image: e.target.value },
              },
            })
          }
          placeholder="image name"
          disabled={disabled}
          w={{ base: 200, lg: 300, xl: 400 }}
        />
      )}
    </Group>
  );
}

function ImageTypeSelector({
  selected,
  onSelect,
  disabled,
}: {
  selected: Types.DeploymentImage["type"] | undefined;
  onSelect: (type: Types.DeploymentImage["type"]) => void;
  disabled: boolean;
}) {
  return (
    <Select
      disabled={disabled}
      value={selected || undefined}
      onChange={(selected) =>
        selected && onSelect(selected as Types.DeploymentImage["type"])
      }
      data={["Image", "Build"]}
      w={100}
    />
  );
}

function BuildVersionSelector({
  disabled,
  buildId,
  selected,
  onSelect,
}: {
  disabled: boolean;
  buildId: string | undefined;
  selected: Types.Version | undefined;
  onSelect: (version: Types.Version) => void;
}) {
  const [search, setSearch] = useState("");
  const versions = useRead(
    "ListBuildVersions",
    { build: buildId! },
    { enabled: !!buildId },
  ).data;
  const combobox = useCombobox({
    onDropdownOpen: () => {
      combobox.focusSearchInput();
    },
    onDropdownClose: () => {
      combobox.resetSelectedOption();
      combobox.focusTarget();
      setSearch("");
    },
  });
  useEffect(() => {
    combobox.selectFirstOption();
  }, [search]);
  const filtered = filterBySplit(versions, search, (item) =>
    fmtVersion(item.version),
  );
  return (
    <Combobox
      store={combobox}
      disabled={disabled}
      onOptionSubmit={() => {
        combobox.closeDropdown();
      }}
      width={250}
      position="bottom-start"
    >
      <Combobox.Target>
        <Button
          onClick={() => combobox.openDropdown()}
          rightSection={<ChevronsUpDown size="0.9rem" />}
          disabled={disabled}
        >
          {selected ? fmtVersion(selected) : "Latest"}
        </Button>
      </Combobox.Target>

      <Combobox.Dropdown>
        <Combobox.Search
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          leftSection={<ICONS.Search size="1rem" />}
          placeholder="Search"
          styles={{
            section: {
              marginRight: 4,
            },
          }}
        />
        <Combobox.Options mah={224} style={{ overflowY: "auto" }}>
          <Combobox.Option
            value="Latest"
            onSelect={() => onSelect({ major: 0, minor: 0, patch: 0 })}
          >
            Latest
          </Combobox.Option>
          {filtered.map((v) => {
            const version = fmtVersion(v.version);
            return (
              <Combobox.Option key={version} value={version}>
                <Group justify="space-between" wrap="nowrap">
                  <Text>{version}</Text>
                  <Text c="dimmed">{fmtDate(new Date(v.ts))}</Text>
                </Group>
              </Combobox.Option>
            );
          })}
        </Combobox.Options>
      </Combobox.Dropdown>
    </Combobox>
  );
}
