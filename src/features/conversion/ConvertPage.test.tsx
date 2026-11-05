import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { AppInfoProvider } from '@/app/AppInfoContext';
import { ConvertPage } from '@/features/conversion/ConvertPage';

vi.mock('@/lib/ipc', () => ({
  getAppInfo: vi.fn(),
  getCapabilities: vi.fn(),
  getEngineStatus: vi.fn(),
  inspectFiles: vi.fn(),
  startConversion: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}));

const { getAppInfo, getCapabilities, getEngineStatus, inspectFiles, startConversion } =
  await import('@/lib/ipc');
const { open } = await import('@tauri-apps/plugin-dialog');

function mockBackend(options: { availableConversions: boolean; engineCount: number }) {
  vi.mocked(getAppInfo).mockResolvedValue({
    name: 'Universal File Converter',
    version: '0.1.0',
    coreVersion: '0.1.0',
    hasAvailableConversions: options.availableConversions,
  });

  vi.mocked(getCapabilities).mockResolvedValue({
    capabilities: options.availableConversions
      ? [
          {
            sourceFormat: 'png',
            targetFormat: 'jpeg',
            engineId: 'imagemagick',
            engineVersionRange: { min: '7.1.0', max: '7.99.99' },
            status: 'experimental',
            validationStrategy: 'independent-jpeg-decoder',
            limitations: [],
            available: true,
          },
        ]
      : [],
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

function mockInspection() {
  vi.mocked(inspectFiles).mockResolvedValue([
    {
      path: '/home/user/photo.png',
      fileName: 'photo.png',
      parentDirectory: '/home/user',
      sizeBytes: 2048,
      format: 'png',
      detection: 'extension',
      extensionMismatch: false,
      readable: true,
    },
  ]);
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

  it('asks for a file before it offers a target format', async () => {
    mockBackend({ availableConversions: true, engineCount: 1 });
    renderPage();

    expect(await screen.findByText('No file selected yet')).toBeInTheDocument();
    expect(screen.queryByRole('combobox', { name: 'Format' })).not.toBeInTheDocument();
  });

  it('offers only the targets of the registry for the selected file', async () => {
    const user = userEvent.setup();
    mockBackend({ availableConversions: true, engineCount: 1 });
    mockInspection();
    vi.mocked(open).mockResolvedValue('/home/user/photo.png');

    renderPage();

    await user.click(screen.getByRole('button', { name: 'Add files' }));

    const select = await screen.findByRole('combobox', { name: 'Format' });

    expect(select).toBeInTheDocument();
    expect(within(select).getByRole('option', { name: 'JPEG' })).toBeInTheDocument();
    expect(within(select).getAllByRole('option')).toHaveLength(1);
  });

  it('starts the conversion with the target of the registry', async () => {
    const user = userEvent.setup();
    mockBackend({ availableConversions: true, engineCount: 1 });
    mockInspection();
    vi.mocked(open).mockResolvedValue('/home/user/photo.png');
    vi.mocked(startConversion).mockResolvedValue({
      jobId: 'job-1',
      status: 'queued',
      progress: { kind: 'indeterminate' },
      validationLevel: 'exists',
    });

    renderPage();

    await user.click(screen.getByRole('button', { name: 'Add files' }));
    await screen.findByRole('combobox', { name: 'Format' });

    await user.click(screen.getByRole('button', { name: /Convert file/i }));

    await waitFor(() => expect(startConversion).toHaveBeenCalledTimes(1));

    const request = vi.mocked(startConversion).mock.calls[0]?.[0];
    expect(request?.inputPath).toBe('/home/user/photo.png');
    expect(request?.targetFormat).toBe('jpeg');
    expect(request?.outputDirectory).toBe('/home/user');
  });

  it('does not offer a conversion for a file with an unknown format', async () => {
    const user = userEvent.setup();
    mockBackend({ availableConversions: true, engineCount: 1 });
    vi.mocked(inspectFiles).mockResolvedValue([
      {
        path: '/home/user/notes.qqq',
        fileName: 'notes.qqq',
        parentDirectory: '/home/user',
        sizeBytes: 10,
        format: null,
        detection: 'unknown',
        extensionMismatch: false,
        readable: true,
      },
    ]);
    vi.mocked(open).mockResolvedValue('/home/user/notes.qqq');

    renderPage();

    await user.click(screen.getByRole('button', { name: 'Add files' }));

    expect(
      await screen.findByText('No conversion is available for these files'),
    ).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /Convert file/i })).not.toBeInTheDocument();
  });

  it('marks a file that could not be read and never converts it', async () => {
    const user = userEvent.setup();
    mockBackend({ availableConversions: true, engineCount: 1 });
    vi.mocked(inspectFiles).mockResolvedValue([
      {
        path: '/home/user/photo.png',
        fileName: 'photo.png',
        parentDirectory: '/home/user',
        sizeBytes: 0,
        format: 'png',
        detection: 'unknown',
        extensionMismatch: false,
        readable: false,
      },
    ]);
    vi.mocked(open).mockResolvedValue('/home/user/photo.png');

    renderPage();

    await user.click(screen.getByRole('button', { name: 'Add files' }));

    expect(await screen.findByText('The file could not be read')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /Convert file/i })).not.toBeInTheDocument();
  });
});
