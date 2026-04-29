---
cargo/satteri-ast: minor
npm/satteri: minor
---

MDAST plugins can now set `data.hName`, `data.hProperties`, and `data.hChildren` on a node and have Sätteri render the corresponding HAST element, matching the rehype idiom.

This is especially useful for rendering directives, given a `containerDirective`, an `hName` of `"aside"` and `hProperties` of `{ className: ["note"] }`, satteri will emit `<aside class="note">…</aside>`.
