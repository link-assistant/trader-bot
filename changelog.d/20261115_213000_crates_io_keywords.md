---
bump: patch
---

### Fixed

- Limit crate keywords to five so crates.io accepts the first publication.
- Check the crates.io keyword limit in the automated test suite.
- Use Node.js 24 for release scripts whose dependencies require Set.union.
- Verify package publishing in CI and explain missing bootstrap credentials in release failures.
