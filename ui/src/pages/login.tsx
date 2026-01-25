import {
  komodo_client,
  useLogin,
  useLoginOptions,
  useUserInvalidate,
} from "@/lib/hooks";
import { sanitizeQuery } from "@/lib/utils";
import {
  Button,
  Center,
  Fieldset,
  Flex,
  Group,
  Loader,
  PasswordInput,
  Text,
  TextInput,
} from "@mantine/core";
import { useForm } from "@mantine/form";
import { notifications } from "@mantine/notifications";
import { MoghAuth } from "komodo_client";
import { KeyRound } from "lucide-react";
import { useState } from "react";

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

  // const noAuthConfigured =
  //   options !== undefined &&
  //   Object.values(options).every((value) => value === false);

  // const showSignUp = options !== undefined && !options.registration_disabled;

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

  const registration_disabled = options?.registration_disabled ?? true;

  return (
    <Center h="80vh">
      <Fieldset
        legend={
          <Flex gap="4rem" align="center" justify="space-between">
            <Group gap="sm">
              <img
                src="/mogh-512x512.png"
                width={42}
                height={42}
                alt="moghtech"
              />
              <Flex direction="column">
                <Text fz="h2" fw="450" lts="0.1rem">
                  KOMODO
                </Text>
                <Text size="md" opacity={0.6} mt={-8}>
                  Log In
                </Text>
              </Flex>
            </Group>
            <Flex gap="sm" align="center" wrap="wrap">
              {(
                [
                  [options?.oidc, "Oidc"],
                  [options?.github, "Github"],
                  [options?.google, "Google"],
                ] as Array<
                  [boolean | undefined, MoghAuth.Types.ExternalLoginProvider]
                >
              ).map(
                ([enabled, provider]) =>
                  enabled && (
                    <Button
                      key={provider}
                      onClick={() =>
                        komodo_client().auth.externalLogin(provider)
                      }
                      leftSection={
                        provider === "Oidc" ? (
                          <KeyRound size="1rem" />
                        ) : (
                          <img
                            src={`/icons/${provider.toLowerCase()}.svg`}
                            alt={provider}
                            style={{ width: "1rem", height: "1rem" }}
                          />
                        )
                      }
                      w={110}
                      disabled={secondFactorPending}
                    >
                      {provider}
                    </Button>
                  ),
              )}
            </Flex>
          </Flex>
        }
        component="form"
        onSubmit={
          totpIsPending
            ? totpForm.onSubmit((form) => completeTotpLogin(form))
            : (localForm.onSubmit((form) => login(form)) as any)
        }
        style={{ display: "flex", flexDirection: "column", gap: "1rem" }}
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
            <Flex gap="md" mt="sm" justify="flex-end">
              {!registration_disabled && (
                <Button
                  w={110}
                  variant="default"
                  onClick={localForm.onSubmit((form) => signup(form)) as any}
                  loading={signupPending}
                >
                  Sign Up
                </Button>
              )}
              <Button
                w={110}
                variant="filled"
                type="submit"
                loading={loginPending}
              >
                Log In
              </Button>
            </Flex>
          </>
        )}

        {passkeyIsPending && (
          <>
            <KeyRound size="1.5rem" />
            <Group>
              <Loader />
              <Text size="lg">Provide your passkey to finish login</Text>
            </Group>
          </>
        )}

        {totpIsPending && (
          <>
            <TextInput
              {...totpForm.getInputProps("code")}
              label={
                <Flex align="center" gap="sm">
                  <KeyRound size="1rem" />
                  2FA Code
                </Flex>
              }
              autoComplete="code"
              autoCapitalize="none"
              autoCorrect="off"
              autoFocus
            />
            <Flex justify="flex-end">
              <Button
                w={110}
                variant="filled"
                type="submit"
                loading={totpPending}
              >
                Log In
              </Button>
            </Flex>
          </>
        )}
      </Fieldset>
    </Center>
  );
}
