import type { JobStatus } from '@/types/generated/JobStatus';
import type { Progress } from '@/types/generated/Progress';

/**
 * Display helpers for job status.
 *
 * The label mapping is the counterpart of `JobStatus::label()` in
 * `crates/ufc-core/src/domain/job.rs`. Both sides must agree, because
 * `DESIGN.md` section 8 defines the wording the user sees.
 */
export const JOB_STATUS_LABELS: Record<JobStatus, string> = {
  queued: 'Queued',
  validating: 'Checking file',
  waitingForEngine: 'Waiting for engine',
  running: 'Converting',
  validatingOutput: 'Validating output',
  succeeded: 'Completed',
  succeededWithWarnings: 'Completed with warnings',
  failed: 'Failed',
  cancelled: 'Cancelled',
};

/** Terminal statuses never change again. */
export const TERMINAL_STATUSES: readonly JobStatus[] = [
  'succeeded',
  'succeededWithWarnings',
  'failed',
  'cancelled',
];

/** Returns `true` when the job can no longer change state. */
export function isTerminal(status: JobStatus): boolean {
  return TERMINAL_STATUSES.includes(status);
}

/** Returns the user facing label of a job status. */
export function statusLabel(status: JobStatus): string {
  return JOB_STATUS_LABELS[status];
}

/**
 * Returns the percentage to render, or `null` when the progress is
 * indeterminate.
 *
 * The UI must never substitute a fabricated percentage
 * (`ARCHITECTURE.md` section 9).
 */
export function progressPercent(progress: Progress): number | null {
  if (progress.kind !== 'determinate') return null;
  return progress.value.percent;
}
