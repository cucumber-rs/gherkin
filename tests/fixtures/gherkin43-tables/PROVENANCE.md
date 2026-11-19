# Official Gherkin 43 invalid table oracle

Files are exact upstream testdata/bad fixtures at commit
3d70e2bb4e0f16a568ee539a5ce0b03fb247a777 (v43.0.0):
https://github.com/cucumber/gherkin/tree/v43.0.0/testdata/bad

The feature has a short step row at line 6 and a long Examples row at line 14.
The official errors file records both. Native parsing remains fail-fast; the
acceptance requires the typed inconsistent-cell-count diagnostic, not identical
error aggregation, message wording or positions. Dedicated BDD examples isolate
both directions for step and Examples tables. MIT notice is retained in LICENSE.

042bda2882d793c42fcd5f4d272579a432d1d52a0e5436202e5e823eae4771b6  LICENSE
35a18d3027b9ec01235eb17d770ae635de091ece5b8ef17dc82b7598621dcafe  inconsistent_cell_count.feature
42fd199970057f37a328773ad624030864940788a7fc34cd39d415a01ca16386  inconsistent_cell_count.feature.errors.ndjson
