In non-colocated workspaces, `jj git import; jj undo; jj git import` no longer
re-imports external changes. Run `jj git export` to reflect undone changes in
Git. `jj git fetch; jj undo; jj git fetch` continues to work as expected.
