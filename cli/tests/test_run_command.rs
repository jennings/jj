// Copyright 2024 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use crate::common::TestEnvironment;
use crate::common::TestWorkDir;

#[test]
fn test_run_simple() {
    let mut test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let fake_formatter = assert_cmd::cargo::cargo_bin("fake-formatter");
    assert!(fake_formatter.is_file());
    let fake_formatter_path = fake_formatter.to_string_lossy().into_owned();
    test_env.add_paths_to_normalize(fake_formatter.clone(), "$FAKE_FORMATTER_PATH");
    let work_dir = test_env.work_dir("repo");
    work_dir.write_file("A.txt", "A");
    work_dir.run_jj(&["commit", "-m", "A"]).success();
    work_dir.write_file("b.txt", "b");
    work_dir.run_jj(&["commit", "-m", "B"]).success();
    work_dir.write_file("c.txt", "test to replace");
    work_dir.run_jj(&["commit", "-m", "C"]).success();
    insta::assert_snapshot!(get_log_output(&work_dir), @r"
    @  zsuskulnrvyrovkzqrwmxqlsskqntxvp
    ○  kkmpptxzrspxrzommnulwmwkkqwworplC
    │
    ○  rlvkpnrzqnoowoytxnquwvuryrwnrmlpB
    │
    ○  qpvuntsmwlqtpsluzzsnyyzlmlwvmlnuA
    │
    ◆  zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz
    [EOF]
    ");
    // `--tee touched.txt` creates a file in each working copy, so every commit's
    // tree gets rewritten.
    let stdout = work_dir
        .run_jj(&[
            "run",
            "-r",
            "..@",
            "--",
            &fake_formatter_path,
            "--stdout",
            "x",
            "--tee",
            "touched.txt",
        ])
        .success()
        .stdout;
    insta::assert_snapshot!(stdout, @"xxxx[EOF]");
}

#[test]
fn test_run_on_immutable() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");
    let fake_formatter = assert_cmd::cargo::cargo_bin("fake-formatter");
    assert!(fake_formatter.is_file());
    let fake_formatter_path = fake_formatter.to_string_lossy();
    work_dir.write_file("A.txt", "A");
    work_dir.run_jj(&["commit", "-m", "A"]).success();
    work_dir.write_file("b.txt", "b");
    work_dir.run_jj(&["commit", "-m", "B"]).success();
    work_dir.write_file("c.txt", "test to replace");
    work_dir.run_jj(&["commit", "-m", "C"]).success();
    insta::assert_snapshot!(get_log_output(&work_dir), @r"
    @  zsuskulnrvyrovkzqrwmxqlsskqntxvp
    ○  kkmpptxzrspxrzommnulwmwkkqwworplC
    │
    ○  rlvkpnrzqnoowoytxnquwvuryrwnrmlpB
    │
    ○  qpvuntsmwlqtpsluzzsnyyzlmlwvmlnuA
    │
    ◆  zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz
    [EOF]
    ");
    let output = work_dir.run_jj(&[
        "run",
        "-r",
        "all()",
        "--",
        &fake_formatter_path,
        "--uppercase",
    ]);
    insta::assert_snapshot!(output, @r"
    ------- stderr -------
    Error: The root commit 000000000000 is immutable
    [EOF]
    [exit status: 1]
    ");
}

#[test]
fn test_run_noop() {
    let mut test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let fake_formatter = assert_cmd::cargo::cargo_bin("fake-formatter");
    assert!(fake_formatter.is_file());
    let fake_formatter_path = fake_formatter.to_string_lossy().into_owned();
    test_env.add_paths_to_normalize(fake_formatter.clone(), "$FAKE_FORMATTER_PATH");
    let work_dir = test_env.work_dir("repo");
    work_dir.write_file("A.txt", "A");
    work_dir.run_jj(&["commit", "-m", "A"]).success();
    work_dir.write_file("b.txt", "b");
    work_dir.run_jj(&["commit", "-m", "B"]).success();
    work_dir.write_file("c.txt", "test to replace");
    work_dir.run_jj(&["commit", "-m", "C"]).success();
    insta::assert_snapshot!(get_log_output(&work_dir), @r"
    @  zsuskulnrvyrovkzqrwmxqlsskqntxvp
    ○  kkmpptxzrspxrzommnulwmwkkqwworplC
    │
    ○  rlvkpnrzqnoowoytxnquwvuryrwnrmlpB
    │
    ○  qpvuntsmwlqtpsluzzsnyyzlmlwvmlnuA
    │
    ◆  zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz
    [EOF]
    ");
    // `--stdout foo` writes to the subprocess's stdout, which `jj run` buffers
    // and emits to its own stdout. No tracked files in the working copy change,
    // so no commits get rewritten. Using a fixed string keeps the per-commit
    // output identical, so the concatenated stdout is stable regardless of the
    // (non-deterministic) order in which the parallel jobs finish.
    let output = work_dir
        .run_jj(&[
            "run",
            "-r",
            "..@",
            "--",
            &fake_formatter_path,
            "--stdout",
            "foo",
        ])
        .success();
    insta::assert_snapshot!(output.stdout, @"foofoofoofoo[EOF]");
    insta::assert_snapshot!(output.stderr, @r"
    No commits were rewritten as the command did not modify any tracked files
    Nothing changed.
    [EOF]
    ");
}

#[test]
fn test_run_sets_env_vars() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");
    work_dir.write_file("seed.txt", "seed");
    work_dir.run_jj(&["commit", "-m", "seed"]).success();

    // Show the change_id and commit_id so the reader can match them against
    // the values the subprocess writes into the per-commit working copy.
    let log_template = r#"change_id ++ " " ++ commit_id ++ " " ++ description ++ "\n""#;
    insta::assert_snapshot!(
        work_dir.run_jj(&["log", "-T", log_template]),
        @r"
    @  rlvkpnrzqnoowoytxnquwvuryrwnrmlp fc4c875c9bc90128cbb9e8084dd5f5f336b383d9
    ○  qpvuntsmwlqtpsluzzsnyyzlmlwvmlnu 5fbe90560fed1c39d46a46a672ba98abd53bdc6d seed
    │
    ◆  zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz 0000000000000000000000000000000000000000
    [EOF]
    "
    );

    // Each subprocess echoes its JJ_CHANGE_ID and JJ_COMMIT_ID into files in
    // the per-commit working copy, modifying the tree so the commit gets
    // rewritten with those files.
    let jj_args: &[&str] = if cfg!(windows) {
        &[
            "run",
            "-r",
            "@-",
            "--",
            "cmd",
            "/c",
            "echo %JJ_CHANGE_ID%>change_id.txt && echo %JJ_COMMIT_ID%>commit_id.txt",
        ]
    } else {
        &[
            "run",
            "-r",
            "@-",
            "--",
            "sh",
            "-c",
            "echo $JJ_CHANGE_ID > change_id.txt && echo $JJ_COMMIT_ID > commit_id.txt",
        ]
    };
    work_dir.run_jj(jj_args).success();

    let normalize_whitespace = |s: String| {
        s.replace("\r\n", "\n")
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    };
    insta::assert_snapshot!(
        work_dir
            .run_jj(&["file", "show", "-r", "@-", "change_id.txt"])
            .normalize_stdout_with(normalize_whitespace),
        @r"
    qpvuntsmwlqtpsluzzsnyyzlmlwvmlnu
    [EOF]
    "
    );
    insta::assert_snapshot!(
        work_dir
            .run_jj(&["file", "show", "-r", "@-", "commit_id.txt"])
            .normalize_stdout_with(normalize_whitespace),
        @r"
    5fbe90560fed1c39d46a46a672ba98abd53bdc6d
    [EOF]
    "
    );
}

#[test]
fn test_run_from_subdir_skips_commits_without_it() {
    let mut test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    // `fake-formatter --tee ran.txt` is a portable way to create an empty
    // `ran.txt`, equivalent to `touch ran.txt` but available on all platforms.
    let fake_formatter = assert_cmd::cargo::cargo_bin("fake-formatter");
    assert!(fake_formatter.is_file());
    let fake_formatter_path = fake_formatter.to_string_lossy().into_owned();
    test_env.add_paths_to_normalize(fake_formatter.clone(), "$FAKE_FORMATTER_PATH");
    let work_dir = test_env.work_dir("repo");

    // First commit has only root-level files; no `sub/` exists yet.
    work_dir.write_file("seed.txt", "seed");
    work_dir.run_jj(&["commit", "-m", "no-sub"]).success();
    // Second commit adds `sub/file.txt`, so `sub/` exists from here on.
    work_dir.write_file("sub/file.txt", "x");
    work_dir.run_jj(&["commit", "-m", "with-sub"]).success();

    // Run from inside sub/ on both ancestors. The command creates `ran.txt`
    // in cwd, so we can later tell where it ran. The `no-sub` commit has no
    // `sub/` directory and should be skipped; the `with-sub` commit has
    // `sub/` and should be rewritten with `sub/ran.txt` added.
    let sub_dir = work_dir.dir("sub");
    let output = sub_dir
        .run_jj(&[
            "run",
            "-r",
            "@-|@--",
            "--",
            &fake_formatter_path,
            "--tee",
            "ran.txt",
        ])
        .success()
        .normalize_backslash();
    insta::assert_snapshot!(output.stderr, @r"
    Skipped commit 3bb1f1ca3c09a8e6be46ef48515803464b16b426: directory does not exist: sub
    Rewrote 1 commits with $FAKE_FORMATTER_PATH --tee ran.txt
    Working copy  (@) now at: kkmpptxz 3548431a (empty) (no description set)
    Parent commit (@-)      : rlvkpnrz 3aa9a235 with-sub
    Added 1 files, modified 0 files, removed 0 files
    [EOF]
    ");

    // The rewritten `with-sub` commit has `sub/ran.txt`, alongside the
    // pre-existing `sub/file.txt`.
    insta::assert_snapshot!(
        work_dir
            .run_jj(&["file", "list", "-r", "@-"])
            .normalize_backslash(),
        @r"
    seed.txt
    sub/file.txt
    sub/ran.txt
    [EOF]
    "
    );
}

#[test]
fn test_run_root_flag() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    // `fake-formatter --tee ran.txt` is a portable way to create an empty
    // `ran.txt`, equivalent to `touch ran.txt` but available on all platforms.
    let fake_formatter = assert_cmd::cargo::cargo_bin("fake-formatter");
    assert!(fake_formatter.is_file());
    let fake_formatter_path = fake_formatter.to_string_lossy().into_owned();
    let work_dir = test_env.work_dir("repo");
    work_dir.write_file("sub/file.txt", "x");
    work_dir.run_jj(&["commit", "-m", "with-sub"]).success();

    // Invoke `jj run` from inside sub/, but pass `--root` so the command
    // executes from the workspace root and `ran.txt` lands at the top level.
    let sub_dir = work_dir.dir("sub");
    sub_dir
        .run_jj(&[
            "run",
            "--root",
            "-r",
            "@-",
            "--",
            &fake_formatter_path,
            "--tee",
            "ran.txt",
        ])
        .success();

    insta::assert_snapshot!(
        work_dir
            .run_jj(&["file", "list", "-r", "@-"])
            .normalize_backslash(),
        @r"
    ran.txt
    sub/file.txt
    [EOF]
    "
    );
}

#[test]
fn test_run_uses_revsets_run_as_default() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    // `fake-formatter --tee ran.txt` is a portable way to create an empty
    // `ran.txt`, equivalent to `touch ran.txt` but available on all platforms.
    let fake_formatter = assert_cmd::cargo::cargo_bin("fake-formatter");
    assert!(fake_formatter.is_file());
    let fake_formatter_path = fake_formatter.to_string_lossy().into_owned();
    let work_dir = test_env.work_dir("repo");

    // Two sibling commits, `foo` and `bar`.
    work_dir.write_file("file", "foo");
    work_dir
        .run_jj(["bookmark", "create", "-r@", "foo"])
        .success();
    work_dir.run_jj(["new", "root()"]).success();
    work_dir.write_file("file", "bar");
    work_dir
        .run_jj(["bookmark", "create", "-r@", "bar"])
        .success();
    work_dir.run_jj(["edit", "foo"]).success();

    test_env.add_config(r#"revsets.run = "bar""#);

    // Running `jj run` with `revsets.run=bar` should only modify bar
    work_dir
        .run_jj([
            "--config=revsets.run=\"bar\"",
            "run",
            "--",
            &fake_formatter_path,
            "--tee",
            "ran.txt",
        ])
        .success();

    insta::assert_snapshot!(
        work_dir.run_jj(["file", "list", "-r", "foo"]),
        @r"
    file
    [EOF]
    "
    );
    insta::assert_snapshot!(
        work_dir.run_jj(["file", "list", "-r", "bar"]),
        @r"
    file
    ran.txt
    [EOF]
    "
    );

    // Run again but now with foo in the config
    work_dir.run_jj(["undo"]).success();
    work_dir
        .run_jj([
            "--config=revsets.run=\"foo\"",
            "run",
            "--",
            &fake_formatter_path,
            "--tee",
            "ran.txt",
        ])
        .success();

    insta::assert_snapshot!(
        work_dir.run_jj(["file", "list", "-r", "foo"]),
        @r"
    file
    ran.txt
    [EOF]
    "
    );
    insta::assert_snapshot!(
        work_dir.run_jj(["file", "list", "-r", "bar"]),
        @r"
    file
    [EOF]
    "
    );
}

#[test]
fn test_run_failure_rewrites_nothing() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");
    work_dir.write_file("A.txt", "A");
    work_dir.run_jj(&["commit", "-m", "A"]).success();
    work_dir.write_file("b.txt", "b");
    work_dir.run_jj(&["commit", "-m", "B"]).success();
    let log_before = get_log_output(&work_dir);
    insta::assert_snapshot!(log_before, @r"
    @  kkmpptxzrspxrzommnulwmwkkqwworpl
    ○  rlvkpnrzqnoowoytxnquwvuryrwnrmlpB
    │
    ○  qpvuntsmwlqtpsluzzsnyyzlmlwvmlnuA
    │
    ◆  zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz
    [EOF]
    ");

    // Fail on commit B; succeed (modify the tree) on every other commit. If
    // any subprocess fails, `jj run` must roll back: no commit gets rewritten,
    // even the ones whose commands ran to completion before B's failure
    // propagated.
    let cmd = "if [ \"$JJ_CHANGE_ID\" = 'rlvkpnrzqnoowoytxnquwvuryrwnrmlp' ]; then exit 1; fi; \
               touch ran.txt";
    let output = work_dir.run_jj(&["run", "-r", "..@", "--", "sh", "-c", cmd]);
    assert!(!output.status.success(), "expected `jj run` to fail");

    // Log is unchanged: same change_ids, same shape, no descendants of B got
    // rebased onto a new commit.
    assert_eq!(get_log_output(&work_dir), log_before);
}

#[test]
fn test_run_recovers_after_failure() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    // `fake-formatter --fail` exits non-zero (like `false`) and
    // `fake-formatter --tee ran.txt` creates an empty `ran.txt` (like `touch`);
    // both are portable across platforms.
    let fake_formatter = assert_cmd::cargo::cargo_bin("fake-formatter");
    assert!(fake_formatter.is_file());
    let fake_formatter_path = fake_formatter.to_string_lossy().into_owned();
    let work_dir = test_env.work_dir("repo");
    work_dir.write_file("A.txt", "A");
    work_dir.run_jj(&["commit", "-m", "A"]).success();
    work_dir.write_file("b.txt", "b");
    work_dir.run_jj(&["commit", "-m", "B"]).success();

    // First run fails; the slots under `.jj/run/default/` are left without a
    // persisted tree_state (the dirty-marker mechanism).
    let first = work_dir.run_jj(&["run", "-r", "..@", "--", &fake_formatter_path, "--fail"]);
    assert!(!first.status.success(), "expected first `jj run` to fail");

    // A second run with a working command must succeed: the missing tree_state
    // triggers a clean wipe-and-reinit of each slot.
    work_dir
        .run_jj(&[
            "run",
            "-r",
            "..@",
            "--",
            &fake_formatter_path,
            "--tee",
            "ran.txt",
        ])
        .success();

    // Both commits in `..@` now carry `ran.txt`.
    insta::assert_snapshot!(
        work_dir.run_jj(&["file", "list", "-r", "@-"]),
        @r"
    A.txt
    b.txt
    ran.txt
    [EOF]
    "
    );
    insta::assert_snapshot!(
        work_dir.run_jj(&["file", "list", "-r", "@--"]),
        @r"
    A.txt
    ran.txt
    [EOF]
    "
    );
}

#[test]
fn test_run_shell_command() {
    // The new positional-args interface means users have to invoke a shell
    // explicitly to use shell features. This verifies that path works
    // end-to-end: each per-commit subprocess sees its `JJ_COMMIT_ID` and the
    // shell echoes it to stdout.
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");
    work_dir.write_file("A.txt", "A");
    work_dir.run_jj(&["commit", "-m", "A"]).success();
    work_dir.write_file("b.txt", "b");
    work_dir.run_jj(&["commit", "-m", "B"]).success();
    work_dir.write_file("c.txt", "test to replace");
    work_dir.run_jj(&["commit", "-m", "C"]).success();

    // Show the commit_ids so the reader can match them against the values
    // the snapshot below was captured with.
    let log_template = r#"change_id ++ " " ++ commit_id ++ " " ++ description ++ "\n""#;
    insta::assert_snapshot!(
        work_dir.run_jj(&["log", "-T", log_template, "-r", "..@"]),
        @r"
    @  zsuskulnrvyrovkzqrwmxqlsskqntxvp 8d0cb96bac2cfefd56a8691b9301ef44cc94a368
    ○  kkmpptxzrspxrzommnulwmwkkqwworpl 3406218c99ce8076f3a28434ebda109cbd84de9e C
    │
    ○  rlvkpnrzqnoowoytxnquwvuryrwnrmlp 9453b0f03bbda20fa849b10eb051d1e3eed1ec5d B
    │
    ○  qpvuntsmwlqtpsluzzsnyyzlmlwvmlnu 26d8ff9bba4faa4da6735ced959c57280e49afa7 A
    │
    ~
    [EOF]
    "
    );

    let jj_args: &[&str] = if cfg!(windows) {
        &["run", "-r", "..@", "--", "cmd", "/c", "echo %JJ_COMMIT_ID%"]
    } else {
        &[
            "run",
            "-r",
            "..@",
            "--",
            "bash",
            "-c",
            r#"echo "$JJ_COMMIT_ID""#,
        ]
    };
    let output = work_dir.run_jj(jj_args).success();

    // Parallel jobs finish in non-deterministic order, so sort before
    // asserting.
    let mut lines: Vec<&str> = output.stdout.raw().lines().collect();
    lines.sort_unstable();
    let sorted_stdout = lines.join("\n");
    insta::assert_snapshot!(sorted_stdout, @r"
    26d8ff9bba4faa4da6735ced959c57280e49afa7
    3406218c99ce8076f3a28434ebda109cbd84de9e
    8d0cb96bac2cfefd56a8691b9301ef44cc94a368
    9453b0f03bbda20fa849b10eb051d1e3eed1ec5d
    ");
}

#[test]
fn test_run_sets_workspace_root_env_var() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");

    work_dir.write_file("seed.txt", "seed");
    work_dir.run_jj(&["commit", "-m", "seed"]).success();

    // Each subprocess writes $JJ_WORKSPACE_ROOT into a file so we can assert
    // it equals the actual workspace root (not the per-commit working copy).
    let jj_args: &[&str] = if cfg!(windows) {
        &[
            "run",
            "-r",
            "@-",
            "--",
            "cmd",
            "/c",
            "echo %JJ_WORKSPACE_ROOT%>workspace_root.txt",
        ]
    } else {
        &[
            "run",
            "-r",
            "@-",
            "--",
            "sh",
            "-c",
            "echo $JJ_WORKSPACE_ROOT > workspace_root.txt",
        ]
    };
    work_dir.run_jj(jj_args).success();

    // Trim trailing whitespace per line and normalize CRLF to LF so the
    // snapshot is identical on Windows and Unix.
    let normalize_whitespace = |s: String| {
        s.replace("\r\n", "\n")
            .lines()
            .map(|line| line.trim_end().to_string())
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    };
    // $TEST_ENV is the normalized placeholder for the test environment's temp
    // root directory. JJ_WORKSPACE_ROOT should point to the slot working copy
    // under .jj/run/default/1/working_copy, not to the original workspace.
    insta::assert_snapshot!(
        work_dir
            .run_jj(&["file", "show", "-r", "@-", "workspace_root.txt"])
            .normalize_stdout_with(normalize_whitespace),
        @r"
    $TEST_ENV/repo/.jj/run/default/1/working_copy
    [EOF]
    "
    );
}

#[test]
fn test_run_pool_persists_between_runs() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");
    work_dir.write_file("seed.txt", "seed");
    work_dir.run_jj(&["commit", "-m", "seed"]).success();

    work_dir
        .run_jj(&[
            "run",
            "--config",
            "run.jobs=1",
            "-r",
            "@-",
            "--",
            "touch",
            "ran.txt",
        ])
        .success();

    // The pool slot directory survives the run.
    let pool_slot = work_dir.root().join(".jj/run/default/1");
    assert!(
        pool_slot.exists(),
        "expected pool slot 1 to persist between runs at {pool_slot:?}",
    );
    assert!(pool_slot.join("working_copy").exists());
    assert!(pool_slot.join("state").exists());

    // A second run reuses the existing slot rather than recreating it from
    // scratch.
    work_dir
        .run_jj(&[
            "run",
            "--config",
            "run.jobs=1",
            "-r",
            "@-",
            "--",
            "touch",
            "ran2.txt",
        ])
        .success();
    assert!(pool_slot.join("working_copy").exists());
}

#[test]
fn test_run_pool_size_from_config() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");
    work_dir.write_file("a.txt", "a");
    work_dir.run_jj(&["commit", "-m", "A"]).success();
    work_dir.write_file("b.txt", "b");
    work_dir.run_jj(&["commit", "-m", "B"]).success();
    work_dir.write_file("c.txt", "c");
    work_dir.run_jj(&["commit", "-m", "C"]).success();

    // `run.jobs = 1` forces all three commits to share a single slot
    // and be processed sequentially.
    work_dir
        .run_jj(&[
            "run",
            "--config",
            "run.jobs=1",
            "-r",
            "..@",
            "--",
            "touch",
            "ran.txt",
        ])
        .success();

    let pool_dir = work_dir.root().join(".jj/run/default");
    assert!(pool_dir.join("1").exists(), "pool/1 should exist");
    assert!(
        !pool_dir.join("2").exists(),
        "pool/2 should NOT exist with size=1",
    );

    // All three commits picked up `ran.txt`.
    for parent in ["@---", "@--", "@-"] {
        let files = work_dir
            .run_jj(&["file", "list", "-r", parent])
            .success()
            .stdout
            .to_string();
        assert!(
            files.contains("ran.txt"),
            "expected ran.txt in {parent}, got:\n{files}",
        );
    }
}

