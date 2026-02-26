import { useLogin, useLoginOptions, useUserInvalidate } from "@/lib/hooks";
import { sanitizeQuery } from "@/lib/utils";
import {
  Button,
  Center,
  Fieldset,
  Group,
  Loader,
  PasswordInput,
  Text,
  TextInput,
} from "@mantine/core";
import { useForm } from "@mantine/form";
import { notifications } from "@mantine/notifications";
import { MoghAuth } from "komodo_client";
import { AlertTriangle, KeyRound } from "lucide-react";
import { useState } from "react";
import LoginHeader from "./header";

export default function Login({
  passkeyIsPending: _passkeyIsPending,
  totpIsPending: _totpIsPending,
}: {
  passkeyIsPending?: boolean;
  totpIsPending?: boolean;
}) {
  const options = useLoginOptions().data;
  const userInvalidate = useUserInvalidate();
  const [passkeyIsPending, setPasskeyPending] = useState(
    _passkeyIsPending ?? false,
  );
  const [totpIsPending, setTotpPending] = useState(_totpIsPending ?? false);
  const secondFactorPending = passkeyIsPending || totpIsPending;

  // If signing in another user, need to redirect away from /login manually
  const maybeNavigate = location.pathname.startsWith("/login")
    ? () =>
        location.replace(
          new URLSearchParams(location.search).get("backto") ?? "/",
        )
    : undefined;

  const onSuccess = ({ jwt }: MoghAuth.Types.JwtResponse) => {
    MoghAuth.LOGIN_TOKENS.add_and_change(jwt);
    userInvalidate();
    maybeNavigate?.();
  };

  const secondFactorOnSuccess = (res: MoghAuth.Types.JwtResponse) => {
    sanitizeQuery();
    onSuccess(res);
  };

  const { mutate: signup, isPending: signupPending } = useLogin(
    "SignUpLocalUser",
    {
      onSuccess,
    },
  );

  const { mutate: completePasskeyLogin } = useLogin("CompletePasskeyLogin", {
    onSuccess: secondFactorOnSuccess,
  });

  const { mutate: completeTotpLogin, isPending: totpPending } = useLogin(
    "CompleteTotpLogin",
    {
      onSuccess: secondFactorOnSuccess,
    },
  );

  const { mutate: login, isPending: loginPending } = useLogin(
    "LoginLocalUser",
    {
      onSuccess: ({ type, data }) => {
        switch (type) {
          case "Jwt":
            return onSuccess(data);
          case "Passkey":
            setPasskeyPending(true);
            return navigator.credentials
              .get(MoghAuth.Passkey.prepareRequestChallengeResponse(data))
              .then((credential) => completePasskeyLogin({ credential }))
              .catch((e) => {
                console.error(e);
                notifications.show({
                  title: "Failed to select passkey",
                  message: "See console for details",
                  color: "red",
                });
              });
          case "Totp":
            return setTotpPending(true);
        }
      },
    },
  );

  const noAuthConfigured =
    options !== undefined &&
    Object.values(options).every((value) => value === false);

  const showSignUp = options !== undefined && !options.registration_disabled;

  const localForm = useForm({
    mode: "uncontrolled",
    initialValues: {
      username: "",
      password: "",
    },
    validate: {
      username: (username) =>
        username.length ? null : "Username cannot be empty",
      password: (password) =>
        password.length ? null : "Password cannot be empty",
    },
  });

  const totpForm = useForm({
    mode: "uncontrolled",
    initialValues: {
      code: "",
    },
    validate: {
      code: (code) => (code.length === 6 ? null : "Code should be 6 digits"),
    },
  });

  return (
    <Center h="80vh">
      <Fieldset
        legend={<LoginHeader secondFactorPending={secondFactorPending} />}
        component="form"
        onSubmit={
          totpIsPending
            ? totpForm.onSubmit((form) => completeTotpLogin(form))
            : (localForm.onSubmit((form) => login(form)) as any)
        }
        style={{ display: "flex", flexDirection: "column", gap: "1rem" }}
        miw={{ lg: "500px" }}
        maw="95vw"
      >
        {options?.local && !secondFactorPending && (
          <>
            <TextInput
              {...localForm.getInputProps("username")}
              autoFocus
              label="Username"
              placeholder="Enter username"
              key={localForm.key("username")}
            />
            <PasswordInput
              {...localForm.getInputProps("password")}
              label="Password"
              placeholder="Enter password"
              key={localForm.key("password")}
            />
            <Group mt="sm" justify="end">
              {showSignUp && (
                <Button
                  variant="outline"
                  w={110}
                  onClick={localForm.onSubmit((form) => signup(form)) as any}
                  loading={signupPending}
                >
                  Sign Up
                </Button>
              )}
              <Button w={110} type="submit" loading={loginPending}>
                Log In
              </Button>
            </Group>
          </>
        )}

        {passkeyIsPending && (
          <Group justify="center" my="lg">
            <KeyRound size="1.5rem" />
            <Text size="lg">Provide your passkey to finish login...</Text>
            <Loader />
          </Group>
        )}

        {totpIsPending && (
          <>
            <TextInput
              {...totpForm.getInputProps("code")}
              label={
                <Group gap="sm">
                  <KeyRound size="1rem" />
                  2FA Code
                </Group>
              }
              autoComplete="code"
              autoCapitalize="none"
              autoCorrect="off"
              autoFocus
            />
            <Group justify="end">
              <Button
                w={110}
                variant="filled"
                type="submit"
                loading={totpPending}
              >
                Log In
              </Button>
            </Group>
          </>
        )}

        {noAuthConfigured && (
          <Group my="lg">
            <AlertTriangle size="2rem" />
            <Text>
              No login methods have been configured. <br />
              See the{" "}
              <a
                href="https://github.com/moghtech/komodo/blob/main/config/core.config.toml"
                target="_blank"
                rel="noreferrer"
                className="hover-underline"
              >
                <b>example config</b>
              </a>{" "}
              for information on configuring auth.
            </Text>
          </Group>
        )}
      </Fieldset>
    </Center>
  );
}
