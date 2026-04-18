---
sidebar_label: Use a Reverse Proxy
---

<!-- markdownlint-disable-file first-line-heading no-inline-html -->

```mdx-code-block
import Tabs from "@theme/Tabs";
import TabItem from "@theme/TabItem";
```

# Use A Reverse Proxy With Core And Periphery

If Komodo Core runs behind a reverse proxy, treat Periphery access as a separate network decision.
Avoid public Periphery exposure when Core can talk to Periphery over a private path.

## Prefer A Private Network First

If you control both ends, prefer one of these before publishing Periphery at all:

- an outbound Periphery connection to Core
- a WireGuard or Tailscale overlay
- a private network segment where Core can reach Periphery directly

That keeps the Periphery listener off the public internet and makes IP restrictions much simpler.

## If Core Is Public But Periphery Should Not Be

When Core is exposed through a public hostname but Periphery connections should only come from
trusted internal ranges or specific external addresses, restrict access to the Periphery websocket
endpoint at `/ws/periphery`.

For example:

<Tabs groupId="reverse-proxy">
  <TabItem value="caddy" label="Caddyfile">

```caddyfile
(reject-ips) {
  @externalIp not remote_ip 192.168.0.0/16 12.34.56.78/32
  respond @externalIp 403
}

komodo.example.com {
  handle /ws/periphery {
    import reject-ips
    reverse_proxy komodo-core:9120
  }
  handle {
    reverse_proxy komodo-core:9120
  }
}
```

  </TabItem>
  <TabItem value="nginx" label="nginx.conf">

```nginx
server {
    listen 443 ssl http2;
    server_name komodo.example.com;

    location /ws/periphery {
        allow 192.168.0.0/16;
        allow 12.34.56.78/32;
        deny all;

        proxy_pass http://komodo-core:9120;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    location / {
        proxy_pass http://komodo-core:9120;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

  </TabItem>
</Tabs>

Your reverse proxy should set the `X-FORWARDED-HOST` header to the Komodo Core domain. Caddy does
this by default, and the Nginx example sets it explicitly.

## If Periphery Itself Must Be Reachable

If you must expose Periphery itself rather than only Core's websocket endpoint, reduce the exposure
as much as possible:

- bind Periphery to a specific interface with `PERIPHERY_BIND_IP`
- restrict source networks with `PERIPHERY_ALLOWED_IPS`
- keep TLS enabled
- prefer a reverse proxy with mTLS or an allowlist in front of the public edge

The protocol can still authenticate correctly over a public edge. Limiting exposure reduces the
number of systems that can reach the listener in the first place.

## Related Pages

- [Install Komodo](../setup/install-komodo.mdx)
- [Connect More Servers](../setup/connect-servers.mdx)
- [Periphery](../periphery.md)
- [Connect Periphery Without An Onboarding Key](./connect-periphery-without-onboarding-key.md)
