---
cargo/satteri-ast: patch
npm/satteri: patch
---

Fixed source positions being dropped for most node types during mdast-to-hast conversion, so hast plugins now see accurate positions across the tree
