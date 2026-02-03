import { Fragment, ReactNode, SetStateAction, useMemo } from "react";
import { MonacoLanguage } from "@/components/monaco";
import { ICONS } from "@/lib/icons";
import {
  Anchor,
  Box,
  Button,
  Flex,
  Group,
  ScrollArea,
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
  selector?: ReactNode;
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
  selector,
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

  const groupsComponent = useMemo(
    () =>
      groups.map(([group, groupArgs]) => {
        return (
          <Fragment key={group}>
            {/* <div className="xl:hidden sticky top-16 h-16 flex items-center justify-between bg-background z-10">
                  {section && <p className="uppercase text-2xl">{section}</p>}
                  <Select
                    onValueChange={(value) => (window.location.hash = value)}
                  >
                    <SelectTrigger className="w-32 capitalize xl:hidden">
                      <SelectValue placeholder="Go To" />
                    </SelectTrigger>
                    <SelectContent className="w-32">
                      {components[section]
                        .filter((item) => !item.hidden)
                        .map(({ label }) => (
                          <SelectItem
                            key={section + label}
                            value={section + label}
                            className="capitalize"
                          >
                            {label}
                          </SelectItem>
                        ))}
                    </SelectContent>
                  </Select>
                </div> */}

            {group && (
              <Text visibleFrom="xl" fz="h1" tt="uppercase" mt="xl">
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
                      bd="1px solid var(--mantine-color-accent-border-0)"
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
                            {description && <Text c="dimmed">{description}</Text>}
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
        {unsavedIndicator && <UnsavedChanges />}
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

  return (
    <ConfigLayout
      original={original}
      titleOther={titleOther}
      update={update}
      disabled={disabled}
      onConfirm={onConfirm}
      onReset={onReset}
      selector={selector}
      fileContentsLanguage={fileContentsLanguage}
    >
      {disableSidebar && (
        <>
          {groupsComponent}
          <Group justify="flex-end">
            <SaveOrReset unsavedIndicator />
          </Group>
        </>
      )}
      {!disableSidebar && (
        <Flex w="100%" gap="xl">
          {/** SIDEBAR (XL) */}
          <Box
            visibleFrom="xl"
            pos="relative"
            bdrs="md"
            style={{
              borderColor: "var(--mantine-color-accent-border-0)",
              borderStyle: "solid",
              borderTopWidth: 1,
              borderRightWidth: 1,
              borderBottomWidth: 0,
              borderLeftWidth: 0,
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
                  {groups.map(([group, groupArgs]) => (
                    <Stack key={group} gap="xs">
                      <Group justify="flex-end" mr="md" c="dimmed">
                        <Bookmark size="1rem" />
                        <Text tt="uppercase">{group || "GENERAL"}</Text>
                      </Group>
                      <Stack gap="0.25rem">
                        {groupArgs &&
                          groupArgs
                            .filter((groupArgs) => !groupArgs.hidden)
                            .map((groupArgs) => (
                              <Button
                                key={group + groupArgs.label}
                                variant="subtle"
                                c="inherit"
                                justify="flex-end"
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

          {/** CONTENT */}
          <Stack style={{ flexGrow: 1 }} mb="50vh" gap="lg">
            {groupsComponent}
            <Group justify="flex-end">
              <SaveOrReset unsavedIndicator />
            </Group>
          </Stack>
        </Flex>
      )}
    </ConfigLayout>
  );
}
