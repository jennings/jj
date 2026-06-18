# Running commands across revisions with `jj run`

`jj run` executes a command against each revision in a set. For every selected
revision it:

- Checks the tree out into an isolated working copy
- Runs the command
- If the command succeeded, amends the revision with any changes made to the working copy

Descendants are rebased on top of the amended revisions so the diff propagates
through the stack.

This is useful for things like running a code generator, formatter, build step,
or test suite against every revision in a stack — either to update each revision
in place, or to verify each one builds.

## When to use `jj run` vs. `jj fix`

[`jj fix`] also transforms a set of revisions, but operates differently:

- Fix tools are pre-configured, usually in the repository configuration.
- Each file's contents are piped to the stdin of the fix tool, and the result is
  read from stdout.
- The working copy is not materialized.

If your transformation can be expressed as a [`jj fix`] tool (i.e., read from
stdin and writes to stdout) - prefer `jj fix` because it will be significantly
faster and more parallelizable.

Use `jj run` when the operation:

- needs a real working copy on disk (build systems, test runners, anything
  that resolves imports across files);
- needs to run a single command that observes or modifies the tree as a
  whole, rather than one file at a time;
- needs to produce or consume artifacts outside the repo.

[`jj fix`]: ../config.md#code-formatting-and-other-file-content-transformations

## Basic usage

```shell
$ jj run -r 'A::B' -- cargo nextest run
```

If `-r` is omitted, `jj run` uses the revset defined by `revsets.run` revset,
which defaults to `reachable(@, mutable())` - every mutable revision reachable
from the working copy.

### Examples for common ecosystems

```shell
# Rust
$ jj run -r 'main..@' -- cargo nextest run

# JavaScript / TypeScript
$ jj run -r 'main..@' -- pnpm build
$ jj run -r 'main..@' -- tsc --noEmit

# .NET
$ jj run -r 'main..@' -- dotnet test

# Go
$ jj run -r 'main..@' -- go test ./...
$ jj run -r 'main..@' -- go build ./...

# Python
$ jj run -r 'main..@' -- pytest
$ jj run -r 'main..@' -- ruff check .
```

If the command exits non-zero on a revision, `jj run` stops and leaves that
revision unchanged. Use `--keep-going` (`-k`) to continue past failures
instead - successful rewrites are still applied atomically at the end.

## Running in parallel

`jj run` processes one revision at a time by default. Use `--jobs` (`-j`) to
run several in parallel:

```shell
$ jj run -j 4 -r 'main..@' -- cargo nextest run
```

Each parallel job runs in its own working copy under `.jj/run/default/N/`.
These working copies are reused between invocations, so build artifacts and
incremental caches survive between runs. Pass `--clean` to wipe each working
copy before checking out, which is occasionally useful when a build system
gets confused by stale state.

The default parallelism can also be configured globally with
[`run.jobs`](../config.md#runjobs-default-parallelism).

## Inspecting commits without rewriting them

Pass `--no-amend` to run the command against each revision without amending
the result back. The revision is still checked out and the command still
executes — only the outcome is discarded. This is the right flag for
read-only checks like tests or linters across a stack:

```shell
$ jj run --no-amend -r 'main..@' -j 4 -- cargo nextest run
```

Unlike the default mode, `--no-amend` is permitted on immutable commits.

This is also useful as an alternative to [`jj bisect run`] when you want to
see results from _every_ commit, not just find the boundary between
good and bad:

```shell
$ jj run --no-amend -k -r 'main..@' -- cargo test
```

[`jj bisect run`]: ../cli-reference.md#jj-bisect-run

## Customizing output with a shell

The subprocess's stdout and stderr are captured and emitted as a single block
when the command finishes, so output from parallel jobs does not interleave.
For each invocation, `jj run` sets these environment variables:

- `JJ_CHANGE_ID` — the change ID of the revision being processed
- `JJ_COMMIT_ID` — the commit ID
- `JJ_WORKSPACE_ROOT` — the path to the per-job working copy

To produce a compact one-line summary per revision, wrap the work in a
shell and reference these variables:

```shell
$ jj run -j 4 -- bash -c '
    cargo nextest run >/dev/null 2>&1
    echo "$JJ_CHANGE_ID: $?"
  '
```

## Producing artifacts for multiple revisions

`JJ_CHANGE_ID` and `JJ_COMMIT_ID` also make it straightforward to build
artifacts for a set of tagged revisions in one pass. Combine a revset that
selects them with a shell script that copies the build output somewhere
outside the working copy:

```shell
$ out=$PWD/artifacts
$ mkdir -p "$out"
$ jj run --no-amend -j 4 -r 'tag1 | tag2 | tag3 | main' -- bash -c '
    set -e
    ./build.sh
    cp output.zip "'"$out"'/$JJ_CHANGE_ID.zip"
  '
```

`--no-amend` is usually the right choice here: artifact builds typically
should not rewrite the tagged revisions themselves.

Note that the output directory has to be resolved to an absolute path before
the subprocess starts, because each invocation runs from inside its own
working copy under `.jj/run/default/N/`, not from the directory `jj run` was
launched from.

## Restoring descendants

By default, `jj run` propagates the diff each command introduced into
descendants by rebasing them. If you want to update each revision's tree in
place but leave descendants' contents alone, pass `--restore-descendants`.
Descendants are still reparented onto the rewritten ancestors, but their
trees are preserved.
