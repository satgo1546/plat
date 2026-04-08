My attempt at [Making a Language](https://thunderseethe.dev/series/making-a-language/).

---

If you see ‘TypeError: typeVariables.getOrInsert is not a function’ while running tests:

As of this writing (Node 25), no released Node.js version supports Map.prototype.getOrInsert.
I'm living in the future. Use `deno x vitest` instead of `npx vitest`.
