---
sidebar_label: Trust Custom CA Certificates
---

# Trust Custom CA Certificates

Komodo Core and Periphery may need to trust one or more custom certificate authorities when they
connect to services on private networks. Common examples are internal OIDC providers, self-hosted
git providers, and private container registries.

Starting in v2, both images run `update-ca-certificates` on startup. To add custom trust roots,
mount the certificate files into `/usr/local/share/ca-certificates`.

```yaml
services:
  core:
    volumes:
      - /path/to/root_ca1.crt:/usr/local/share/ca-certificates/root_ca1.crt
      - /path/to/root_ca2.crt:/usr/local/share/ca-certificates/root_ca2.crt
      - /path/to/custom-certs:/usr/local/share/ca-certificates

  periphery:
    volumes:
      - /path/to/root_ca1.crt:/usr/local/share/ca-certificates/root_ca1.crt
      - /path/to/root_ca2.crt:/usr/local/share/ca-certificates/root_ca2.crt
      - /path/to/custom-certs:/usr/local/share/ca-certificates
```

Mount the certificates in whichever containers need to trust the remote service:

- mount them in **Core** if Core talks to the service directly
- mount them in **Periphery** if host-side clone, build, or registry operations need that trust
- mount them in both when both sides connect to the same internal service

## Related Pages

- [Install Komodo](../setup/install-komodo.mdx)
- [Connect More Servers](../setup/connect-servers.mdx)
- [OIDC / OAuth2 Login](./oidc-oauth2.md)
- [Providers](../configuration/providers.md)
