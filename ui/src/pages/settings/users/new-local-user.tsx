import { useInvalidate, useWrite } from "@/lib/hooks";
import CreateModal from "@/ui/create-modal";
import { PasswordInput, TextInput } from "@mantine/core";
import { notifications } from "@mantine/notifications";
import { useState } from "react";

export default function NewLocalUser() {
  const inv = useInvalidate();
  const { mutateAsync: create, isPending } = useWrite("CreateLocalUser", {
    onSuccess: () => {
      inv(["ListUsers"]);
      notifications.show({ message: "Created Local User", color: "green" });
    },
  });
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");

  return (
    <CreateModal
      entityType="Local User"
      onConfirm={() => create({ username, password }).then(() => true)}
      loading={isPending}
      configureLabel="login information"
      configSection={() => (
        <>
          <TextInput
            autoFocus
            label="Username"
            placeholder="username"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            error={!username.trim() && "Enter username"}
          />
          <PasswordInput
            label="Password"
            placeholder="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            error={!password.trim() && "Enter password"}
          />
          <PasswordInput
            label="Confirm Password"
            placeholder="confirm-password"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            error={
              password !== confirmPassword ? "Passwords don't match" : undefined
            }
          />
        </>
      )}
    />
  );
}
