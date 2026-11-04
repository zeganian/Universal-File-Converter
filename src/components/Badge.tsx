import type { ReactNode } from 'react';

/**
 * A status badge.
 *
 * Status is always conveyed by text as well as colour
 * (`DESIGN.md` section 2), so the badge never relies on the colour
 * alone.
 */
export function Badge({
  tone = 'neutral',
  children,
}: {
  tone?: 'neutral' | 'success' | 'warning' | 'danger' | 'info' | 'accent';
  children: ReactNode;
}) {
  return <span className={`badge badge--${tone}`}>{children}</span>;
}
