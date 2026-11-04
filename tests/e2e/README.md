# End-to-end tests

Tests that drive the assembled application, from adding files to reading
the result (`TESTING.md` section 2):

- adding files through the picker and through drag and drop;
- selecting a valid target format;
- starting a job and cancelling it;
- inspecting the success and the failure details;
- opening the output folder;
- clearing the history.

This directory stays empty until the application can start a
conversion. The frontend component tests that exist today live in
`src/**/*.test.tsx`; they cover the user interface, not the whole
application.
