---
bump: minor
---

### Added
- Links Notation configuration support via new `lino_args` module
  - Case conversion utilities: `to_upper_case`, `to_camel_case`, `to_kebab_case`, `to_snake_case`, `to_pascal_case`
  - Environment variable helpers with case-insensitive lookup: `getenv`, `getenv_int`, `getenv_bool`, `getenv_decimal`, `getenv_u64`
  - Lenv file loading support for `.lenv` configuration files
  - Auto-loading of lenv files from CLI args, environment, or common locations
- Full CLI argument parsing with clap
  - Support for `--config`, `--lenv`, `--log-level`, `--verbose`, `--dry-run`, `--account`, `--user`, `--balance-interval`, `--order-delay`, `--run-once`, `--demo` options
  - All options have corresponding environment variable support (e.g., `TRADER_BOT_CONFIG`, `TRADER_BOT_LOG_LEVEL`)
- Configuration priority chain following Links Notation: CLI args > Environment variables > Config files > Defaults
- `RuntimeConfig` struct for unified configuration management
- 26 new unit tests for lino_args module

### Changed
- Updated `main.rs` to use new CLI parsing with lenv support
- Integrated lino_args helpers into prelude for easy access
