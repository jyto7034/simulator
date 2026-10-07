# Auditor checker note

The first condition-record assembly exited with KeyError(path) because original meaning records use both path/file_sha256 and source/source_sha256 forms. No condition was marked successful by that failed attempt and no target file changed. The checker now handles both explicit forms and separately validates the optional completed-scope source digest. This is an auditor metadata-shape correction, not a product or execution finding.
