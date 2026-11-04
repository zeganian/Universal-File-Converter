import { useState } from 'react';

import { PageHeader } from '@/components/PageHeader';
import { readPreference, writePreference } from '@/app/theme';
import type { ThemePreference } from '@/app/theme';

/**
 * The Settings screen.
 *
 * Only settings that are actually honoured by the application are
 * offered. A setting that has no effect is worse than a missing one.
 */
export function SettingsPage() {
  const [theme, setTheme] = useState<ThemePreference>(readPreference);

  function handleThemeChange(next: ThemePreference): void {
    setTheme(next);
    writePreference(next);
  }

  return (
    <div className="page">
      <PageHeader title="Settings" description="Preferences for this device." />

      <section className="panel" aria-labelledby="appearance-heading">
        <h2 id="appearance-heading" className="panel__title">
          Appearance
        </h2>

        <div className="field">
          <label className="field__label" htmlFor="theme-select">
            Theme
          </label>
          <select
            id="theme-select"
            className="field__control"
            value={theme}
            onChange={(event) => handleThemeChange(event.target.value as ThemePreference)}
          >
            <option value="system">Follow system</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
          <p className="field__hint">
            Following the system preference keeps the window consistent with the rest of your
            desktop.
          </p>
        </div>
      </section>

      <section className="panel" aria-labelledby="conversion-heading">
        <h2 id="conversion-heading" className="panel__title">
          Conversion
        </h2>
        <p className="panel__meta">
          The default output folder, the collision policy and the concurrency limit become available
          once the first conversion can be started.
        </p>
      </section>

      <section className="panel" aria-labelledby="privacy-heading">
        <h2 id="privacy-heading" className="panel__title">
          Privacy
        </h2>
        <p className="panel__meta">
          Conversion history is disabled by default, so no file path is stored unless you turn it
          on. The application never uploads files and collects no telemetry.
        </p>
      </section>
    </div>
  );
}
