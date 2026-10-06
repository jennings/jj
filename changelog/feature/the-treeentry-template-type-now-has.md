The `TreeEntry` template type now has
`.removed_values() -> List<Option<TreeValue>>` and
`.added_values() -> List<Option<TreeValue>>` methods, and `TreeValue` now
exposes `.object_id()`, `.file_type()`, and `.executable()` methods for
inspecting each side of a (possibly conflicted) tree entry.
