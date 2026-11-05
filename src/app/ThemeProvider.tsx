import { useEffect } from 'react';
import type { ReactNode } from 'react';

import { readPreference } from '@/app/theme';

/**
 * Applies the theme preference to the document element.
 *
 * `data-theme` is only set for an explicit choice. Without it the
 * stylesheet follows the operating system preference through
 * `prefers-color-scheme` (`DESIGN.md` section 2).
 */
export function ThemeProvider({ children }: { children: ReactNode }) {
  useEffect(() => {
    const preference = readPreference();
    const root = document.documentElement;

    if (preference === 'system') {
      root.removeAttribute('data-theme');
    } else {
      root.setAttribute('data-theme', preference);
    }
  }, []);

  return <>{children}</>;
}