/// `--jobs N` controls pool size when `run.jobs` is not set.
#[test]
fn test_run_pool_size_from_jobs_flag() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");
    work_dir.write_file("a.txt", "a");
    work_dir.run_jj(&["commit", "-m", "A"]).success();
    work_dir.write_file("b.txt", "b");
    work_dir.run_jj(&["commit", "-m", "B"]).success();
    work_dir.write_file("c.txt", "c");
    work_dir.run_jj(&["commit", "-m", "C"]).success();

    // `--jobs 2` with pool: expect exactly 2 slots, no more.
    work_dir
        .run_jj(&["run", "--jobs", "2", "-r", "..@", "--", "touch", "ran.txt"])
        .success();

    let pool_dir = work_dir.root().join(".jj/run/default");
    assert!(pool_dir.join("1").exists(), "pool/1 should exist");
    assert!(pool_dir.join("2").exists(), "pool/2 should exist");
    assert!(
        !pool_dir.join("3").exists(),
        "pool/3 must NOT exist with --jobs 2",
    );
}

#[test]
fn test_run_pool_preserves_untracked_artifacts() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");

    // The .gitignore keeps `cache/` out of the snapshot's tracking set, so
    // files there persist on disk between jobs without being committed.
    work_dir.write_file(".gitignore", "cache/\n");
    work_dir.write_file("seed.txt", "seed");
    work_dir.run_jj(&["commit", "-m", "seed"]).success();

    // Run 1: drop a marker into the gitignored cache directory.
    work_dir
        .run_jj(&[
            "run",
            "--config",
            "run.jobs=1",
            "-r",
            "@-",
            "--",
            "sh",
            "-c",
            "mkdir -p cache && echo run1 > cache/marker && touch ran1.txt",
        ])
        .success();

    // Run 2: assert the marker is still there, write its contents into a
    // tracked file so we can verify from outside.
    work_dir
        .run_jj(&[
            "run",
            "--config",
            "run.jobs=1",
            "-r",
            "@-",
            "--",
            "sh",
            "-c",
            "if [ -f cache/marker ]; then cp cache/marker result.txt; else echo MISSING > \
             result.txt; fi",
        ])
        .success();

    let result = work_dir
        .run_jj(&["file", "show", "-r", "@-", "result.txt"])
        .success()
        .stdout
        .to_string();
    assert!(
        result.starts_with("run1"),
        "expected result.txt to contain marker content from run 1; got: {result}",
    );
}

