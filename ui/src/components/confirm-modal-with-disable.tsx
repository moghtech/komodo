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
  return (
    <ConfirmModal
      disableModal={disabled}
      onDoubleClick={(event) => {
        const target = event.target;
        const confirmation =
          target instanceof Element ? target.closest("b") : null;

        if (confirmation?.textContent !== props.confirmText) return;

        event.preventDefault();
        const selection = window.getSelection();
        if (!selection) return;

        const range = document.createRange();
        range.selectNodeContents(confirmation);
        selection.removeAllRanges();
        selection.addRange(range);
      }}
      {...props}
    />
  );
}
