import { useContext } from 'react';

import { AppContext } from '@/app/appContext';
import type { AppContextValue } from '@/app/appContext';

/** Returns the backend capability state. */
export function useAppInfo(): AppContextValue {
  const value = useContext(AppContext);
  if (!value) {
    throw new Error('useAppInfo must be used inside an AppInfoProvider');
  }
  return value;
}
