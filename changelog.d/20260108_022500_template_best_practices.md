### Changed

- Updated CI/CD pipeline to match latest best practices from rust-ai-driven-development-pipeline-template
- Added detect-changes job for optimized CI workflow that skips unnecessary checks
- Added release_mode option (instant or changelog-pr) for manual releases
- Added changelog-pr job for creating PRs with changelog fragments
- Added crates.io publishing steps to auto-release and manual-release workflows
- Improved job dependencies using always() patterns for consistent behavior
- Updated changelog check to require fragments for code changes (exits with error instead of warning)
- Updated CONTRIBUTING.md with trader-bot specific references
