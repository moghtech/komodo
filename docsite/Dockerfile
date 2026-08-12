FROM node:22.12-alpine AS builder

ARG DOCSITE_URL
ARG DOCSITE_BASE_URL=/

WORKDIR /builder

COPY package.json yarn.lock ./
RUN yarn install --frozen-lockfile

COPY ./docs ./docs
COPY ./src ./src
COPY ./static ./static
COPY ./babel.config.js ./babel.config.js
COPY ./docusaurus.config.ts ./docusaurus.config.ts
COPY ./sidebars.ts ./sidebars.ts
COPY ./tsconfig.json ./tsconfig.json

RUN DOCSITE_URL=${DOCSITE_URL} DOCSITE_BASE_URL=${DOCSITE_BASE_URL} yarn build

FROM caddy:latest

ARG DOCSITE_BASE_URL=/

# Copy into caddy image /srv, nested under the base url path
COPY --from=builder /builder/build /srv${DOCSITE_BASE_URL}

# Add a basic caddyfile serving http.
# BASE is the base url without its trailing slash (empty when base url is "/"),
# used to redirect e.g. /komodo -> /komodo/ so relative urls resolve correctly.
RUN BASE="${DOCSITE_BASE_URL%/}" && cat > /etc/caddy/Caddyfile <<EOF
:80 {
    root * /srv
    ${BASE:+redir $BASE $BASE/ 308}
    file_server
    try_files {path} {path}.html ${DOCSITE_BASE_URL}index.html
}
EOF

EXPOSE 80