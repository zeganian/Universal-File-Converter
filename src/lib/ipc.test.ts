import { describe, expect, it } from 'vitest';

import { BackendError, toBackendError } from '@/lib/ipc';

describe('toBackendError', () => {
  it('wraps a structured backend error', () => {
    const error = toBackendError({
      code: 'missingDependency',
      message: 'FFmpeg was not found.',
      remediation: 'Install FFmpeg.',
      detail: 'probe exited with code 1',
    });

    expect(error).toBeInstanceOf(BackendError);
    expect(error.code).toBe('missingDependency');
    expect(error.message).toBe('FFmpeg was not found.');
    expect(error.remediation).toBe('Install FFmpeg.');
    expect(error.detail).toBe('probe exited with code 1');
  });

  it('normalises a missing optional field to null', () => {
    const error = toBackendError({ code: 'internal', message: 'boom' });

    expect(error.remediation).toBeNull();
    expect(error.detail).toBeNull();
  });

  it('falls back to an internal error for an unknown rejection value', () => {
    const error = toBackendError('something went wrong');

    expect(error.code).toBe('internal');
    expect(error.message).toBe('The backend could not be reached.');
  });

  it('uses the message of a thrown Error', () => {
    const error = toBackendError(new Error('IPC channel missing'));

    expect(error.code).toBe('internal');
    expect(error.message).toBe('IPC channel missing');
  });

  it('never loses the error code of a structured payload', () => {
    const error = toBackendError({ code: 'outputExists', message: 'File exists.' });

    expect(error.code).toBe('outputExists');
  });
});
