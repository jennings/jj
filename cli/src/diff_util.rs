use jj_lib::conflicts::{materialize_tree_value, MaterializedTreeValue};
use jj_lib::merge::MergedTreeValue;
use jj_lib::repo::Repo;
use jj_lib::{diff, files, rewrite};
fn diff_content(path: &RepoPath, value: MaterializedTreeValue) -> Result<Vec<u8>, CommandError> {
    match value {
        MaterializedTreeValue::Absent => Ok(vec![]),
        MaterializedTreeValue::File { mut reader, .. } => {
            let mut contents = vec![];
            reader.read_to_end(&mut contents)?;
            Ok(contents)
        MaterializedTreeValue::Symlink { id: _, target } => Ok(target.into_bytes()),
        MaterializedTreeValue::GitSubmodule(id) => {
        MaterializedTreeValue::Conflict { id: _, contents } => Ok(contents),
        MaterializedTreeValue::Tree(id) => {
            panic!("Unexpected tree with id {id:?} in diff at path {path:?}");
fn basic_diff_file_type(value: &MaterializedTreeValue) -> &'static str {
    match value {
        MaterializedTreeValue::Absent => {
        MaterializedTreeValue::File { executable, .. } => {
                "executable file"
                "regular file"
        MaterializedTreeValue::Symlink { .. } => "symlink",
        MaterializedTreeValue::Tree(_) => "tree",
        MaterializedTreeValue::GitSubmodule(_) => "Git submodule",
        MaterializedTreeValue::Conflict { .. } => "conflict",
    let store = workspace_command.repo().store();
            let left_value = materialize_tree_value(store, &path, left_value).block_on()?;
            let right_value = materialize_tree_value(store, &path, right_value).block_on()?;
                let right_content = diff_content(&path, right_value)?;
                let description = match (&left_value, &right_value) {
                        MaterializedTreeValue::File {
                        },
                        MaterializedTreeValue::File {
                        },
                        if *left_executable && *right_executable {
                        } else if *left_executable {
                        } else if *right_executable {
                    (
                        MaterializedTreeValue::Conflict { .. },
                        MaterializedTreeValue::Conflict { .. },
                    ) => "Modified conflict in".to_string(),
                    (MaterializedTreeValue::Conflict { .. }, _) => {
                        "Resolved conflict in".to_string()
                    }
                    (_, MaterializedTreeValue::Conflict { .. }) => {
                        "Created conflict in".to_string()
                    (
                        MaterializedTreeValue::Symlink { .. },
                        MaterializedTreeValue::Symlink { .. },
                    ) => "Symlink target changed at".to_string(),
                    (_, _) => {
                        let left_type = basic_diff_file_type(&left_value);
                        let right_type = basic_diff_file_type(&right_value);
                let left_content = diff_content(&path, left_value)?;
                let right_content = diff_content(&path, right_value)?;
                let left_content = diff_content(&path, left_value)?;
    value: MaterializedTreeValue,
    let mut contents: Vec<u8>;
    match value {
        MaterializedTreeValue::Absent => {
            panic!("Absent path {path:?} in diff should have been handled by caller");
        }
        MaterializedTreeValue::File {
            id,
            executable,
            mut reader,
        } => {
            mode = if executable {
            contents = vec![];
            reader.read_to_end(&mut contents)?;
        MaterializedTreeValue::Symlink { id, target } => {
            contents = target.into_bytes();
        MaterializedTreeValue::GitSubmodule(id) => {
            contents = vec![];
        MaterializedTreeValue::Conflict {
            id: _,
            contents: conflict_data,
        } => {
            contents = conflict_data
        MaterializedTreeValue::Tree(_) => {
            panic!("Unexpected tree in diff at path {path:?}");
        content: contents,
    let store = workspace_command.repo().store();
            let left_value = materialize_tree_value(store, &path, left_value).block_on()?;
            let right_value = materialize_tree_value(store, &path, right_value).block_on()?;
                let right_part = git_diff_part(&path, right_value)?;
                let left_part = git_diff_part(&path, left_value)?;
                let right_part = git_diff_part(&path, right_value)?;
                let left_part = git_diff_part(&path, left_value)?;
    let store = workspace_command.repo().store();
            let left = materialize_tree_value(store, &repo_path, left).block_on()?;
            let right = materialize_tree_value(store, &repo_path, right).block_on()?;
            let left_content = diff_content(&repo_path, left)?;
            let right_content = diff_content(&repo_path, right)?;