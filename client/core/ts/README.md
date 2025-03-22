# Komodo

_A system to build and deploy software across many servers_. [https://komo.do](https://komo.do)

```sh
npm install komodo_client
```

or

```sh
yarn add komodo_client
```

```ts
import { KomodoClient, Types } from "komodo_client";

const komodo = KomodoClient("https://demo.komo.do", {
  type: "api-key",
  params: {
    api_key: "your_key",
    secret: "your secret",
  },
});

// Inferred as Types.StackListItem[]
const stacks = await komodo.read("ListStacks", {});

// Inferred as Types.Stack
const stack = await komodo.read("GetStack", {
  stack: stacks[0].name,
});
```
