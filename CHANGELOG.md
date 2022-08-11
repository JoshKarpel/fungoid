# Changelog

## 0.2.2

## 0.2.1

### Added

- IDE mode now allows editing of the program.
- A set of example programs is now bundled into the `fungoid` executable.
  Run `fungoid examples` to see the `NAME`s of the examples,
  and `fungoid run example:NAME` or `fungoid ide example:NAME` to execute them.

### Changed

- Errors during program execution now return `Result`s instead of panicking.

## 0.2.0

### Added

- Added IDE mode (`fungoid ide FILE`), with visual execution of programs.
