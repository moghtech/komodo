import { Fragment, ReactNode, SetStateAction, useMemo } from "react";
import { MonacoLanguage } from "@/components/monaco";
import { ICONS } from "@/theme/icons";
import {
  Anchor,
  Box,
  Button,
  Flex,
  Group,
  ScrollArea,
  Select,
  Stack,
  Text,
} from "@mantine/core";
import ConfirmUpdate from "./confirm";
import { Bookmark } from "lucide-react";
import ConfigGroup from "./group";
import UnsavedChanges from "./unsaved-changes";
import ConfigLayout from "./layout";

export interface ConfigFieldArgs {
  label?: string;
  boldLabel?: boolean;
  description?: ReactNode;
  /** Use a selector instead of input */
  options?: { value: string; label?: string; icon?: ReactNode }[];
  placeholder?: string;
  hidden?: boolean;
  disabled?: boolean;
}

export interface ConfigGroupArgs<T> {
  label: string;
  boldLabel?: boolean; // defaults to true
  labelExtra?: ReactNode;
  icon?: ReactNode;
  description?: ReactNode;
  actions?: ReactNode;
  hidden?: boolean;
  labelHidden?: boolean;
  contentHidden?: boolean;
  fields: {
    [K in keyof Partial<T>]:
      | boolean
      | ConfigFieldArgs
      | ((value: T[K], set: (value: Partial<T>) => void) => ReactNode);
  };
}

export interface ConfigProps<T> {
  original: T;
  update: Partial<T>;
  setUpdate: React.Dispatch<SetStateAction<Partial<T>>>;
  disabled: boolean;
  onSave: () => Promise<unknown>;
  titleOther?: ReactNode;
  disableSidebar?: boolean;
  fileContentsLanguage?: MonacoLanguage;
  groups: Record<
    string, // Section key
    ConfigGroupArgs<T>[] | false | undefined
  >;
}

