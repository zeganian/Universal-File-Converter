import { describe, expect, it } from 'vitest';

import { isTerminal, progressPercent, statusLabel } from '@/features/queue/jobStatus';

describe('job status helpers', () => {
  it('uses the wording defined in the design document', () => {
    expect(statusLabel('validating')).toBe('Checking file');
    expect(statusLabel('running')).toBe('Converting');
    expect(statusLabel('succeeded')).toBe('Completed');
    expect(statusLabel('succeededWithWarnings')).toBe('Completed with warnings');
    expect(statusLabel('failed')).toBe('Failed');
    expect(statusLabel('cancelled')).toBe('Cancelled');
  });

  it('recognises terminal statuses', () => {
    expect(isTerminal('succeeded')).toBe(true);
    expect(isTerminal('succeededWithWarnings')).toBe(true);
    expect(isTerminal('failed')).toBe(true);
    expect(isTerminal('cancelled')).toBe(true);
    expect(isTerminal('queued')).toBe(false);
    expect(isTerminal('running')).toBe(false);
  });

  it('returns the percentage only for determinate progress', () => {
    expect(progressPercent({ kind: 'determinate', value: 42 })).toBe(42);
    expect(progressPercent({ kind: 'indeterminate' })).toBeNull();
    expect(progressPercent({ kind: 'stage', value: 'converting' })).toBeNull();
  });
});
