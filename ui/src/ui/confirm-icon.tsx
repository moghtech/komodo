import { ICONS } from "@/theme/icons";
import {
  ActionIcon,
  ActionIconProps,
  createPolymorphicComponent,
  Loader,
} from "@mantine/core";
import { Check } from "lucide-react";
import {
  FocusEventHandler,
  forwardRef,
  MouseEventHandler,
  useState,
} from "react";

// https://mantine.dev/guides/polymorphic/#create-your-own-polymorphic-components

export interface ConfirmIconProps extends ActionIconProps {
  onClick?: MouseEventHandler<HTMLButtonElement>;
  onBlur?: FocusEventHandler<HTMLButtonElement>;
}

const ConfirmIcon = createPolymorphicComponent<"button", ConfirmIconProps>(
  forwardRef<HTMLButtonElement, ConfirmIconProps>(
    ({ children, onClick, onBlur, miw, loading, ...props }, ref) => {
      const [clickedOnce, setClickedOnce] = useState(false);
      return (
        <ActionIcon
          onClick={(e) => {
            e.stopPropagation();
            if (clickedOnce) {
              onClick?.(e);
              setClickedOnce(false);
            } else {
              setClickedOnce(true);
            }
          }}
          onBlur={(e) => {
            setClickedOnce(false);
            onBlur?.(e);
          }}
          onPointerDown={e => e.stopPropagation()}
          {...props}
          ref={ref}
        >
          {clickedOnce ? (
            <Check size="1rem" />
          ) : loading ? (
            <Loader color="white" size="1rem" />
          ) : (
            (children ?? <ICONS.Unknown size="1rem" />)
          )}
        </ActionIcon>
      );
    },
  ),
);

export default ConfirmIcon;
