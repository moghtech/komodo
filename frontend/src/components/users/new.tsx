import { NewLayout } from "@components/layouts";
import { useInvalidate, useWrite } from "@lib/hooks";
import { Input } from "@ui/input";
import { useToast } from "@ui/use-toast";
import { useState } from "react";

export const NewUserGroup = () => {
  const { toast } = useToast();
  const inv = useInvalidate();
  const { mutateAsync } = useWrite("CreateUserGroup", {
    onSuccess: () => {
      inv(["ListUserGroups"]);
      toast({ title: "Created User Group" });
    },
  });
  const [name, setName] = useState("");
  return (
    <NewLayout
      entityType="User Group"
      onConfirm={() => mutateAsync({ name })}
      enabled={!!name}
      onOpenChange={() => setName("")}
    >
      <div className="grid md:grid-cols-2">
        Name
        <Input
          placeholder="user-group-name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
      </div>
    </NewLayout>
  );
};

export const NewServiceUser = () => {
  const { toast } = useToast();
  const inv = useInvalidate();
  const { mutateAsync } = useWrite("CreateServiceUser", {
    onSuccess: () => {
      inv(["ListUsers"]);
      toast({ title: "Created Service User" });
    },
  });
  const [username, setUsername] = useState("");
  return (
    <NewLayout
      entityType="Service User"
      onConfirm={() => mutateAsync({ username, description: "" })}
      enabled={!!username}
      onOpenChange={() => setUsername("")}
    >
      <div className="grid md:grid-cols-2">
        Username
        <Input
          placeholder="username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
        />
      </div>
    </NewLayout>
  );
};

export const NewLocalUser = () => {
  const { toast } = useToast();
  const inv = useInvalidate();
  const { mutateAsync } = useWrite("CreateLocalUser", {
    onSuccess: () => {
      inv(["ListUsers"]);
      toast({ title: "Created Local User" });
    },
  });
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [passwordConfirm, setPasswordConfirm] = useState("");
  return (
    <NewLayout
      entityType="Local User"
      configureLabel="unique credentials"
      onConfirm={async () => {
        if (
          username.length === 0 ||
          password.length === 0 ||
          password !== passwordConfirm
        ) {
          toast({ title: "Invalid user info", variant: "destructive" });
        }
        return await mutateAsync({ username, password });
      }}
      enabled={!!username && !!password && password === passwordConfirm}
      onOpenChange={() => {
        setUsername("");
        setPassword("");
        setPasswordConfirm("");
      }}
    >
      <div className="grid md:grid-cols-2 gap-2">
        Username
        <Input
          placeholder="username"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
        />
        Password
        <Input
          placeholder="password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
        Confirm Password
        <Input
          placeholder="confirm password"
          type="password"
          value={passwordConfirm}
          onChange={(e) => setPasswordConfirm(e.target.value)}
          className={
            !password
              ? undefined
              : password === passwordConfirm
                ? "border-green-500"
                : "border-red-500"
          }
        />
      </div>
    </NewLayout>
  );
};
