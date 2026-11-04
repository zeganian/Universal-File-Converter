import { useCallback, useRef, useState } from 'react';

import { PageHeader } from '@/components/PageHeader';
import { EmptyState } from '@/components/EmptyState';
import { useAppInfo } from '@/app/useAppInfo';

/**
 * The drop zone.
 *
 * This milestone establishes the foundation only: the zone accepts
 * files and lists their names, but it does not start a conversion.
 * That is deliberate — no conversion can be performed yet, and the UI
 * must not pretend otherwise (`RULES.md` section 1).
 *
 * The names come from the web view's file list, which does not expose
 * the path on the disk. Resolving the real path belongs to the file
 * intake milestone and goes through the Rust command boundary.
 */
export function DropZone({ onFilesSelected }: { onFilesSelected: (paths: string[]) => void }) {
  const [isDragging, setIsDragging] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const handleDragOver = useCallback((event: React.DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((event: React.DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback(
    (event: React.DragEvent<HTMLDivElement>) => {
      event.preventDefault();
      setIsDragging(false);

      const paths = Array.from(event.dataTransfer.files).map((file) => file.name);
      if (paths.length > 0) {
        onFilesSelected(paths);
      }
    },
    [onFilesSelected],
  );

  const handleBrowse = useCallback(() => {
    inputRef.current?.click();
  }, []);

  const handleInputChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const files = Array.from(event.target.files ?? []);
      if (files.length > 0) {
        onFilesSelected(files.map((file) => file.name));
      }
    },
    [onFilesSelected],
  );

  return (
    <div
      className={isDragging ? 'drop-zone drop-zone--active' : 'drop-zone'}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
      data-testid="drop-zone"
    >
      <div className="drop-zone__content">
        <h2 className="drop-zone__title">Drop files here</h2>
        <p className="drop-zone__description">
          Drag files onto this window, or use the button below to browse.
        </p>
        <button type="button" className="button button--primary" onClick={handleBrowse}>
          Add files
        </button>
        <input
          ref={inputRef}
          type="file"
          multiple
          className="sr-only"
          aria-label="Add files"
          onChange={handleInputChange}
        />
      </div>
    </div>
  );
}

/**
 * The Convert screen.
 *
 * It shows the drop zone, the detected capabilities and the reason why
 * no conversion can be started yet.
 */
export function ConvertPage() {
  const { capabilities, engines, loading, error } = useAppInfo();
  const [selectedFiles, setSelectedFiles] = useState<string[]>([]);

  const hasAvailableConversion = capabilities?.hasAvailableConversions ?? false;
  const availableCount = capabilities?.availableCount ?? 0;
  const engineCount = engines?.engines.length ?? 0;

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

      <DropZone onFilesSelected={setSelectedFiles} />

      {selectedFiles.length > 0 ? (
        <section className="panel" aria-labelledby="selected-files-heading">
          <h2 id="selected-files-heading" className="panel__title">
            Selected files
          </h2>
          <ul className="file-list">
            {selectedFiles.map((name) => (
              <li key={name} className="file-list__item">
                {name}
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
        ) : hasAvailableConversion ? (
          <p className="panel__meta">
            {availableCount} {availableCount === 1 ? 'conversion is' : 'conversions are'} available
            on this device.
          </p>
        ) : (
          <EmptyState
            title="No conversion is available yet"
            description={
              engineCount === 0
                ? 'No conversion engine has been detected. Conversions become selectable once an engine is available on this device.'
                : 'The detected engines cannot convert any file pair yet. Conversion pairs become selectable once an engine implementation and its tests exist.'
            }
          />
        )}
      </section>
    </div>
  );
}