/// Pool correctly removes a file that is present in one commit's tree but
/// absent in the next commit processed by the same slot.
#[test]
fn test_run_pool_removes_file_absent_in_next_commit() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");

    // Commit A has only_in_a.txt.
    work_dir.write_file("only_in_a.txt", "a");
    work_dir.run_jj(&["commit", "-m", "A"]).success();
    // After commit -m A the WC inherits A's files. Delete the file so that
    // commit B's tree is empty (no only_in_a.txt).
    std::fs::remove_file(work_dir.root().join("only_in_a.txt")).unwrap();
    work_dir.run_jj(&["commit", "-m", "B"]).success();
    // Stack: root → A (only_in_a.txt) → B ({}) → @

    // Pool size 1 forces both commits through the same slot sequentially.
    // @-- = A, @- = B (WC is above B).
    work_dir
        .run_jj(&[
            "run",
            "--config",
            "run.jobs=1",
            "-r",
            "@--::@-",
            "--",
            "touch",
            "ran.txt",
        ])
        .success();

    // A's rewrite (@--) keeps only_in_a.txt (and gains ran.txt).
    let files_a = work_dir
        .run_jj(&["file", "list", "-r", "@--"])
        .success()
        .stdout
        .to_string();
    assert!(
        files_a.contains("only_in_a.txt"),
        "expected only_in_a.txt in A; got:\n{files_a}",
    );

    // B's rewrite (@-) must NOT have only_in_a.txt leaked from A's slot.
    let files_b = work_dir
        .run_jj(&["file", "list", "-r", "@-"])
        .success()
        .stdout
        .to_string();
    assert!(
        !files_b.contains("only_in_a.txt"),
        "only_in_a.txt must not leak into B; got:\n{files_b}",
    );
    assert!(
        files_b.contains("ran.txt"),
        "expected ran.txt in B; got:\n{files_b}",
    );
}

