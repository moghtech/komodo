import { ReactNode } from "react";
import { Section, SectionProps } from "mogh_ui";
import { ICONS } from "@/lib/icons";

/** Includes save buttons */
export default function ConfigLayout({
  title,
  icon,
  titleOther,
  SaveOrReset,
  ...sectionProps
}: {
  SaveOrReset: ReactNode | undefined;
} & SectionProps) {
  const titleProps = titleOther
    ? { titleOther }
    : {
        title: title ?? "Config",
        icon: icon ?? <ICONS.Settings size="1rem" />,
      };
  return <Section actions={SaveOrReset} gap="md" {...titleProps} {...sectionProps} />;
}
