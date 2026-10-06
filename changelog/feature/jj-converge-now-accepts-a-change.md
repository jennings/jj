`jj converge` now accepts a change ID prefix when prompting for the change to
converge. The special `q` abort choice was removed; canceling with EOF now
exits with an error (previously `q` aborted successfully).
