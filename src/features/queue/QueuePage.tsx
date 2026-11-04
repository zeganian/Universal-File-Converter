import { EmptyState } from '@/components/EmptyState';
import { PageHeader } from '@/components/PageHeader';

/**
 * The Queue screen.
 *
 * The queue shows queued, running and finished jobs. Results are
 * presented here rather than on a separate screen, matching
 * `DESIGN.md` section 8.
 */
export function QueuePage() {
  return (
    <div className="page">
      <PageHeader
        title="Queue"
        description="Conversions that are waiting, running or finished appear here."
      />

      <EmptyState
        title="The queue is empty"
        description="Add files on the Convert screen to start a conversion. Progress, results and warnings appear here."
      />
    </div>
  );
}
