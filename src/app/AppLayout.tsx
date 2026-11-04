import { NavLink, Outlet } from 'react-router-dom';

import { AppStatusBar } from '@/app/AppStatusBar';
import { ThemeProvider } from '@/app/ThemeProvider';

/** Navigation destinations, in the order defined in `DESIGN.md` section 6. */
const NAV_ITEMS = [
  { to: '/convert', label: 'Convert' },
  { to: '/queue', label: 'Queue' },
  { to: '/history', label: 'History' },
  { to: '/engines', label: 'Engines' },
  { to: '/settings', label: 'Settings' },
] as const;

/**
 * The application shell: left navigation, main content area and the
 * bottom status bar.
 */
export function AppLayout() {
  return (
    <ThemeProvider>
      <div className="app-shell">
        <nav className="app-nav" aria-label="Primary">
          <div className="app-nav__brand">
            <span className="app-nav__title">File Converter</span>
            <span className="app-nav__subtitle">Local &amp; offline</span>
          </div>

          <ul className="app-nav__list">
            {NAV_ITEMS.map((item) => (
              <li key={item.to}>
                <NavLink
                  to={item.to}
                  className={({ isActive }) =>
                    isActive ? 'app-nav__link app-nav__link--active' : 'app-nav__link'
                  }
                >
                  {item.label}
                </NavLink>
              </li>
            ))}
          </ul>
        </nav>

        <div className="app-main">
          <main className="app-content" id="main-content">
            <Outlet />
          </main>
          <AppStatusBar />
        </div>
      </div>
    </ThemeProvider>
  );
}
