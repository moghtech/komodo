# Komodo UI

Komodo UI uses Yarn + Vite + React + Mantine UI

## Setup Dev Environment

Before starting development, the ui dependencies must be installed first.

The following command should set everything up:
```sh
yarn
```

You can make a new file `.env.development` (gitignored) which holds:
```sh
VITE_KOMODO_HOST=https://demo.komo.do
```
You can point it to any Komodo host you like, including the demo.

Now you can start the dev ui server:
```sh
yarn workspace komodo-ui dev
```