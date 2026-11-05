import { useCallback, useEffect, useState } from 'react';

import { Badge } from '@/components/Badge';
import { EmptyState } from '@/components/EmptyState';
import { PageHeader } from '@/components/PageHeader';
import { isTerminal, statusLabel } from '@/features/queue/jobStatus';
import { cancelJob, JOB_UPDATED_EVENT, listJobs } from '@/lib/ipc';
import type { ConversionResult } from '@/types/generated/ConversionResult';
import type { JobStatus } from '@/types/generated/JobStatus';

function statusBadge(status: JobStatus) {
  switch (status) {
    case 'succeeded':
      return <Badge tone="success">Completed</Badge>;
    case 'succeededWithWarnings':
      return <Badge tone="warning">Completed with warnings</Badge>;
    case 'failed':
      return <Badge tone="danger">Failed</Badge>;
    case 'cancelled':
      return <Badge tone="neutral">Cancelled</Badge>;
    default:
      return <Badge tone="info">{statusLabel(status)}</Badge>;
  }
}

/**
 * The Queue screen.
 *
 * The queue shows queued, running and finished jobs. Results are
 * presented here rather than on a separate screen, matching
 * `DESIGN.md` section 8.
 */
export function QueuePage() {
  const [jobs, setJobs] = useState<ConversionResult[]>([]);
  const [loading, setLoading] = useState(true);

  const merge = useCallback((job: ConversionResult) => {
    setJobs((current) => {
      const index = current.findIndex((entry) => entry.jobId === job.jobId);

      if (index === -1) {
        return [job, ...current];
      }

      const next = [...current];
      next[index] = job;
      return next;
    });
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    void (async () => {
      try {
        const existing = await listJobs();
        if (!cancelled) setJobs(existing);

        const { listen } = await import('@tauri-apps/api/event');
        unlisten = await listen<ConversionResult>(JOB_UPDATED_EVENT, (event) => {
          merge(event.payload);
        });
      } catch {
        // The backend cannot be reached. The empty state explains it.
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [merge]);

  const cancel = useCallback(async (jobId: string) => {
    try {
      await cancelJob(jobId);
    } catch {
      // The job keeps running; the list shows the real state.
    }
  }, []);

  return (
    <div className="page">
      <PageHeader
        title="Queue"
        description="Conversions that are waiting, running or finished appear here."
      />

      {loading ? (
        <p className="panel__meta">Loading the queue…</p>
      ) : jobs.length === 0 ? (
        <EmptyState
          title="The queue is empty"
          description="Add files on the Convert screen to start a conversion. Progress, results and warnings appear here."
        />
      ) : (
        <section className="panel">
          <ul className="job-list">
            {jobs.map((job) => (
              <li key={job.jobId} className="job-row">
                <div className="job-row__header">
                  <span className="job-row__name">
                    {job.sourceName ?? fileNameOf(job.outputPath) ?? job.jobId.slice(0, 8)}
                  </span>
                  {statusBadge(job.status)}
                </div>

                <div className="job-row__meta">
                  {job.engineId ? (
                    <span className="job-row__engine">
                      {job.engineId}
                      {job.engineVersion ? ` ${job.engineVersion}` : ''}
                    </span>
                  ) : null}
                  {job.progress.kind === 'stage' ? (
                    <span className="job-row__stage">{job.progress.value.name}</span>
                  ) : null}
                  {job.progress.kind === 'determinate' ? (
                    <span className="job-row__stage">{job.progress.value.percent}%</span>
                  ) : null}
                </div>

                {job.outputPath ? (
                  <p className="job-row__output" title={job.outputPath}>
                    {job.outputPath}
                  </p>
                ) : null}

                {(job.warnings ?? []).map((warning) => (
                  <p key={warning} className="job-row__warning">
                    {warning}
                  </p>
                ))}

                {job.error ? (
                  <div className="job-row__error">
                    <p className="job-row__message">{job.error.message}</p>
                    {job.error.remediation ? (
                      <p className="job-row__remediation">{job.error.remediation}</p>
                    ) : null}
                    {job.error.detail ? (
                      <details className="job-row__details">
                        <summary>Technical details</summary>
                        <pre>{job.error.detail}</pre>
                      </details>
                    ) : null}
                  </div>
                ) : null}

                {!isTerminal(job.status) ? (
                  <button type="button" className="button" onClick={() => void cancel(job.jobId)}>
                    Cancel
                  </button>
                ) : null}
              </li>
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}

/** Returns the file name of an output path. */
function fileNameOf(path: string | null | undefined): string | null {
  if (!path) return null;

  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || null;
}
