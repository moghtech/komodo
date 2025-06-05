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
import { useAuth, useLoginOptions, useUserInvalidate } from "@lib/hooks";
import { useState } from "react";
import { ThemeToggle } from "@ui/theme";
import { AUTH_TOKEN_STORAGE_KEY, KOMODO_BASE_URL } from "@main";
import { KeyRound, Loader2, X } from "lucide-react";
import { cn } from "@lib/utils";
import { useToast } from "@ui/use-toast";

type OauthProvider = "Github" | "Google" | "OIDC";

const login_with_oauth = (provider: OauthProvider) => {
  const redirect = encodeURIComponent(location.href);
  location.replace(
    `${KOMODO_BASE_URL}/auth/${provider.toLowerCase()}/login?redirect=${redirect}`
  );
};

const sanitize_query = (search: URLSearchParams) => {
  search.delete("token");
  const query = search.toString();
  location.replace(
    `${location.origin}${location.pathname}${query.length ? "?" + query : ""}`
  );
};

let exchange_token_sent = false;

/// returns whether to show login / loading screen depending on state of exchange token loop
const useExchangeToken = () => {
  const search = new URLSearchParams(location.search);
  const exchange_token = search.get("token");
  const { mutate } = useAuth("ExchangeForJwt", {
    onSuccess: ({ jwt }) => {
      localStorage.setItem(AUTH_TOKEN_STORAGE_KEY, jwt);
      sanitize_query(search);
    },
  });

  // In this case, failed to get user (jwt unset / invalid)
  // and the exchange token is not in url.
  // Just show the login.
  if (!exchange_token) return false;

  // guard against multiple reqs sent
  // maybe isPending would do this but not sure about with render loop, this for sure will.
  if (!exchange_token_sent) {
    mutate({ token: exchange_token });
    exchange_token_sent = true;
  }

  return true;
};

export default function Login() {
  const options = useLoginOptions().data;
  const [creds, set] = useState({ username: "", password: "" });
  const userInvalidate = useUserInvalidate();
  const { toast } = useToast();
  const onSuccess = ({ jwt }: { jwt: string }) => {
    localStorage.setItem(AUTH_TOKEN_STORAGE_KEY, jwt);
    userInvalidate();
  };
  const { mutate: signup, isPending: signupPending } = useAuth(
    "CreateLocalUser",
    {
      onSuccess,
      onError: (e: any) => {
        const message = e?.response?.data?.error as string | undefined;
        if (message) {
          toast({
            title: `Failed to login user. '${message}'`,
            variant: "destructive",
          });
          console.error(e);
        } else {
          toast({
            title: "Failed to login user. See console log for details.",
            variant: "destructive",
          });
          console.error(e);
        }
      },
    }
  );
  const { mutate: login, isPending: loginPending } = useAuth("LoginLocalUser", {
    onSuccess,
    onError: (e: any) => {
      const message = e?.response?.data?.error as string | undefined;
      if (message) {
        toast({
          title: `Failed to login user. '${message}'`,
          variant: "destructive",
        });
        console.error(e);
      } else {
        toast({
          title: "Failed to login user. See console log for details.",
          variant: "destructive",
        });
        console.error(e);
      }
    },
  });

  // Handle exchange token loop to avoid showing login flash
  const exchangeTokenPending = useExchangeToken();
  if (exchangeTokenPending) {
    return (
      <div className="w-screen h-screen flex justify-center items-center">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  const no_auth_configured =
    options !== undefined &&
    Object.values(options).every((value) => value === false);

  const show_sign_up = options !== undefined && !options.registration_disabled;

  // Otherwise just standard login
  return (
    <div className="flex flex-col min-h-screen">
      <div className="container flex justify-end items-center h-16">
        {/* <img src="/komodo-512x512.png" className="w-[32px]" /> */}
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
          </CardHeader>
          {options?.local && (
            <>
              <CardContent className="flex flex-col justify-center w-full gap-4">
                <div className="flex flex-col gap-2">
                  <Label htmlFor="username">Username</Label>
                  <Input
                    id="username"
                    value={creds.username}
                    onChange={({ target }) =>
                      set((c) => ({ ...c, username: target.value }))
                    }
                  />
                </div>
                <div className="flex flex-col gap-2">
                  <Label htmlFor="password">Password</Label>
                  <Input
                    id="password"
                    type="password"
                    value={creds.password}
                    onChange={({ target }) =>
                      set((c) => ({ ...c, password: target.value }))
                    }
                    onKeyDown={(e) => e.key === "Enter" && login(creds)}
                  />
                </div>
              </CardContent>
              <CardFooter className="flex gap-4 w-full justify-end">
                {show_sign_up && (
                  <Button
                    variant="outline"
                    onClick={() => signup(creds)}
                    disabled={signupPending}
                  >
                    Sign Up
                  </Button>
                )}
                <Button
                  variant="default"
                  onClick={() => login(creds)}
                  disabled={loginPending}
                >
                  Log In
                </Button>
              </CardFooter>
            </>
          )}
          {no_auth_configured && (
            <CardContent className="w-full gap-2 text-muted-foreground text-sm">
              No login methods have been configured. See the
              <Button variant="link" className="text-sm py-0 px-1">
                <a
                  href="https://github.com/moghtech/komodo/blob/main/config/core.config.toml"
                  target="_blank"
                  className="flex text-sm"
                >
                  example config
                </a>
              </Button>
              for information on configuring auth.
            </CardContent>
          )}
        </Card>
      </div>
    </div>
  );
}
