import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { AppInfoProvider } from '@/app/AppInfoContext';
import { EnginesPage } from '@/features/engines/EnginesPage';

vi.mock('@/lib/ipc', () => ({
  getAppInfo: vi.fn(),
  getCapabilities: vi.fn(),
  getEngineStatus: vi.fn(),
}));

const { getAppInfo, getCapabilities, getEngineStatus } = await import('@/lib/ipc');

function renderPage() {
  return render(
    <MemoryRouter>
      <AppInfoProvider>
        <EnginesPage />
      </AppInfoProvider>
    </MemoryRouter>,
  );
}

describe('EnginesPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    vi.mocked(getAppInfo).mockResolvedValue({
      name: 'Universal File Converter',
      version: '0.1.0',
      coreVersion: '0.1.0',
      hasAvailableConversions: false,
    });

    vi.mocked(getCapabilities).mockResolvedValue({
      capabilities: [],
      availableCount: 0,
      hasAvailableConversions: false,
    });
  });

  it('states that no engine was detected when none is registered', async () => {
    vi.mocked(getEngineStatus).mockResolvedValue({ engines: [], availableCount: 0 });
    renderPage();

    expect(await screen.findByText('No engine detected')).toBeInTheDocument();
  });

  it('shows a detected engine with its version and source', async () => {
    vi.mocked(getEngineStatus).mockResolvedValue({
      engines: [
        {
          engineId: 'ffmpeg',
          displayName: 'FFmpeg',
          availability: 'available',
          source: 'system',
          version: '9.0.2',
          installationHint: null,
          reason: null,
        },
      ],
      availableCount: 1,
    });
    renderPage();

    expect(await screen.findByText('FFmpeg')).toBeInTheDocument();
    expect(screen.getByText('Available')).toBeInTheDocument();
    expect(screen.getByText('version 9.0.2')).toBeInTheDocument();
    expect(screen.getByText('system')).toBeInTheDocument();
  });

  it('explains a missing engine and how to resolve it', async () => {
    vi.mocked(getEngineStatus).mockResolvedValue({
      engines: [
        {
          engineId: 'imagemagick',
          displayName: 'ImageMagick',
          availability: 'missing',
          source: null,
          version: null,
          installationHint: null,
          reason: {
            code: 'missingDependency',
            message: 'ImageMagick was not found on this device.',
            remediation: 'Install the ImageMagick package for your system.',
            detail: null,
          },
        },
      ],
      availableCount: 0,
    });
    renderPage();

    expect(await screen.findByText('Not detected')).toBeInTheDocument();
    expect(screen.getByText('ImageMagick was not found on this device.')).toBeInTheDocument();
    expect(
      screen.getByText('Install the ImageMagick package for your system.'),
    ).toBeInTheDocument();
  });

  it('never uses colour alone to convey the engine state', async () => {
    vi.mocked(getEngineStatus).mockResolvedValue({
      engines: [
        {
          engineId: 'ffmpeg',
          displayName: 'FFmpeg',
          availability: 'available',
          source: 'system',
          version: '9.0.2',
          installationHint: null,
          reason: null,
        },
      ],
      availableCount: 1,
    });
    renderPage();

    // The state is spelled out in text, not only expressed through colour.
    expect(await screen.findByText('Available')).toBeInTheDocument();
  });
});
