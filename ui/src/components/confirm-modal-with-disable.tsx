import { useRead } from "@/lib/hooks";
import { ConfirmModal, ConfirmModalProps } from "mogh_ui";

export interface ConfirmModalWithDisableProps extends Omit<
  ConfirmModalProps,
  "disableModal"
> {}

export default function ConfirmModalWithDisable({
  ...props
}: ConfirmModalWithDisableProps) {
  const disabled = useRead("GetCoreInfo", {}).data?.disable_confirm_dialog;
  return <ConfirmModal disableModal={disabled} {...props} />;
}
