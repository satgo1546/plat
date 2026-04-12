My attempt at [Making a Language](https://thunderseethe.dev/series/making-a-language/) in TypeScript.

The project is abandoned for the following reasons.

- The series make excessive use of crates.
  There are often no equivalent packages in TypeScript (or any language other than Rust).
  While convenient, it leaves the compiler implementation with a large amount of glue code.
  Such an approach is less helpful for learning.
- The language design is questionable.
  In Base, the AST (λ-calculus) is in fact a level lower than the IR (System F),
  leading to confusing and pointless “lowering” and “monomorphization” passes
  that does not reflect what real-world compilers do.
  Later passes also seem to be hindered by poor design choices.

---

If you see ‘TypeError: typeVariables.getOrInsert is not a function’ while running tests:

As of this writing (Node 25), no released Node.js version supports Map.prototype.getOrInsert.
I'm living in the future. Use `deno x vitest` instead of `npx vitest`.
