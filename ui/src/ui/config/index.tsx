import { Fragment, ReactNode, SetStateAction, useMemo } from "react";
import { MonacoLanguage } from "@/components/monaco";
import { ICONS } from "@/lib/icons";
import Section, { SectionProps } from "@/ui/section";
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
import { ConfigInput, ConfigSwitch } from "./item";
import { Bookmark } from "lucide-react";

/** Includes save buttons */
export function ConfigLayout<T>({
  original,
  update,
  disabled,
  onConfirm,
  onReset,
  selector,
  fileContentsLanguage,
  title,
  icon,
  titleOther,
  ...sectionProps
}: {
  original: T;
  update: Partial<T>;
  disabled: boolean;
  onConfirm: () => Promise<void>;
  onReset: () => void;
  selector?: ReactNode;
  fileContentsLanguage?: MonacoLanguage;
} & SectionProps) {
  const titleProps = titleOther
    ? { titleOther }
    : {
        title: title ?? "Config",
        icon: icon ?? <ICONS.Settings size="1rem" />,
      };
  const changesMade = Object.keys(update).length ? true : false;
  return (
    <Section
      actions={
        <Group>
          {changesMade && <UnsavedChanges />}
          {selector}
          {changesMade && (
            <>
              <Button
                variant="outline"
                onClick={onReset}
                disabled={disabled || !changesMade}
                leftSection={<ICONS.History size="1rem" />}
                w={100}
              >
                Reset
              </Button>
              <ConfirmUpdate
                previous={original}
                content={update}
                onConfirm={onConfirm}
                disabled={disabled}
                fileContentsLanguage={fileContentsLanguage}
              />
            </>
          )}
        </Group>
      }
      {...titleProps}
      {...sectionProps}
    />
  );
}

const UnsavedChanges = () => (
  <Group>
    <ICONS.Alert size="1rem" />
    Unsaved changes
    <ICONS.Alert size="1rem" />
  </Group>
);

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

export function Config<T>({
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
}: {
  original: T;
  update: Partial<T>;
  setUpdate: React.Dispatch<SetStateAction<Partial<T>>>;
  disabled: boolean;
  onSave: () => Promise<void>;
  titleOther?: ReactNode;
  selector?: ReactNode;
  disableSidebar?: boolean;
  fileContentsLanguage?: MonacoLanguage;
  groups: Record<
    string, // Section key
    ConfigGroupArgs<T>[] | false | undefined
  >;
}) {
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
              <Text visibleFrom="xl" fz="h1" tt="uppercase" mb="sm">
                {group}
              </Text>
            )}

            <Stack>
              {(groupArgs as ConfigGroupArgs<T>[]).map(
                ({
                  label,
                  boldLabel = true,
                  labelHidden,
                  icon,
                  labelExtra,
                  actions,
                  description,
                  hidden,
                  contentHidden,
                  fields,
                }) => (
                  <Stack
                    key={group + label}
                    id={group + label}
                    hidden={hidden}
                    p="xl"
                    gap="xs"
                    bd="1px solid var(--mantine-color-accent-border-0)"
                    bdrs="md"
                    style={{ scrollMarginTop: 94 }}
                  >
                    {!labelHidden && (
                      <Group justify="space-between">
                        <Stack>
                          <Group>
                            {icon}
                            <Text fw={boldLabel ? "bold" : undefined} fz="h2">
                              {label}
                            </Text>
                            {labelExtra}
                          </Group>
                          {description && <Text></Text>}
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

  const SaveOrReset = ({ unsavedIndicator }: { unsavedIndicator?: boolean }) =>
    changesMade && (
      <>
        {unsavedIndicator && <UnsavedChanges />}
        <Button
          variant="outline"
          onClick={onReset}
          disabled={disabled || !changesMade}
          leftSection={<ICONS.History size="1rem" />}
          w={100}
        >
          Reset
        </Button>
        <ConfirmUpdate
          previous={original}
          content={update}
          onConfirm={onConfirm}
          disabled={disabled}
          fileContentsLanguage={fileContentsLanguage}
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
        <Flex w="100%" gap="lg">
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
                renderRoot={(props) => <Stack {...props} />}
                mah={changesMade ? "calc(100vh-220px)" : "calc(100vh-130px)"}
              >
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
              </ScrollArea>

              {/** SAVE */}
              <SaveOrReset />
            </Stack>
          </Box>

          {/** CONTENT */}
          <Stack style={{ flexGrow: 1 }}>
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

function ConfigGroup<T>({
  config,
  update,
  setUpdate,
  disabled,
  fields,
}: {
  config: T;
  update: Partial<T>;
  setUpdate: React.Dispatch<SetStateAction<Partial<T>>>;
  disabled: boolean;
  fields: ConfigGroupArgs<T>["fields"];
}) {
  return Object.entries(fields).map(([key, field]) => {
    const value =
      (update as { [key: string]: unknown })[key] ??
      (config as { [key: string]: unknown })[key];
    if (typeof field === "function") {
      return <Fragment key={key}>{field(value, setUpdate)}</Fragment>;
    } else if (typeof field === "object" || field === true) {
      const args =
        typeof field === "object" ? (field as ConfigFieldArgs) : undefined;

      if (args?.hidden) {
        return null;
      }

      switch (typeof value) {
        case "string":
          return (
            <ConfigInput
              key={key}
              label={args?.label ?? key}
              value={value}
              onChange={(value) => setUpdate({ [key]: value } as Partial<T>)}
              disabled={args?.disabled || disabled}
              placeholder={args?.placeholder}
              description={args?.description}
              boldLabel={args?.boldLabel}
            />
          );

        case "number":
          return (
            <ConfigInput
              key={key}
              label={args?.label ?? key}
              value={Number(value)}
              onChange={(value) =>
                setUpdate({ [key]: Number(value) } as Partial<T>)
              }
              disabled={args?.disabled || disabled}
              placeholder={args?.placeholder}
              description={args?.description}
              boldLabel={args?.boldLabel}
            />
          );

        case "boolean":
          return (
            <ConfigSwitch
              key={key}
              label={args?.label ?? key}
              value={value}
              onChange={(value) => setUpdate({ [key]: value } as Partial<T>)}
              disabled={args?.disabled || disabled}
              description={args?.description}
              boldLabel={args?.boldLabel}
            />
          );

        default:
          return (
            <Group>
              Config '{args?.label ?? key}': <ICONS.Unknown size="1rem" />
            </Group>
          );
      }
    } else {
      return <Fragment key={key} />;
    }
  });
}
