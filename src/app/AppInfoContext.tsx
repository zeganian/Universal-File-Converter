import { useEffect, useMemo, useState } from 'react';
import type { ReactNode } from 'react';

import { AppContext } from '@/app/appContext';
import type { AppContextValue } from '@/app/appContext';
import { getAppInfo, getCapabilities, getEngineStatus, refreshEngineStatus } from '@/lib/ipc';
import type { AppInfo, CapabilitySummary, EngineReport } from '@/lib/ipc';

/** Provides the backend capability state to the application. */
export function AppInfoProvider({ children }: { children: ReactNode }) {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [capabilities, setCapabilities] = useState<CapabilitySummary | null>(null);
  const [engines, setEngines] = useState<EngineReport | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function load(): Promise<void> {
      try {
        const [appInfo, capabilitySummary, engineReport] = await Promise.all([
          getAppInfo(),
          getCapabilities(),
          getEngineStatus(),
        ]);

        if (cancelled) return;
        setInfo(appInfo);
        setCapabilities(capabilitySummary);
        setEngines(engineReport);
        setError(null);
      } catch (cause: unknown) {
        if (cancelled) return;
        setError(cause instanceof Error ? cause.message : 'The backend could not be reached.');
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    void load();

    return () => {
      cancelled = true;
    };
  }, []);

  const value = useMemo<AppContextValue>(
    () => ({
      info,
      capabilities,
      engines,
      loading,
      error,
      refresh: async () => {
        const [appInfo, capabilitySummary, engineReport] = await Promise.all([
          getAppInfo(),
          getCapabilities(),
          getEngineStatus(),
        ]);
        setInfo(appInfo);
        setCapabilities(capabilitySummary);
        setEngines(engineReport);
      },
      refreshEngines: async () => {
        // The engines are detected again, so that a newly installed
        // engine becomes available without restarting the application.
        const engineReport = await refreshEngineStatus();
        setEngines(engineReport);
        const capabilitySummary = await getCapabilities();
        setCapabilities(capabilitySummary);
      },
    }),
    [info, capabilities, engines, loading, error],
  );

  return <AppContext.Provider value={value}>{children}</AppContext.Provider>;
}
