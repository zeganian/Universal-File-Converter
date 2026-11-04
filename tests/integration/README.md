# Integration tests

Tests that run a real conversion against a real fixture and verify the
result.

They cover, per `TESTING.md` section 2:

- a real conversion using small, known fixtures;
- the existence and the signature of the output;
- that the output opens in an independent reader;
- a batch with mixed success and failure;
- the behaviour of output name collisions;
- the cleanup of temporary directories.

This directory stays empty until a conversion pair exists. A test that
does not convert anything would only pretend that a conversion works,
and the completion rule of the project forbids advertising a pair
before an adapter, a fixture, a validation and a documentation entry
exist.

Rust integration tests for the domain layer live next to the crate they
test, in `crates/ufc-core/tests/`.
