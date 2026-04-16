---
cargo/satteri-ast: patch
npm/satteri: patch
---

Fixed script and style element contents being entity-escaped, which produced invalid output (e.g. `&lt;` inside `<script>`)
