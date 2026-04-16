---
cargo/satteri-mdxjs: patch
cargo/satteri-pulldown-cmark: patch
npm/satteri: patch
---

Fixed JSX inside MDX expression bodies, JSX inside `.map()` callbacks or other expressions is now compiled to `_jsx()` calls instead of being dropped or emitted as raw JSX
