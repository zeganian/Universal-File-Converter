import { Badge } from '@/components/Badge';
import { EmptyState } from '@/components/EmptyState';
import { PageHeader } from '@/components/PageHeader';
import { useAppInfo } from '@/app/useAppInfo';
import type { EngineAvailability, EngineStatus } from '@/lib/ipc';

function availabilityBadge(availability: EngineAvailability) {
  switch (availability) {
    case 'available':
      return <Badge tone="success">Available</Badge>;
    case 'incompatibleVersion':
      return <Badge tone="warning">Incompatible version</Badge>;
    case 'missing':
    default:
      return <Badge tone="danger">Not detected</Badge>;
  }
}

function EngineRow({ engine }: { engine: EngineStatus }) {
  return (
    <li className="engine-row">
      <div className="engine-row__identity">
        <span className="engine-row__name">{engine.displayName}</span>
        <span className="engine-row__id">{engine.engineId}</span>
      </div>

      <div className="engine-row__status">
        {availabilityBadge(engine.availability)}
        {engine.source ? <span className="engine-row__source">{engine.source}</span> : null}
        {engine.version ? (
          <span className="engine-row__version">version {engine.version}</span>
        ) : null}
      </div>

      {engine.reason ? (
        <div className="engine-row__detail">
          <p className="engine-row__message">{engine.reason.message}</p>
          {engine.reason.remediation ? (
            <p className="engine-row__remediation">{engine.reason.remediation}</p>
          ) : null}
        </div>
      ) : null}

      {engine.installationHint ? (
        <p className="engine-row__hint">{engine.installationHint}</p>
      ) : null}
    </li>
  );
}

/**
 * The Engines screen.
 *
 * It lists every engine the application knows about, whether it was
 * detected, and how to resolve a missing engine (`PRD.md` FR-004,
 * `DESIGN.md` section 9).
 */
export function EnginesPage() {
  const { engines, loading, error } = useAppInfo();

  return (
    <div className="page">
      <PageHeader
        title="Engines"
        description="Conversion engines detected on this device, with their version and installation source."
      />

      {error ? (
        <div className="alert alert--danger" role="alert">
          <strong>The backend could not be reached.</strong>
          <span>{error}</span>
        </div>
      ) : null}

      {loading ? (
        <p className="panel__meta">Detecting engines…</p>
      ) : engines && engines.engines.length > 0 ? (
        <section className="panel">
          <ul className="engine-list">
            {engines.engines.map((engine) => (
              <EngineRow key={engine.engineId} engine={engine} />
            ))}
          </ul>
        </section>
      ) : (
        <EmptyState
          title="No engine detected"
          description="No conversion engine is registered yet. Engine detection and its platform specific installation guidance will appear here once an engine adapter exists."
        />
      )}
    </div>
  );
}