/// Files created by a command (not in `.gitignore`) must not leak from one
/// commit to another in the same `jj run` invocation.
#[test]
fn test_run_pool_no_file_leak_between_commits() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");

    work_dir.write_file("seed.txt", "seed");
    work_dir.run_jj(&["commit", "-m", "A"]).success();
    work_dir.run_jj(&["commit", "-m", "B"]).success();
    work_dir.run_jj(&["commit", "-m", "C"]).success();

    // A's command produces from_a.txt; B and C do not.
    work_dir
        .run_jj(&[
            "run",
            "--config",
            "run.jobs=1",
            "-r",
            "..@",
            "--",
            "sh",
            "-c",
            // Write from_a.txt only for commit A; B and C only touch ran.txt.
            r#"if [ "$JJ_CHANGE_ID" = "$(jj log -r '@---' --no-graph -T change_id 2>/dev/null)" ]; then touch from_a.txt; fi; touch ran.txt"#,
        ])
        .success();

    // A's rewrite may or may not have from_a.txt depending on the order of
    // processing, but B and C must not have it.
    for (rev, parent) in [("B", "@--"), ("C", "@-")] {
        let files = work_dir
            .run_jj(&["file", "list", "-r", parent])
            .success()
            .stdout
            .to_string();
        assert!(
            !files.contains("from_a.txt"),
            "from_a.txt must not leak into commit {rev}; got:\n{files}",
        );
    }
}

