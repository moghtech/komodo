FROM node:22.12-alpine AS builder

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

RUN yarn build

FROM caddy:latest

# Copy into caddy image /srv
COPY --from=builder /builder/build /srv

# Add a basic caddyfile serving http
RUN <<EOF cat > /etc/caddy/Caddyfile
:80 {
    root * /srv
    file_server
    try_files {path} /index.html
}
EOF

EXPOSE 80