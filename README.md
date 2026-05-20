# Savepoint

[![crates-badge]][crates-package]  [![nix-badge]][nix-package] 

[nix-badge]: https://img.shields.io/badge/Packaged_for-Nix-5277C3.svg?logo=nixos&labelColor=73C3D5
[nix-package]: https://github.com/NamtaoProductions/savepoint/blob/main/flake.nix
[crates-badge]: https://img.shields.io/crates/v/savepoint
[crates-package]: https://crates.io/crates/savepoint

[![Release](https://github.com/NamtaoProductions/savepoint/actions/workflows/release.yaml/badge.svg)](https://github.com/NamtaoProductions/savepoint/actions/workflows/release.yaml) [![Merge checks](https://github.com/NamtaoProductions/savepoint/actions/workflows/merge.yaml/badge.svg)](https://github.com/NamtaoProductions/savepoint/actions/workflows/merge.yaml) [![Deploy docs](https://github.com/NamtaoProductions/savepoint/actions/workflows/docs.yaml/badge.svg)](https://github.com/NamtaoProductions/savepoint/actions/workflows/docs.yaml) [![Nightly build](https://github.com/NamtaoProductions/savepoint/actions/workflows/nightly.yaml/badge.svg)](https://github.com/NamtaoProductions/savepoint/actions/workflows/nightly.yaml)

## [Can't beat the boss? Load a previous save!](https://savepoint.namtao.com)

<picture>
  <source height="150px" align="right" media="(prefers-color-scheme: dark)" srcset="https://www.namtao.com/img/user/Resources/Meta/attachments/savepoint-white.png">
  <source height="150px" align="right" media="(prefers-color-scheme: light)" srcset="[/savepoint.png](https://www.namtao.com/img/user/Resources/Meta/attachments/savepoint.png)">
  <img alt="Savepoint logo" align="right" height="150px" src="https://www.namtao.com/img/user/Resources/Meta/attachments/savepoint-white.png">
</picture>

_A command watcher that commits when you fix errors_

Feel free to poke around the [code](https://github.com/NamtaoProductions/savepoint/tree/main/), I would love [feature suggestions](https://github.com/NamtaoProductions/savepoint/issues/new/choose)!

# Usage

```shell
Usage: savepoint [OPTIONS] --filetypes <filetypes> [COMMAND]...

Arguments:
  [COMMAND]...  Command to run (use after -- if your shell requires it)

Options:
  -f, --filetypes <filetypes>  Filename extensions to watch (eg rs, js, py, java), comma-separated
  -d, --dryrun               Don't run git commit when tests pass
  -c, --clear                Clear screen between executions
  -q, --quiet                Don't display test output
  -h, --help                 Print help
  -V, --version              Print version
```

# DEMO

```shell
🏁 SAVEPOINT: Running command...
   Compiling savepoint v0.1.4 (/home/oatman/projects/savepoint)
error: expected one of `!` or `::`, found `SavePoint`
  --> src/main.rs:37:10
   |
37 | uhoh! impl SavePoint<Passing> {
   |            ^^^^^^^^^ expected one of `!` or `::`

error: could not compile `savepoint` (bin "savepoint") due to 1 previous error
🏁 SAVEPOINT: Error!
🏁 SAVEPOINT: Monitoring..
```

```shell
🏁 SAVEPOINT: Running command...
   Compiling savepoint v0.1.4 (/home/oatman/projects/savepoint)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.18s
🏁 SAVEPOINT: Autosaving!
[main 3d65083] SAVEPOINT REACHED!
 2 files changed, 16 insertions(+), 1 deletion(-)
🏁 SAVEPOINT: Monitoring..
```

### State diagram

```mermaid
flowchart LR
PASSING-->|fail|FAILING
FAILING-->|pass; git commit|PASSING
```

> Other transitions are no-ops (such as tests passing while in passing state)

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://brainmade.org/white-logo.svg">
  <source media="(prefers-color-scheme: light)" srcset="https://brainmade.org/black-logo.svg">
  <img align="right" alt="Brain mark." src="https://brainmade.org/black-logo.svg">
</picture>