/// Files tracked into a commit by run 1 must not reappear in a different
/// commit processed by the same pool slot in run 2.
///
/// Run 1 rewrites commit2 (adding artifact.txt). The slot's saved tree_state
/// therefore records {seed.txt, artifact.txt}. Run 2 targets commit1 whose
/// tree is {seed.txt}; the checkout diff removes artifact.txt from the slot,
/// so commit1's rewrite must not contain it.
#[test]
fn test_run_pool_no_file_leak_between_invocations() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");

    // commit1: seed.txt only.
    work_dir.write_file("seed.txt", "seed");
    work_dir.run_jj(&["commit", "-m", "commit1"]).success();
    // commit2: descends from commit1, no extra files (same tree).
    work_dir.run_jj(&["commit", "-m", "commit2"]).success();
    // Stack: root → commit1 (seed.txt) → commit2 (seed.txt) → @

    // Run 1: produce artifact.txt in commit2 (@-) so the slot's saved
    // tree_state records {seed.txt, artifact.txt}.
    // Stack before run 1: root → commit1 (@--) → commit2 (@-) → @
    work_dir
        .run_jj(&[
            "run",
            "--config",
            "run.jobs=1",
            "-r",
            "@-",
            "--",
            "touch",
            "artifact.txt",
        ])
        .success();
    // After run 1: root → commit1 (@--) → commit2' (@-) → @ (rebased).
    // commit2' now has {seed.txt, artifact.txt}.

    // Run 2: no-op on commit1 (@--). Pool reuses the slot whose tree_state
    // says {seed.txt, artifact.txt}. The checkout diff removes artifact.txt,
    // so commit1's rewrite must not contain it.
    work_dir
        .run_jj(&["run", "--config", "run.jobs=1", "-r", "@--", "--", "true"])
        .success();
    // After run 2: root → commit1' (@--) → commit2'' (@-) → @ (rebased).

    let files = work_dir
        .run_jj(&["file", "list", "-r", "@--"])
        .success()
        .stdout
        .to_string();
    assert!(
        !files.contains("artifact.txt"),
        "artifact.txt must not leak into commit1 from previous slot state; got:\n{files}",
    );
    assert!(
        files.contains("seed.txt"),
        "expected seed.txt in commit1; got:\n{files}",
    );
}

