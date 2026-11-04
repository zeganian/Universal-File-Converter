# Security and Privacy Requirements

## Universal File Converter

## 1. Security posture

All files selected by the user must be treated as untrusted input. Conversion engines may parse complex and malformed data. The application should minimize privileges, isolate temporary data, and avoid network access during conversion.

## 2. Privacy defaults

- Files remain on the user's device.
- No automatic upload or cloud processing.
- No telemetry or analytics by default.
- No document contents in logs or history.
- Do not access folders beyond user-selected paths and required app directories.
- Explain any future feature that transmits data and require explicit consent.

## 3. Path and filesystem safety

- Canonicalize and validate paths in Rust.
- Reject directories where a regular file is expected.
- Prevent output paths from escaping the chosen destination.
- Never overwrite the source.
- Do not follow untrusted archive paths outside the extraction root.
- Handle symlinks carefully.
- Use unique per-job temporary directories and restrictive permissions.
- Clean temporary data after terminal job states, with safe recovery after crashes.

## 4. Process safety

- Invoke binaries directly with argument vectors.
- Never interpolate user-controlled data into shell commands.
- Resolve engines from trusted app/system locations.
- Do not accept arbitrary executable paths from the frontend.
- Apply timeouts, output limits, and concurrency limits.
- Terminate child process trees on cancellation.
- Sanitize environment variables where practical.

## 5. Tauri security

- Use the minimum required Tauri capabilities.
- Restrict filesystem scope.
- Keep privileged operations in Rust.
- Validate all command arguments in the backend.
- Do not expose generic command execution to the UI.
- Review plugin permissions before adding plugins.
- Keep dependencies updated and audit security advisories.

## 6. Resource exhaustion

Consider:

- decompression bombs;
- huge image dimensions;
- malformed PDFs;
- media files with extreme duration/stream counts;
- archive nesting;
- excessive page counts;
- engine hangs;
- temporary disk exhaustion.

Provide configurable concurrency and safe failure states. Avoid claiming a hard universal maximum size.

## 7. Logs and diagnostics

Logs may include:

- job ID;
- engine and version;
- conversion pair;
- duration;
- exit code;
- sanitized error summary.

Logs must not include:

- file contents;
- extracted text;
- passwords;
- access tokens;
- unnecessary full paths;
- personal metadata unless explicitly needed for a local diagnostic.

## 8. Third-party engines

Before bundling an engine:

- review its license and build configuration;
- include notices and license texts;
- document codecs/features enabled;
- review known vulnerabilities;
- pin or constrain versions;
- provide a process to update vulnerable dependencies.

## 9. Vulnerability reporting

Add a private vulnerability reporting route in the GitHub repository if supported. Do not ask reporters to publish sensitive sample files or personal documents in public issues.
