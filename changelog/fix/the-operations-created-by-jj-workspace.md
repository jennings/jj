The operations created by `jj workspace add` are now attributed to the
workspace the command was run in, so `jj undo` there undoes them instead of
refusing to undo an operation from another workspace.
