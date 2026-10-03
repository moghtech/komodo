# Build context is compose/oidc-dev (see oidc-dev.compose.yaml build.context),
# hence the oidc-mock/ prefix on every COPY path.
FROM node:22.12-alpine
WORKDIR /app
# COPY BOTH package.json and yarn.lock: --frozen-lockfile with only
# package.json in the image prints "No lockfile found" and falls back to a
# fresh resolve — silently floating the transitive tree. No `|| yarn install`
# fallback: with the lockfile present, a mismatch should FAIL the build
# loudly, not silently reinstall.
COPY oidc-mock/package.json oidc-mock/yarn.lock ./
RUN yarn install --frozen-lockfile
COPY oidc-mock/index.mjs ./
ENV MOCK_ISSUER=https://portal.oidctest.localhost:8443
EXPOSE 3344
CMD ["yarn", "start"]
