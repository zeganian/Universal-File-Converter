import { createContext } from 'react';

import type { AppInfo, CapabilitySummary, EngineReport } from '@/lib/ipc';

/**
 * State describing what the backend currently offers.
 *
 * This is *read only* application state. It mirrors what the backend
 * reports and never computes capabilities of its own: the frontend must
 * derive the list of selectable formats from the Rust capability
 * registry (`SUPPORTED_FORMATS.md`, "Registry rules").
 */
export interface AppContextValue {
  readonly info: AppInfo | null;
  readonly capabilities: CapabilitySummary | null;
  readonly engines: EngineReport | null;
  readonly loading: boolean;
  readonly error: string | null;
  readonly refresh: () => Promise<void>;
  readonly refreshEngines: () => Promise<void>;
}

export const AppContext = createContext<AppContextValue | null>(null);
