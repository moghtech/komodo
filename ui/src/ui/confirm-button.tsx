import { ICONS } from "@/theme/icons";
import {
  Button,
  ButtonProps,
  createPolymorphicComponent,
  Loader,
} from "@mantine/core";
import { Check } from "lucide-react";
import {
  FocusEventHandler,
  forwardRef,
  MouseEventHandler,
  ReactNode,
  useState,
} from "react";

// https://mantine.dev/guides/polymorphic/#create-your-own-polymorphic-components

export interface ConfirmButtonProps extends ButtonProps {
  icon?: ReactNode;
  onClick?: MouseEventHandler<HTMLButtonElement>;
  onBlur?: FocusEventHandler<HTMLButtonElement>;
}

const ConfirmButton = createPolymorphicComponent<"button", ConfirmButtonProps>(
  forwardRef<HTMLButtonElement, ConfirmButtonProps>(
    (
      { icon, rightSection, children, onClick, onBlur, miw, loading, ...props },
      ref,
    ) => {
      const [clickedOnce, setClickedOnce] = useState(false);
      return (
        <Button
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
          justify="space-between"
          w={{ base: "100%", xs: 190 }}
          miw="fit-content"
          rightSection={
            clickedOnce ? (
              <Check size="1rem" />
            ) : loading ? (
              <Loader color="white" size="1rem" />
            ) : (
              (rightSection ?? icon ?? <ICONS.Unknown size="1rem" />)
            )
          }
          {...props}
          ref={ref}
        >
          {clickedOnce ? "Confirm" : children}
        </Button>
      );
    },
  ),
);

export default ConfirmButton;
