const store = new Map();
globalThis.localStorage = {
  getItem: (key) => (store.has(String(key)) ? store.get(String(key)) : null),
  setItem: (key, val) => store.set(String(key), String(val)),
  removeItem: (key) => store.delete(String(key)),
  clear: () => store.clear(),
  key: (idx) => Array.from(store.keys())[idx] ?? null,
  get length() {
    return store.size;
  },
};
