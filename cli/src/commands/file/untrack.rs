// Copyright 2020 The Jujutsu Authors
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

use std::io::Write as _;

use clap_complete::ArgValueCompleter;
use itertools::Itertools as _;
use jj_lib::merge::Merge;
use jj_lib::merged_tree::MergedTreeBuilder;
use jj_lib::repo::Repo as _;
use tracing::instrument;

use crate::cli_util::CommandHelper;
use crate::cli_util::print_snapshot_stats;
use crate::command_error::CommandError;
use crate::command_error::user_error_with_hint;
use crate::complete;
use crate::ui::Ui;

/// Stop tracking specified paths in the working copy
#[derive(clap::Args, Clone, Debug)]
pub(crate) struct FileUntrackArgs {
    /// Paths to untrack. They must already be ignored.
    ///
    /// The paths could be ignored via a .gitignore or .git/info/exclude (in
    /// colocated repos).
    #[arg(
        required = true,
        value_name = "FILESETS",
        value_hint = clap::ValueHint::AnyPath,
        add = ArgValueCompleter::new(complete::all_revision_files),
    )]
    paths: Vec<String>,
}

#[instrument(skip_all)]
pub(crate) fn cmd_file_untrack(
    ui: &mut Ui,
    command: &CommandHelper,
    args: &FileUntrackArgs,
) -> Result<(), CommandError> {
    let mut workspace_command = command.workspace_helper(ui)?;
    let store = workspace_command.repo().store().clone();
    let matcher = workspace_command
        .parse_file_patterns(ui, &args.paths)?
        .to_matcher();
    let auto_tracking_matcher = workspace_command.auto_tracking_matcher(ui)?;
    let options =
        workspace_command.snapshot_options_with_start_tracking_matcher(&auto_tracking_matcher)?;

    let mut tx = workspace_command.start_transaction().into_inner();
    let (mut locked_ws, wc_commit) = workspace_command.start_working_copy_mutation()?;
    // Create a new tree without the unwanted files
    let mut tree_builder = MergedTreeBuilder::new(wc_commit.tree_id().clone());
    let wc_tree = wc_commit.tree()?;
    for (path, _value) in wc_tree.entries_matching(matcher.as_ref()) {
        tree_builder.set_or_remove(path, Merge::absent());
    }
    let new_tree_id = tree_builder.write_tree(&store)?;
    let new_commit = tx
        .repo_mut()
        .rewrite_commit(&wc_commit)
        .set_tree_id(new_tree_id)
        .write()?;
    // Reset the working copy to the new commit
    locked_ws.locked_wc().reset(&new_commit)?;
    // Commit the working copy again so we can inform the user if paths couldn't be
    // untracked because they're not ignored.
    let (wc_tree_id, stats) = locked_ws.locked_wc().snapshot(&options)?;
    if wc_tree_id != *new_commit.tree_id() {
        let wc_tree = store.get_root_tree(&wc_tree_id)?;
        let added_back = wc_tree.entries_matching(matcher.as_ref()).collect_vec();
        if !added_back.is_empty() {
            drop(locked_ws);
            let path = &added_back[0].0;
            let ui_path = workspace_command.format_file_path(path);
            let message = if added_back.len() > 1 {
                format!(
                    "'{}' and {} other files are not ignored.",
                    ui_path,
                    added_back.len() - 1
                )
            } else {
                format!("'{ui_path}' is not ignored.")
            };
            return Err(user_error_with_hint(
                message,
                "Files that are not ignored will be added back by the next command.
Make sure they're ignored, then try again.",
            ));
        } else {
            // This means there were some concurrent changes made in the working copy. We
            // don't want to mix those in, so reset the working copy again.
            locked_ws.locked_wc().reset(&new_commit)?;
        }
    }
    let num_rebased = tx.repo_mut().rebase_descendants()?;
    if num_rebased > 0 {
        writeln!(ui.status(), "Rebased {num_rebased} descendant commits")?;
    }
    let repo = tx.commit("untrack paths")?;
    locked_ws.finish(repo.op_id().clone())?;
    print_snapshot_stats(ui, &stats, workspace_command.env().path_converter())?;
    Ok(())
}
