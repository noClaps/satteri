---
cargo/satteri-ast: minor
cargo/satteri-plugin-api: patch
cargo/satteri-napi: patch
cargo/satteri: patch
npm/satteri: patch
---

Fixed plugins silently dropping all but the last structural change against a given node. Multiple `insertBefore`/`insertAfter` calls on the same node, or sibling inserts paired with a `removeNode` on that same node, now all apply in the order they were issued.

Combinations that don't have a sensible meaning, like modifying something inside a removed subtree, now report an error instead of silently dropping the change.
