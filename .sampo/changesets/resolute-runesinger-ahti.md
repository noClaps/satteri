---
cargo/satteri-pulldown-cmark: patch
cargo/satteri-ast: patch
npm/satteri: patch
---

Fixed `code.value` in the MDAST tree including a trailing newline for well-formed fenced code blocks, which diverged from `remark-parse`. MDAST plugins inspecting `node.value` now see the same bytes as remark.
