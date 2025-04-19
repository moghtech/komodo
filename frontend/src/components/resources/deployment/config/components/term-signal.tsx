import { ConfigItem } from "@components/config/util";
import { Types } from "komodo_client";
import { Input } from "@ui/input";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@ui/select";
import { useToast } from "@ui/use-toast";
import { useEffect, useState } from "react";

export const DefaultTerminationSignal = ({
  arg,
  set,
  disabled,
}: {
  arg?: Types.TerminationSignal;
  set: (input: Partial<Types.DeploymentConfig>) => void;
  disabled: boolean;
}) => {
  return (
    <ConfigItem label="Default Termination Signal">
      <Select
        value={arg}
        onValueChange={(value) =>
          set({ termination_signal: value as Types.TerminationSignal })
        }
        disabled={disabled}
      >
        <SelectTrigger className="w-[200px]" disabled={disabled}>
          <SelectValue placeholder="Select Type" />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            {Object.values(Types.TerminationSignal)
              .reverse()
              .map((term_signal) => (
                <SelectItem
                  key={term_signal}
                  value={term_signal}
                  className="cursor-pointer"
                >
                  {term_signal}
                </SelectItem>
              ))}
          </SelectGroup>
        </SelectContent>
      </Select>
    </ConfigItem>
  );
};

export const TerminationTimeout = ({
  arg,
  set,
  disabled,
}: {
  arg: number;
  set: (input: Partial<Types.DeploymentConfig>) => void;
  disabled: boolean;
}) => {
  const { toast } = useToast();
  const [input, setInput] = useState(arg.toString());
  useEffect(() => {
    setInput(arg.toString());
  }, [arg]);
  return (
    <ConfigItem label="Termination Timeout">
      <div className="flex items-center gap-4">
        <Input
          className="w-[100px]"
          placeholder="time in seconds"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onBlur={(e) => {
            const num = Number(e.target.value);
            if (num || num === 0) {
              set({ termination_timeout: num });
            } else {
              toast({ title: "Termination timeout must be a number" });
              setInput(arg.toString());
            }
          }}
          disabled={disabled}
        />
        seconds
      </div>
    </ConfigItem>
  );
};