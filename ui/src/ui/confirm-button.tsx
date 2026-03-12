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
  useEffect,
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
      {
        icon,
        rightSection,
        children,
        onClick,
        onBlur,
        miw,
        loading,
        disabled,
        ...props
      },
      ref,
    ) => {
      const [clickedOnce, setClickedOnce] = useState(false);
      useEffect(() => {
        if (clickedOnce) {
          const timeout = setTimeout(() => {
            setClickedOnce(false);
          }, 4_000);
          return () => clearTimeout(timeout);
        }
      }, [clickedOnce]);
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
            onBlur?.(e);
            setClickedOnce(false);
          }}
          justify="space-between"
          w={{ base: "100%", xs: 190 }}
          miw="fit-content"
          rightSection={
            clickedOnce ? (
              <Check size="1rem" />
            ) : loading ? (
              <Loader size="1rem" />
            ) : (
              (rightSection ?? icon ?? <ICONS.Unknown size="1rem" />)
            )
          }
          disabled={disabled || loading}
          {...props}
          {...(clickedOnce
            ? { variant: "filled", color: "red" }
            : {})}
          ref={ref}
        >
          {clickedOnce ? "Confirm" : children}
        </Button>
      );
    },
  ),
);

export default ConfirmButton;
