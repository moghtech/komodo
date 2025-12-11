import "globals.css";
import ReactDOM from "react-dom/client";
import { ThemeProvider } from "@ui/theme";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Router } from "@router";
import { WebsocketProvider } from "@lib/socket";
import { Toaster } from "@ui/toaster";
// Run monaco setup
import "./monaco";
import { init_monaco } from "./monaco/init";

export const KOMODO_BASE_URL =
  import.meta.env.VITE_KOMODO_HOST ?? location.origin;
export const UPDATE_WS_URL =
  KOMODO_BASE_URL.replace("http", "ws") + "/ws/update";
const query_client = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

export const sanitize_query = () => {
  sanitize_query_inner(new URLSearchParams(location.search));
};

export const sanitize_query_inner = (search: URLSearchParams) => {
  search.delete("redeem_ready");
  search.delete("totp");
  search.delete("passkey");
  const query = search.toString();
  location.replace(
    `${location.origin}${location.pathname}${query.length ? "?" + query : ""}`
  );
};

// Don't need to await this to render.
init_monaco();

ReactDOM.createRoot(document.getElementById("root")!).render(
  // <React.StrictMode>
  <QueryClientProvider client={query_client}>
    <WebsocketProvider>
      <ThemeProvider>
        <Router />
        <Toaster />
      </ThemeProvider>
    </WebsocketProvider>
  </QueryClientProvider>
  // </React.StrictMode>
);
