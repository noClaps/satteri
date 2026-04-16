---
cargo/satteri-ast: patch
npm/satteri: patch
---

Fixed HAST property names not being mapped to their HTML attribute names during rendering (e.g. `className` now renders as `class`, `htmlFor` as `for`)