/// A slot whose tree_state is absent (simulating a crash mid-job) should be
/// wiped and reinitialised on the next acquisition, not produce garbage.
#[test]
fn test_run_pool_recovers_from_missing_tree_state() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");

    work_dir.write_file("seed.txt", "seed");
    work_dir.run_jj(&["commit", "-m", "seed"]).success();

    // Prime the slot so working_copy/ exists.
    work_dir
        .run_jj(&["run", "--config", "run.jobs=1", "-r", "@-", "--", "true"])
        .success();

    // Simulate a crash: plant a stale file in the slot and delete tree_state.
    let slot = work_dir.root().join(".jj/run/default/1");
    std::fs::write(slot.join("working_copy/stale.txt"), "crash leftovers").unwrap();
    std::fs::remove_file(slot.join("state/tree_state")).unwrap();

    // A second run should wipe the stale state and produce a clean rewrite.
    work_dir
        .run_jj(&[
            "run",
            "--config",
            "run.jobs=1",
            "-r",
            "@-",
            "--",
            "touch",
            "ran.txt",
        ])
        .success();

    let files = work_dir
        .run_jj(&["file", "list", "-r", "@-"])
        .success()
        .stdout
        .to_string();
    assert!(
        files.contains("ran.txt"),
        "expected ran.txt in rewrite after recovery; got:\n{files}",
    );
    assert!(
        !files.contains("stale.txt"),
        "stale.txt must not survive crash recovery; got:\n{files}",
    );
}

