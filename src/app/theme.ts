/**
 * Theme preference helpers.
 *
 * These live outside the React components so that the fast refresh
 * plugin can treat component files as component-only modules.
 */

const STORAGE_KEY = 'ufc.theme';

/** The theme preferences the UI offers. */
export type ThemePreference = 'system' | 'light' | 'dark';

/** Every selectable theme preference, in display order. */
export const THEME_PREFERENCES: readonly ThemePreference[] = ['system', 'light', 'dark'];

/** Reads the stored theme preference, defaulting to `system`. */
export function readPreference(): ThemePreference {
  const stored = window.localStorage.getItem(STORAGE_KEY);
  if (stored === 'light' || stored === 'dark' || stored === 'system') {
    return stored;
  }
  return 'system';
}

/** Stores the theme preference. */
export function writePreference(preference: ThemePreference): void {
  window.localStorage.setItem(STORAGE_KEY, preference);
}
