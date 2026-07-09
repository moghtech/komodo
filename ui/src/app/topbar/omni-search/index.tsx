import { ActionIcon, Badge, Button, Group, Switch, Text } from "@mantine/core";
import { Spotlight, spotlight } from "@mantine/spotlight";
import { useOmniSearch } from "./hooks";
import { ICONS } from "@/lib/icons";
import { useShiftKeyListener } from "mogh_ui";
import classes from "./index.module.scss";
import { useLocalStorage } from "@mantine/hooks";
import { useState } from "react";

export default function OmniSearch({}: {}) {
  // Gates all omni search queries, so nothing is fetched / polled
  // until the spotlight is actually opened.
  const [opened, setOpened] = useState(false);
  const { search, setSearch, actions } = useOmniSearch(opened);
  useShiftKeyListener("S", () => spotlight.open());
  const [clearOnClose, setClearOnClose] = useLocalStorage({
    key: "omnisearch.clearOnClose",
    defaultValue: true,
  });
  return (
    <>
      <ActionIcon
        variant="subtle"
        onClick={() => spotlight.open()}
        hiddenFrom="lg"
        size="xl"
      >
        <ICONS.Search size="1.3rem" />
      </ActionIcon>

      <Button
        justify="space-between"
        rightSection={
          <Badge color="accent.9" tt="lowercase" style={{ cursor: "pointer" }}>
            shift + s
          </Badge>
        }
        onClick={() => spotlight.open()}
        w={{ lg: 230, xl: 300, xl4: 400 }}
        visibleFrom="lg"
      >
        <Group>
          <ICONS.Search size="1rem" />
          Search
        </Group>
      </Button>

      <Spotlight.Root
        query={search}
        onQueryChange={setSearch}
        clearQueryOnClose={clearOnClose}
        onSpotlightOpen={() => setOpened(true)}
        onSpotlightClose={() => setOpened(false)}
        styles={{ content: { borderRadius: "var(--mantine-radius-sm)" } }}
      >
        <Spotlight.Search
          leftSection={<ICONS.Search size="1.3rem" />}
          placeholder="search..."
        />
        <Group justify="flex-end">
          <Group
            gap="xs"
            onClick={() => setClearOnClose((v) => !v)}
            p="0.5rem 1rem"
            style={{ cursor: "pointer" }}
          >
            <Text>Clear on close</Text>
            <Switch
              checked={clearOnClose}
              onClick={() => setClearOnClose((v) => !v)}
            />
          </Group>
        </Group>
        <Spotlight.ActionsList>
          {actions.map((group) => (
            <Spotlight.ActionsGroup key={group.group} label={group.group}>
              {group.actions.map((action) => (
                <Spotlight.Action
                  key={action.id}
                  className={classes["spotlight-action"]}
                  {...action}
                />
              ))}
            </Spotlight.ActionsGroup>
          ))}
          {actions.length === 0 && (
            <Spotlight.Empty>Nothing found...</Spotlight.Empty>
          )}
        </Spotlight.ActionsList>
      </Spotlight.Root>
    </>
  );
}
