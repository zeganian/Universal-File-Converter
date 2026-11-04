import { EmptyState } from '@/components/EmptyState';
import { PageHeader } from '@/components/PageHeader';

/**
 * The History screen.
 *
 * History is disabled by default so that no source path is stored
 * without the user asking for it (`PRD.md` FR-011).
 */
export function HistoryPage() {
  return (
    <div className="page">
      <PageHeader
        title="History"
        description="Previously run conversions are listed here. Only metadata is stored, never file contents."
      />

      <EmptyState
        title="No history recorded"
        description="Conversion history is currently disabled. Enable it in Settings to keep a local record of previous conversions."
      />
    </div>
  );
}
