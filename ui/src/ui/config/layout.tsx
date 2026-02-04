import { MonacoLanguage } from "@/components/monaco";
import { ReactNode } from "react";
import Section, { SectionProps } from "@/ui/section";
import { ICONS } from "@/theme/icons";
import { Button, Group } from "@mantine/core";
import UnsavedChanges from "./unsaved-changes";
import ConfirmUpdate from "./confirm";

/** Includes save buttons */
export default function ConfigLayout<T>({
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
