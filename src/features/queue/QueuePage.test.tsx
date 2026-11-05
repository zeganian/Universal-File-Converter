import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { AppInfoProvider } from '@/app/AppInfoContext';
import { QueuePage } from '@/features/queue/QueuePage';

vi.mock('@/lib/ipc', () => ({
  getAppInfo: vi.fn(),
  getCapabilities: vi.fn(),
  getEngineStatus: vi.fn(),
  listJobs: vi.fn(),
  cancelJob: vi.fn(),
}));

const { getAppInfo, getCapabilities, getEngineStatus, listJobs } = await import('@/lib/ipc');

function mockBackend() {
  vi.mocked(getAppInfo).mockResolvedValue({
    name: 'Universal File Converter',
    version: '0.1.0',
    coreVersion: '0.1.0',
    hasAvailableConversions: true,
  });

  vi.mocked(getCapabilities).mockResolvedValue({
    capabilities: [],
    availableCount: 1,
    hasAvailableConversions: true,
  });

  vi.mocked(getEngineStatus).mockResolvedValue({
    engines: [],
    availableCount: 1,
  });
}

function renderPage() {
  return render(
    <MemoryRouter>
      <AppInfoProvider>
        <QueuePage />
      </AppInfoProvider>
    </MemoryRouter>,
  );
}

describe('QueuePage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockBackend();
  });

  it('explains that the queue is empty', async () => {
    vi.mocked(listJobs).mockResolvedValue([]);
    renderPage();

    expect(await screen.findByText('The queue is empty')).toBeInTheDocument();
  });

  it('shows the state, the engine and the output of a finished job', async () => {
    vi.mocked(listJobs).mockResolvedValue([
      {
        jobId: 'job-1',
        status: 'succeeded',
        outputPath: '/home/user/photo.jpg',
        engineId: 'imagemagick',
        engineVersion: '7.1.2-34',
        warnings: [],
        error: null,
        validationLevel: 'parsed',
        progress: { kind: 'indeterminate' },
      },
    ]);

    renderPage();

    expect(await screen.findByText('photo.jpg')).toBeInTheDocument();
    expect(screen.getByText('Completed')).toBeInTheDocument();
    expect(screen.getByText('imagemagick 7.1.2-34')).toBeInTheDocument();
    expect(screen.getByText('/home/user/photo.jpg')).toBeInTheDocument();
  });

  it('shows the reason of a failed job', async () => {
    vi.mocked(listJobs).mockResolvedValue([
      {
        jobId: 'job-2',
        status: 'failed',
        progress: { kind: 'indeterminate' },
        validationLevel: 'exists',
        engineId: 'imagemagick',
        engineVersion: null,
        warnings: [],
        error: {
          code: 'missingDependency',
          message: 'ImageMagick was not found on this device.',
          remediation: 'Install ImageMagick 7.',
          detail: null,
        },
      },
    ]);

    renderPage();

    expect(await screen.findByText('Failed')).toBeInTheDocument();
    expect(screen.getByText('ImageMagick was not found on this device.')).toBeInTheDocument();
    expect(screen.getByText('Install ImageMagick 7.')).toBeInTheDocument();
  });

  it('shows the source file of a job that has no output yet', async () => {
    vi.mocked(listJobs).mockResolvedValue([
      {
        jobId: 'job-4',
        status: 'failed',
        sourceName: 'holiday photo.png',
        progress: { kind: 'indeterminate' },
        validationLevel: 'exists',
        warnings: [],
        error: {
          code: 'internal',
          message: 'The conversion failed.',
          remediation: null,
          detail: null,
        },
      },
    ]);

    renderPage();

    expect(await screen.findByText('holiday photo.png')).toBeInTheDocument();
  });

  it('offers a cancel action only for a running job', async () => {
    vi.mocked(listJobs).mockResolvedValue([
      {
        jobId: 'job-3',
        status: 'running',
        progress: { kind: 'indeterminate' },
        validationLevel: 'exists',
        warnings: [],
      },
    ]);

    renderPage();

    expect(await screen.findByRole('button', { name: 'Cancel' })).toBeInTheDocument();
  });
});
