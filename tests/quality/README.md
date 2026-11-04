# Quality reports

Local, non-committed reports about conversion runs, as described in
`TESTING.md` section 6.

Each run records:

- the input size and the output size;
- the peak memory, when it can be measured;
- the duration;
- the CPU usage, when it is available;
- the free disk space before and after;
- the cancellation response time;
- the validation level that was reached.

The output of a run is written to `output/` inside this directory,
which is ignored by git. The reports describe a test result, not a
product limit: no file size is guaranteed.