/// A failed command (non-zero exit) must not poison the pool slot for the
/// next `jj run` invocation.
#[test]
fn test_run_pool_failed_command_does_not_poison_slot() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");

    work_dir.write_file("seed.txt", "seed");
    work_dir.run_jj(&["commit", "-m", "seed"]).success();

    // Run 1: command fails, but writes a file first.
    drop(work_dir.run_jj(&[
        "run",
        "--config",
        "run.jobs=1",
        "-r",
        "@-",
        "--",
        "sh",
        "-c",
        "touch poison.txt; exit 1",
    ]));

    // Run 2: a clean command. poison.txt must not appear in the rewrite.
    work_dir
        .run_jj(&[
            "run",
            "--config",
            "run.jobs=1",
            "-r",
            "@-",
            "--",
            "touch",
            "ran.txt",
        ])
        .success();

    let files = work_dir
        .run_jj(&["file", "list", "-r", "@-"])
        .success()
        .stdout
        .to_string();
    assert!(files.contains("ran.txt"), "expected ran.txt; got:\n{files}",);
    assert!(
        !files.contains("poison.txt"),
        "poison.txt from failed run must not appear; got:\n{files}",
    );
}

/// `--clean` wipes each slot before running the command so untracked artifacts
/// left by a previous run do not survive into the next run's working copy.
#[test]
fn test_run_clean_wipes_slot() {
    let test_env = TestEnvironment::default();
    test_env.run_jj_in(".", ["git", "init", "repo"]).success();
    let work_dir = test_env.work_dir("repo");

    work_dir.write_file("seed.txt", "seed");
    work_dir.run_jj(&["commit", "-m", "seed"]).success();

    // Prime slot 1 with a leftover untracked file to simulate a build artifact.
    let slot_wc = work_dir.root().join(".jj/run/default/1/working_copy");
    work_dir
        .run_jj(&["run", "--config", "run.jobs=1", "-r", "@-", "--", "true"])
        .success();
    assert!(slot_wc.exists(), "slot 1 should exist after first run");
    std::fs::write(slot_wc.join("leftover.txt"), "artifact").unwrap();

    // Without --clean the leftover file is visible to the command (tracked via
    // auto-tracking). Confirm by running a command that echoes its presence.
    let jj_args_no_clean: &[&str] = if cfg!(windows) {
        &[
            "run",
            "--config",
            "run.jobs=1",
            "-r",
            "@-",
            "--",
            "cmd",
            "/c",
            "if exist leftover.txt (echo PRESENT > saw_it.txt)",
        ]
    } else {
        &[
            "run",
            "--config",
            "run.jobs=1",
            "-r",
            "@-",
            "--",
            "sh",
            "-c",
            "[ -e leftover.txt ] && echo PRESENT > saw_it.txt || true",
        ]
    };
    work_dir.run_jj(jj_args_no_clean).success();
    let files_no_clean = work_dir
        .run_jj(&["file", "list", "-r", "@-"])
        .success()
        .stdout
        .to_string();
    assert!(
        files_no_clean.contains("saw_it.txt") || files_no_clean.contains("leftover.txt"),
        "expected leftover.txt to be visible without --clean; got:\n{files_no_clean}",
    );

    // With --clean, the slot is wiped before checkout. leftover.txt vanishes.
    work_dir.run_jj(&["undo"]).success();
    std::fs::write(slot_wc.join("leftover.txt"), "artifact").unwrap();

    let jj_args_clean: &[&str] = if cfg!(windows) {
        &[
            "run",
            "--clean",
            "--config",
            "run.jobs=1",
            "-r",
            "@-",
            "--",
            "cmd",
            "/c",
            "if exist leftover.txt (echo PRESENT > saw_it.txt)",
        ]
    } else {
        &[
            "run",
            "--clean",
            "--config",
            "run.jobs=1",
            "-r",
            "@-",
            "--",
            "sh",
            "-c",
            "[ -e leftover.txt ] && echo PRESENT > saw_it.txt || true",
        ]
    };
    work_dir.run_jj(jj_args_clean).success();

    let files_clean = work_dir
        .run_jj(&["file", "list", "-r", "@-"])
        .success()
        .stdout
        .to_string();
    assert!(
        !files_clean.contains("saw_it.txt") && !files_clean.contains("leftover.txt"),
        "expected leftover.txt to be absent with --clean; got:\n{files_clean}",
    );
}

fn get_log_output(work_dir: &TestWorkDir) -> String {
    work_dir
        .run_jj(&["log", "-T", r#"change_id ++ description ++ "\n""#])
        .success()
        .stdout
        .to_string()
}
