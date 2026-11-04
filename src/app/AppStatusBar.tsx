import { useAppInfo } from '@/app/useAppInfo';

/**
 * The bottom status bar.
 *
 * It shows the engine readiness and the application version, as
 * described in `DESIGN.md` section 6.
 */
export function AppStatusBar() {
  const { info, engines, error } = useAppInfo();

  const engineSummary = (() => {
    if (error) return 'Backend unreachable';
    if (!engines) return 'Checking engines…';
    if (engines.engines.length === 0) return 'No engines detected yet';
    if (engines.availableCount === 0) return 'No engine available';
    return `${engines.availableCount} of ${engines.engines.length} engines ready`;
  })();

  return (
    <footer className="status-bar">
      <span className="status-bar__item">
        <span className="status-bar__label">Engines:</span>
        <span className="status-bar__value">{engineSummary}</span>
      </span>
      <span className="status-bar__item">
        <span className="status-bar__label">Version:</span>
        <span className="status-bar__value">{info?.version ?? '—'}</span>
      </span>
    </footer>
  );
}
