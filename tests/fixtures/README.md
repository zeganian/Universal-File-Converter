# Fixtures

Synthetic, redistributable input files for the test suites.

## Generating them

```bash
node scripts/generate-fixtures.mjs
```

The files are written to `fixtures/generated/`, which is ignored by
git. They are generated rather than committed so that no third party
content enters the repository and so that every fixture is
reproducible (`TESTING.md` section 3).

## The generated set

| Path                                 | Category                   | Purpose                                                      |
| ------------------------------------ | -------------------------- | ------------------------------------------------------------ |
| `valid/minimal.png`                  | valid minimal file         | The smallest input an adapter can be tested with.            |
| `valid/gradient.png`                 | complex layout             | A gradient, used for visual and semantic comparison.         |
| `valid/large.png`                    | large file                 | 512×512, used for the file size bands of the resource tests. |
| `valid/nama file ünïcode ✓.png`      | unicode filename           | Non-ASCII characters and spaces in a path.                   |
| `valid/report; rm -rf $(whoami).png` | shell metacharacters       | The name must be treated as data, never as a command.        |
| `invalid/empty.png`                  | zero byte file             | Must be rejected, must not crash the engine.                 |
| `invalid/corrupt.png`                | malformed file             | Random bytes behind a valid extension.                       |
| `invalid/mismatch.png`               | extension/content mismatch | A text file named `.png`.                                    |
| `invalid/truncated.png`              | truncated file             | A valid PNG cut in half.                                     |

## Rules

- Do not commit private or copyrighted documents as fixtures.
- Do not commit generated files; regenerate them instead.
- Add a fixture only together with the test that uses it.
