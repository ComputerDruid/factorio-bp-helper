## Unreleased

### Added
- New subcommand: `save`, which saves blueprints as a .json file, or as a directory of json files for blueprint books

### Changed
- `upgrade-quality` works on combinators
- `upgrade-quality` now errors when encountering something that looks like a quality that it doesn't know if it should upgrade.
- `🟦❯` prompt mode now only enabled if stdin is a terminal. If it's not, instead we just read from stdin until EOF.

## Version 0.1.4

### Added
- New command `upgrade-quality` for upgrading a blueprint to a higher quality when parameterized blueprints don't work.

## Version 0.1.3

### Fixed
- `count-entities` fixed to understand different qualities.

## Version 0.1.2

### Added
- Added support for blueprint books in `count-entities`. Ingredients will be summed across all the blueprints in the blueprint book.

## Version 0.1.1

### Changed
- When prompting for a blueprint, there's now a `🟦❯` prompt and a `received.` response when pasting a blueprint to give better feedback.

## Version 0.1.0

Initial release.
