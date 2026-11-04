import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { AppInfoProvider } from '@/app/AppInfoContext';
import { ConvertPage } from '@/features/conversion/ConvertPage';

vi.mock('@/lib/ipc', () => ({
  getAppInfo: vi.fn(),
  getCapabilities: vi.fn(),
  getEngineStatus: vi.fn(),
}));

const { getAppInfo, getCapabilities, getEngineStatus } = await import('@/lib/ipc');

function mockBackend(options: { availableConversions: boolean; engineCount: number }) {
  vi.mocked(getAppInfo).mockResolvedValue({
    name: 'Universal File Converter',
    version: '0.1.0',
    coreVersion: '0.1.0',
    hasAvailableConversions: options.availableConversions,
  });

  vi.mocked(getCapabilities).mockResolvedValue({
    capabilities: [],
    availableCount: options.availableConversions ? 1 : 0,
    hasAvailableConversions: options.availableConversions,
  });

  vi.mocked(getEngineStatus).mockResolvedValue({
    engines: Array.from({ length: options.engineCount }, (_unused, index) => ({
      engineId: `engine-${index}`,
      displayName: `Engine ${index}`,
      availability: 'missing' as const,
      source: null,
      version: null,
      installationHint: null,
      reason: null,
    })),
    availableCount: 0,
  });
}

function renderPage() {
  return render(
    <MemoryRouter>
      <AppInfoProvider>
        <ConvertPage />
      </AppInfoProvider>
    </MemoryRouter>,
  );
}

describe('ConvertPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('shows the drop zone with an add files action', async () => {
    mockBackend({ availableConversions: false, engineCount: 0 });
    renderPage();

    expect(await screen.findByTestId('drop-zone')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Add files' })).toBeInTheDocument();
  });

  it('states honestly that no conversion is available yet', async () => {
    mockBackend({ availableConversions: false, engineCount: 0 });
    renderPage();

    expect(await screen.findByText('No conversion is available yet')).toBeInTheDocument();
  });

  it('explains that no engine has been detected when none is registered', async () => {
    mockBackend({ availableConversions: false, engineCount: 0 });
    renderPage();

    expect(await screen.findByText(/No conversion engine has been detected/i)).toBeInTheDocument();
  });

  it('does not claim a conversion is available when the registry is empty', async () => {
    mockBackend({ availableConversions: false, engineCount: 2 });
    renderPage();

    expect(await screen.findByText(/cannot convert any file pair yet/i)).toBeInTheDocument();
    expect(screen.queryByText(/^\d+ conversions? (is|are) available/i)).not.toBeInTheDocument();
  });

  it('reports the available count when the backend offers conversions', async () => {
    mockBackend({ availableConversions: true, engineCount: 1 });
    renderPage();

    expect(await screen.findByText(/1 conversion is available/i)).toBeInTheDocument();
    expect(screen.queryByText('No conversion is available yet')).not.toBeInTheDocument();
  });
});
