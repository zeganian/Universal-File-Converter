import type { ReactNode } from 'react';

/**
 * An empty state.
 *
 * The empty state always explains what to do next
 * (`DESIGN.md` section 9).
 */
export function EmptyState({
  title,
  description,
  action,
}: {
  title: string;
  description: string;
  action?: ReactNode;
}) {
  return (
    <div className="empty-state">
      <h2 className="empty-state__title">{title}</h2>
      <p className="empty-state__description">{description}</p>
      {action ? <div className="empty-state__action">{action}</div> : null}
    </div>
  );
}
