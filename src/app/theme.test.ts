import { describe, expect, it } from 'vitest';

import { readPreference, writePreference } from '@/app/theme';

describe('theme preference', () => {
  it('defaults to the system preference when nothing is stored', () => {
    window.localStorage.clear();
    expect(readPreference()).toBe('system');
  });

  it('round-trips an explicit preference', () => {
    writePreference('dark');
    expect(readPreference()).toBe('dark');

    writePreference('light');
    expect(readPreference()).toBe('light');
  });

  it('ignores a corrupted stored value', () => {
    window.localStorage.setItem('ufc.theme', 'not-a-theme');
    expect(readPreference()).toBe('system');
  });
});
