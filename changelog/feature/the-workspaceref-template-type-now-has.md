The `WorkspaceRef` template type now has a `.current_workspace() -> Boolean`
method, which can be used to identify the current workspace. This is slightly
different than `self.target().current_working_copy()` because multiple
workspaces can point to the same commit.
