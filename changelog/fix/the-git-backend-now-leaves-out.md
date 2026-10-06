The Git backend now leaves out the root commit as a parent of a merge commit
instead of failing. This fixes abandoning a child of the root commit that is a
parent of a merge commit, including when a Git import abandons it.
[#8076](https://github.com/jj-vcs/jj/issues/8076)
