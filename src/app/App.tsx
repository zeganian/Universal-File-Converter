import { Navigate, Route, Routes } from 'react-router-dom';

import { AppInfoProvider } from '@/app/AppInfoContext';
import { AppLayout } from '@/app/AppLayout';
import { ConvertPage } from '@/features/conversion/ConvertPage';
import { EnginesPage } from '@/features/engines/EnginesPage';
import { HistoryPage } from '@/features/history/HistoryPage';
import { QueuePage } from '@/features/queue/QueuePage';
import { SettingsPage } from '@/features/settings/SettingsPage';

/**
 * Application routes.
 *
 * The five destinations match the navigation defined in `DESIGN.md`
 * section 6. Results are shown inside the Queue screen rather than as
 * a separate destination.
 */
export function App() {
  return (
    <AppInfoProvider>
      <Routes>
        <Route element={<AppLayout />}>
          <Route index element={<Navigate to="/convert" replace />} />
          <Route path="/convert" element={<ConvertPage />} />
          <Route path="/queue" element={<QueuePage />} />
          <Route path="/history" element={<HistoryPage />} />
          <Route path="/engines" element={<EnginesPage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="*" element={<Navigate to="/convert" replace />} />
        </Route>
      </Routes>
    </AppInfoProvider>
  );
}
