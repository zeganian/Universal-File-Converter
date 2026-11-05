/**
 * Small formatting helpers for the interface.
 *
 * They live outside of the feature components, so that a component file
 * only exports components.
 */

/** Formats a size in bytes for the interface. */
export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
