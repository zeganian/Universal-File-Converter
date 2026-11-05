import { useCallback, useEffect, useMemo, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import { PageHeader } from '@/components/PageHeader';
import { EmptyState } from '@/components/EmptyState';
import { Badge } from '@/components/Badge';
import { useAppInfo } from '@/app/useAppInfo';
import { inspectFiles, startConversion } from '@/lib/ipc';
import type { CapabilitySummary } from '@/lib/ipc';
import { formatBytes } from '@/lib/format';
import type { ConversionCapability } from '@/types/generated/ConversionCapability';
import type { FileInspection } from '@/types/generated/FileInspection';
import type { OverwritePolicy } from '@/types/generated/OverwritePolicy';

/**
 * The file intake.
 *
 * The paths come from the file dialog of the operating system or from
 * a drag and drop onto the window. Both are handled by Tauri, so the
 * application receives real paths and never guesses them from a file
 * name.
 */
export function FileIntake({
  onFilesSelected,
  disabled,
}: {
  onFilesSelected: (paths: string[]) => void;
  disabled?: boolean;
}) {
  const [isDragging, setIsDragging] = useState(false);
  const [failure, setFailure] = useState<string | null>(null);

  const handleBrowse = useCallback(async () => {
    setFailure(null);

    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selection = await open({ multiple: true, directory: false });

      if (!selection) return;

      const paths = Array.isArray(selection) ? selection : [selection];
      if (paths.length > 0) {
        onFilesSelected(paths);
      }
    } catch {
      setFailure(
        'The file dialog is only available in the desktop application. Run the application with "pnpm tauri:dev".',
      );
    }
  }, [onFilesSelected]);

  // The drag and drop of the operating system, including the paths.
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    void (async () => {
      try {
        const { getCurrentWebview } = await import('@tauri-apps/api/webview');
        const webview = getCurrentWebview();

        unlisten = await webview.onDragDropEvent((event) => {
          if (event.payload.type === 'over') {
            setIsDragging(true);
            return;
          }

          setIsDragging(false);

          if (event.payload.type === 'drop' && event.payload.paths.length > 0) {
            onFilesSelected(event.payload.paths);
          }
        });
      } catch {
        // The browser does not provide the desktop drag and drop. The
        // file dialog stays available.
      }
    })();

    return () => unlisten?.();
  }, [onFilesSelected]);

  return (
    <div
      className={isDragging ? 'drop-zone drop-zone--active' : 'drop-zone'}
      data-testid="drop-zone"
    >
      <div className="drop-zone__content">
        <h2 className="drop-zone__title">Drop files here</h2>
        <p className="drop-zone__description">
          Drag files onto this window, or use the button below to browse.
        </p>
        <button
          type="button"
          className="button button--primary"
          onClick={() => void handleBrowse()}
          disabled={disabled}
        >
          Add files
        </button>
        {failure ? (
          <p className="field__hint" role="alert">
            {failure}
          </p>
        ) : null}
      </div>
    </div>
  );
}

/**
 * The options that every selected file accepts for a target format.
 *
 * The registry declares the options of every pair, so the interface
 * never offers a setting that the engine would ignore. An option is
 * only offered when it has an effect on every selected file.
 */
function acceptedOptionsOf(
  capabilities: CapabilitySummary | null,
  files: readonly FileInspection[],
  target: string,
): { quality: boolean; background: boolean } {
  if (!capabilities || files.length === 0 || target === '') {
    return { quality: false, background: false };
  }

  const pairs: ConversionCapability[] = [];

  for (const file of files) {
    const pair = capabilities.capabilities.find(
      (capability) =>
        capability.available &&
        capability.sourceFormat === file.format &&
        capability.targetFormat === target,
    );

    if (!pair) return { quality: false, background: false };
    pairs.push(pair);
  }

  return {
    quality: pairs.every((pair) => pair.options.quality),
    background: pairs.every((pair) => pair.options.background),
  };
}

/**
 * The Convert screen.
 *
 * The list of target formats comes from the capability registry of the
 * backend. A target is only offered when the conversion is implemented
 * and its engine is available on this device
 * (`SUPPORTED_FORMATS.md`, "Registry rules").
 */
