import { Button } from "@ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@ui/card";
import { Input } from "@ui/input";
import { Label } from "@ui/label";
import {
  LOGIN_TOKENS,
  useAuth,
  useLoginOptions,
  useUserInvalidate,
} from "@lib/hooks";
import { useRef, useState } from "react";
import { ThemeToggle } from "@ui/theme";
import { KOMODO_BASE_URL, sanitize_query } from "@main";
import { KeyRound, X } from "lucide-react";
import { cn } from "@lib/utils";
import { Types } from "komodo_client";

type OauthProvider = "Github" | "Google" | "OIDC";

const login_with_oauth = (provider: OauthProvider) => {
  const _redirect = location.pathname.startsWith("/login")
    ? location.origin +
      (new URLSearchParams(location.search).get("backto") ?? "")
    : location.href;
  const redirect = encodeURIComponent(_redirect);
  location.replace(
    `${KOMODO_BASE_URL}/auth/${provider.toLowerCase()}/login?redirect=${redirect}`
  );
};

export default function Login({ twoFactorToken }: { twoFactorToken?: string }) {
  const options = useLoginOptions().data;
  const userInvalidate = useUserInvalidate();
  const formRef = useRef<HTMLFormElement>(null);
  const totpFormRef = useRef<HTMLFormElement>(null);
  const [totpPendingToken, setTotpPendingToken] = useState(twoFactorToken);

  // If signing in another user, need to redirect away from /login manually
  const maybeNavigate = location.pathname.startsWith("/login")
    ? () =>
        location.replace(
          new URLSearchParams(location.search).get("backto") ?? "/"
        )
    : undefined;

  const onSuccess = ({ user_id, jwt }: Types.JwtResponse) => {
    LOGIN_TOKENS.add_and_change(user_id, jwt);
    userInvalidate();
    maybeNavigate?.();
  };

  const { mutate: signup, isPending: signupPending } = useAuth(
    "SignUpLocalUser",
    {
      onSuccess,
    }
  );
  const { mutate: login, isPending: loginPending } = useAuth("LoginLocalUser", {
    onSuccess: ({ type, data }) => {
      switch (type) {
        case "Jwt":
          return onSuccess(data);
        case "TwoFactor":
          return setTotpPendingToken(data.token);
      }
    },
  });
  const { mutate: totp, isPending: totpPending } = useAuth(
    "CompleteTotpLogin",
    {
      onSuccess: (res) => {
        sanitize_query();
        onSuccess(res);
      },
    }
  );

  const getFormCredentials = () => {
    if (!formRef.current) return undefined;
    const fd = new FormData(formRef.current);
    const username = String(fd.get("username") ?? "");
    const password = String(fd.get("password") ?? "");
    return { username, password };
  };

  const handleLogin = () => {
    const creds = getFormCredentials();
    if (!creds) return;
    login(creds);
  };

  const handleSubmit = (e: any) => {
    e.preventDefault();
    handleLogin();
  };

  const handleSignUp = () => {
    const creds = getFormCredentials();
    if (!creds) return;
    signup(creds);
  };

  const getTotpFormCredentials = () => {
    if (!totpFormRef.current) return undefined;
    const fd = new FormData(totpFormRef.current);
    const code = String(fd.get("code") ?? "");
    return { code };
  };

  const handleTotpSubmit = (e: any) => {
    e.preventDefault();
    const creds = getTotpFormCredentials();
    if (!creds || !totpPendingToken) return;
    totp({ token: totpPendingToken, code: creds.code });
  };

  const no_auth_configured =
    options !== undefined &&
    Object.values(options).every((value) => value === false);

  const show_sign_up = options !== undefined && !options.registration_disabled;

  // Otherwise just standard login
  return (
    <div className="flex flex-col min-h-screen">
      <div className="container flex justify-end items-center h-16">
        <ThemeToggle />
      </div>
      <div
        className={cn(
          "flex justify-center items-center container",
          options?.local ? "mt-32" : "mt-64"
        )}
      >
        <Card className="w-full max-w-[500px] place-self-center">
          <CardHeader className="flex-row justify-between">
            <div className="flex gap-4 items-center">
              <img src="/komodo-512x512.png" className="w-[32px] h-[32px]" />
              <div>
                <CardTitle className="text-xl">Komodo</CardTitle>{" "}
                <CardDescription>Log In</CardDescription>
              </div>
            </div>
            {!totpPendingToken && (
              <div className="flex gap-2">
                {(
                  [
                    [options?.google, "Google"],
                    [options?.github, "Github"],
                    [options?.oidc, "OIDC"],
                  ] as Array<[boolean | undefined, OauthProvider]>
                ).map(
                  ([enabled, provider]) =>
                    enabled && (
                      <Button
                        key={provider}
                        variant="outline"
                        className="flex gap-2 px-3 items-center"
                        onClick={() => login_with_oauth(provider)}
                      >
                        {provider}
                        {provider === "OIDC" ? (
                          <KeyRound className="w-4 h-4" />
                        ) : (
                          <img
                            src={`/icons/${provider.toLowerCase()}.svg`}
                            alt={provider}
                            className="w-4 h-4"
                          />
                        )}
                      </Button>
                    )
                )}
                {no_auth_configured && (
                  <Button variant="destructive" size="icon">
                    {" "}
                    <X className="w-4 h-4" />{" "}
                  </Button>
                )}
              </div>
            )}
          </CardHeader>
          {options?.local && !totpPendingToken && (
            <form ref={formRef} onSubmit={handleSubmit} autoComplete="on">
              <CardContent className="flex flex-col justify-center w-full gap-4">
                <div className="flex flex-col gap-2">
                  <Label htmlFor="username">Username</Label>
                  <Input
                    id="username"
                    name="username"
                    autoComplete="username"
                    autoCapitalize="none"
                    autoCorrect="off"
                    autoFocus
                  />
                </div>
                <div className="flex flex-col gap-2">
                  <Label htmlFor="password">Password</Label>
                  <Input
                    id="password"
                    name="password"
                    type="password"
                    autoComplete="current-password"
                  />
                </div>
              </CardContent>
              <CardFooter className="flex gap-4 w-full justify-end">
                {show_sign_up && (
                  <Button
                    variant="outline"
                    type="button"
                    value="signup"
                    onClick={handleSignUp}
                    disabled={signupPending}
                  >
                    Sign Up
                  </Button>
                )}
                <Button
                  variant="default"
                  type="submit"
                  value="login"
                  disabled={loginPending}
                >
                  Log In
                </Button>
              </CardFooter>
            </form>
          )}
          {totpPendingToken && (
            <form
              ref={totpFormRef}
              onSubmit={handleTotpSubmit}
              autoComplete="on"
            >
              <CardContent className="flex flex-col justify-center w-full gap-4">
                <div className="flex flex-col gap-2">
                  <Label htmlFor="code">2FA Code</Label>
                  <Input
                    id="code"
                    name="code"
                    autoComplete="code"
                    autoCapitalize="none"
                    autoCorrect="off"
                    autoFocus
                  />
                </div>
              </CardContent>
              <CardFooter className="flex gap-4 w-full justify-end">
                <Button
                  variant="default"
                  type="submit"
                  value="login"
                  disabled={totpPending}
                >
                  Log In
                </Button>
              </CardFooter>
            </form>
          )}
          {no_auth_configured && (
            <CardContent className="w-full gap-2 text-muted-foreground text-sm">
              No login methods have been configured. See the
              <a
                href="https://github.com/moghtech/komodo/blob/main/config/core.config.toml"
                target="_blank"
                rel="noreferrer"
                className="text-sm py-0 px-1 underline"
              >
                example config
              </a>
              for information on configuring auth.
            </CardContent>
          )}
        </Card>
      </div>
    </div>
  );
}
