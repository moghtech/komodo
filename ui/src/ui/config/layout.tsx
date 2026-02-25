import { ReactNode } from "react";
import Section, { SectionProps } from "@/ui/section";
import { ICONS } from "@/theme/icons";

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