export function ConvertPage() {
  const { capabilities, engines, loading, error } = useAppInfo();
  const navigate = useNavigate();

  const [inspections, setInspections] = useState<FileInspection[]>([]);
  const [selectedTarget, setSelectedTarget] = useState<string | null>(null);
  const [outputDirectory, setOutputDirectory] = useState('');
  const [policy, setPolicy] = useState<OverwritePolicy>('rename');
  const [quality, setQuality] = useState(92);
  const [background, setBackground] = useState('white');
  const [failure, setFailure] = useState<string | null>(null);
  const [isStarting, setIsStarting] = useState(false);

  const engineCount = engines?.engines.length ?? 0;
  const hasAvailableConversions = capabilities?.hasAvailableConversions ?? false;

  const addFiles = useCallback((paths: string[]) => {
    setFailure(null);

    void (async () => {
      try {
        const inspected = await inspectFiles(paths);
        setInspections(inspected);

        const first = inspected[0];
        if (first?.parentDirectory) {
          setOutputDirectory((current) => current || first.parentDirectory || '');
        }
      } catch (cause: unknown) {
        setFailure(cause instanceof Error ? cause.message : 'The files could not be inspected.');
      }
    })();
  }, []);

  // The files that can be converted at all. A file that could not be
  // read is shown in the list, but it is never sent to the backend.
  const convertible = useMemo(
    () => inspections.filter((inspection) => inspection.readable),
    [inspections],
  );

  // The targets that work for every selected file.
  const targets = useMemo(() => {
    if (!capabilities || convertible.length === 0) return [];

    const perFile = convertible.map((inspection) => {
      const format = inspection.format;
      if (!format) return [] as string[];

      return capabilities.capabilities
        .filter((capability) => capability.available && capability.sourceFormat === format)
        .map((capability) => capability.targetFormat);
    });

    if (perFile.length === 0) return [];

    return perFile.reduce((common, targets) => common.filter((target) => targets.includes(target)));
  }, [capabilities, convertible]);

  // The target that is actually used. A selection that is not offered
  // anymore falls back to the first available target, so that the
  // selection never shows a format the backend does not support.
  const target = useMemo(
    () =>
      selectedTarget && targets.includes(selectedTarget) ? selectedTarget : (targets[0] ?? ''),
    [selectedTarget, targets],
  );

  // The options of the selected pair. The registry declares them, so
  // the interface never offers a setting that the engine ignores
  // (`SUPPORTED_FORMATS.md`, "Registry rules").
  const acceptedOptions = acceptedOptionsOf(capabilities, convertible, target);

  const canConvert = convertible.length > 0 && target !== '' && outputDirectory !== '';

  const start = useCallback(async () => {
    if (!canConvert) return;

    setIsStarting(true);
    setFailure(null);

    try {
      // Only the options that the selected pair accepts are sent, so
      // that the backend never receives a setting it would reject.
      const options: { quality?: number; background?: string } = {};

      if (acceptedOptions.quality) options.quality = quality;
      if (acceptedOptions.background) options.background = background;

      for (const inspection of convertible) {
        await startConversion({
          fileId: crypto.randomUUID(),
          inputPath: inspection.path,
          targetFormat: target,
          outputDirectory,
          overwritePolicy: policy,
          options,
        });
      }

      await navigate('/queue');
    } catch (cause: unknown) {
      setFailure(cause instanceof Error ? cause.message : 'The conversion could not be started.');
      setIsStarting(false);
    }
  }, [
    canConvert,
    convertible,
    target,
    outputDirectory,
    policy,
    quality,
    background,
    acceptedOptions,
    navigate,
  ]);

  const chooseOutputDirectory = useCallback(async () => {
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selection = await open({ directory: true, multiple: false });

      if (typeof selection === 'string') {
        setOutputDirectory(selection);
      }
    } catch {
      setFailure('The folder dialog is only available in the desktop application.');
    }
  }, []);

  return (
    <div className="page">
      <PageHeader
        title="Convert"
        description="Add files, choose a target format, and convert them locally on this device."
      />

      {error ? (
        <div className="alert alert--danger" role="alert">
          <strong>The backend could not be reached.</strong>
          <span>{error}</span>
        </div>
      ) : null}

      {failure ? (
        <div className="alert alert--danger" role="alert">
          <strong>The files could not be processed.</strong>
          <span>{failure}</span>
        </div>
      ) : null}

      <FileIntake onFilesSelected={addFiles} disabled={isStarting} />

      {inspections.length > 0 ? (
        <section className="panel" aria-labelledby="selected-files-heading">
          <h2 id="selected-files-heading" className="panel__title">
            Selected files
          </h2>
          <ul className="file-list">
            {inspections.map((inspection) => (
              <li key={inspection.path} className="file-list__item">
                <span className="file-list__name">{inspection.fileName}</span>
                <span className="file-list__meta">
                  {inspection.format ?? 'unknown format'} · {formatBytes(inspection.sizeBytes)}
                </span>
                {inspection.extensionMismatch ? (
                  <Badge tone="warning">Extension does not match the content</Badge>
                ) : null}
                {!inspection.readable ? (
                  <Badge tone="danger">The file could not be read</Badge>
                ) : null}
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      <section className="panel" aria-labelledby="target-format-heading">
        <h2 id="target-format-heading" className="panel__title">
          Target format
        </h2>

        {loading ? (
          <p className="panel__meta">Checking available conversions…</p>
        ) : !hasAvailableConversions ? (
          <EmptyState
            title="No conversion is available yet"
            description={
              engineCount === 0
                ? 'No conversion engine has been detected. Conversions become selectable once an engine is available on this device.'
                : 'The detected engines cannot convert any file pair yet. Conversion pairs become selectable once an engine implementation and its tests exist.'
            }
          />
        ) : convertible.length === 0 ? (
          <EmptyState
            title="No file selected yet"
            description="Add a file to see the formats it can be converted into."
          />
        ) : targets.length === 0 ? (
          <EmptyState
            title="No conversion is available for these files"
            description="The selected files cannot be converted with the engines of this device. Choose other files, or install a different engine."
          />
        ) : (
          <div className="field">
            <label className="field__label" htmlFor="target-select">
              Format
            </label>
            <select
              id="target-select"
              className="field__control"
              value={target}
              onChange={(event) => setSelectedTarget(event.target.value)}
            >
              {targets.map((format) => (
                <option key={format} value={format}>
                  {format.toUpperCase()}
                </option>
              ))}
            </select>
            <p className="field__hint">
              The format applies to every selected file. Only formats that the installed engines can
              produce are listed.
            </p>
          </div>
        )}
      </section>

      {convertible.length > 0 && targets.length > 0 ? (
        <section className="panel" aria-labelledby="output-heading">
          <h2 id="output-heading" className="panel__title">
            Output
          </h2>

          <div className="field">
            <label className="field__label" htmlFor="output-directory">
              Output folder
            </label>
            <div className="field__row">
              <input
                id="output-directory"
                className="field__control"
                value={outputDirectory}
                onChange={(event) => setOutputDirectory(event.target.value)}
              />
              <button type="button" className="button" onClick={() => void chooseOutputDirectory()}>
                Browse…
              </button>
            </div>
          </div>

          <div className="field">
            <label className="field__label" htmlFor="policy-select">
              If the file already exists
            </label>
            <select
              id="policy-select"
              className="field__control"
              value={policy}
              onChange={(event) => setPolicy(event.target.value as OverwritePolicy)}
            >
              <option value="rename">Write a new name (photo (2).jpg)</option>
              <option value="skip">Keep the existing file and report the conflict</option>
            </select>
            <p className="field__hint">
              An existing file is never replaced without an explicit decision.
            </p>
          </div>

          {acceptedOptions.quality ? (
            <div className="field">
              <label className="field__label" htmlFor="quality-range">
                Quality
              </label>
              <input
                id="quality-range"
                className="field__control"
                type="range"
                min={1}
                max={100}
                value={quality}
                onChange={(event) => setQuality(Number(event.target.value))}
              />
              <p className="field__hint">
                Higher quality means a larger file. Current value: {quality}.
              </p>
            </div>
          ) : null}

          {acceptedOptions.background ? (
            <div className="field">
              <label className="field__label" htmlFor="background-input">
                Background colour
              </label>
              <input
                id="background-input"
                className="field__control"
                type="text"
                value={background}
                onChange={(event) => setBackground(event.target.value)}
              />
              <p className="field__hint">
                The target format cannot store transparency. Transparent areas are flattened onto
                this colour. Use a colour name such as <code>white</code> or a hex value such as{' '}
                <code>#ffffff</code>.
              </p>
            </div>
          ) : null}

          <button
            type="button"
            className="button button--primary"
            onClick={() => void start()}
            disabled={!canConvert || isStarting}
          >
            Convert {convertible.length > 1 ? `${convertible.length} files` : 'file'}
          </button>
        </section>
      ) : null}
    </div>
  );
}