export default function Config<T>({
  original,
  update,
  setUpdate,
  disabled,
  onSave,
  titleOther,
  disableSidebar,
  fileContentsLanguage,
  groups: _groups,
}: ConfigProps<T>) {
  const changesMade = Object.keys(update).length ? true : false;
  const onConfirm = async () => {
    await onSave();
    setUpdate({});
  };
  const onReset = () => setUpdate({});

  const groups = useMemo(
    () => Object.entries(_groups).filter(([_, groupArgs]) => !!groupArgs),
    [_groups],
  );

  const GroupsComponent = useMemo(
    () =>
      groups.map(([group, groupArgs]) => {
        return (
          <Fragment key={group}>
            {group && (
              <Text visibleFrom="lg" fz="h1" tt="uppercase" mt="xl">
                {group}
              </Text>
            )}

            <Stack gap="xl">
              {(groupArgs as ConfigGroupArgs<T>[])
                .filter(({ hidden }) => !hidden)
                .map(
                  ({
                    label,
                    boldLabel = true,
                    labelHidden,
                    icon,
                    labelExtra,
                    actions,
                    description,
                    contentHidden,
                    fields,
                  }) => (
                    <Stack
                      key={group + label}
                      id={group + label}
                      p="xl"
                      gap="xl"
                      className="bordered-light"
                      bdrs="md"
                      style={{ scrollMarginTop: 94 }}
                    >
                      {!labelHidden && (
                        <Group justify="space-between">
                          <Stack gap="0">
                            <Group>
                              {icon}
                              <Text fw={boldLabel ? "bold" : undefined} fz="h2">
                                {label}
                              </Text>
                              {labelExtra}
                            </Group>
                            {description && (
                              <Text c="dimmed">{description}</Text>
                            )}
                          </Stack>
                          {actions}
                        </Group>
                      )}
                      {!contentHidden && (
                        <ConfigGroup
                          config={original}
                          update={update}
                          setUpdate={(u) => setUpdate((p) => ({ ...p, ...u }))}
                          fields={fields}
                          disabled={disabled}
                        />
                      )}
                    </Stack>
                  ),
                )}
            </Stack>
          </Fragment>
        );
      }),
    [groups],
  );

  const SaveOrReset = ({
    unsavedIndicator,
    fullWidth,
  }: {
    unsavedIndicator?: boolean;
    fullWidth?: boolean;
  }) =>
    changesMade && (
      <>
        {unsavedIndicator && <UnsavedChanges fullWidth={fullWidth} />}
        <Button
          variant="outline"
          onClick={onReset}
          disabled={disabled || !changesMade}
          leftSection={<ICONS.History size="1rem" />}
          fullWidth={fullWidth}
          w={fullWidth ? undefined : 100}
        >
          Reset
        </Button>
        <ConfirmUpdate
          previous={original}
          content={update}
          onConfirm={onConfirm}
          disabled={disabled}
          fileContentsLanguage={fileContentsLanguage}
          fullWidth={fullWidth}
        />
      </>
    );

  const SaveOrResetComponent = changesMade && (
    <>
      <Group visibleFrom="xs" justify="flex-end">
        <SaveOrReset unsavedIndicator />
      </Group>
      <Stack hiddenFrom="xs">
        <SaveOrReset unsavedIndicator fullWidth />
      </Stack>
    </>
  );

  return (
    <ConfigLayout titleOther={titleOther} SaveOrReset={SaveOrResetComponent}>
      {disableSidebar && (
        <>
          {GroupsComponent}
          {SaveOrResetComponent}
        </>
      )}
      {!disableSidebar && (
        <Flex
          w="100%"
          gap={{ base: "md", lg: "xl" }}
          direction={{ base: "column", lg: "row" }}
        >
          {/** SIDEBAR (LG) */}
          <Box
            visibleFrom="lg"
            pos="relative"
            className="bordered-light"
            style={{
              borderLeftWidth: 0,
              borderBottomWidth: 0,
              borderTopRightRadius: "var(--mantine-radius-md)",
            }}
          >
            <Stack pos="sticky" w={175} top={94} pb={24} m="lg">
              {/** ANCHORS */}
              <ScrollArea
                mah={
                  changesMade ? "calc(100vh - 220px)" : "calc(100vh - 130px)"
                }
              >
                <Stack>
                  {groups
                    .filter(([_, groupArgs]) => groupArgs)
                    .map(([group, groupArgs]) => (
                      <Stack key={group} gap="xs">
                        <Group justify="flex-end" mr="md" c="dimmed">
                          <Bookmark size="1rem" />
                          <Text tt="uppercase">{group || "GENERAL"}</Text>
                        </Group>
                        <Stack gap="0.1rem">
                          {groupArgs &&
                            groupArgs
                              .filter((groupArgs) => !groupArgs.hidden)
                              .map((groupArgs) => (
                                <Button
                                  key={group + groupArgs.label}
                                  variant="subtle"
                                  justify="flex-end"
                                  size="sm"
                                  fullWidth
                                  renderRoot={(props) => (
                                    <Anchor
                                      href={"#" + group + groupArgs.label}
                                      {...props}
                                    />
                                  )}
                                >
                                  {groupArgs.label}
                                </Button>
                              ))}
                        </Stack>
                      </Stack>
                    ))}
                </Stack>
              </ScrollArea>

              {/** SAVE */}
              <SaveOrReset fullWidth />
            </Stack>
          </Box>

          {/** SELECTOR (MOBILE) */}
          <Select
            hiddenFrom="lg"
            className="select-nondimmed-placeholder"
            placeholder="Go To"
            leftSection={<Bookmark size="1rem" />}
            value={null}
            data={groups
              .filter(([_, groupArgs]) => groupArgs)
              .map(([group, groupArgs]) => ({
                group,
                items: (groupArgs as ConfigGroupArgs<T>[]).map((arg) => ({
                  label: arg.label,
                  value: group + arg.label,
                })),
              }))}
            onChange={(group) => {
              if (!group) return;
              window.location.hash = group;
            }}
          />

          {/** CONTENT */}
          <Stack style={{ flexGrow: 1 }} gap="md">
            {GroupsComponent}
            {SaveOrResetComponent}
          </Stack>
        </Flex>
      )}
    </ConfigLayout>
  );
}
