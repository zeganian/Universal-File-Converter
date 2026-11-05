/**
 * Typed bridge to the Rust backend.
 *
 * The frontend never calls `invoke` directly: it goes through this
 * module so that every command has a single, typed definition and a
 * single error shape.
 *
 * Command payloads are validated in Rust even though the frontend also
 * validates them (`ARCHITECTURE.md` section 11).
 */

import { invoke } from '@tauri-apps/api/core';

import type { ConversionErrorCode } from '@/types/generated/ConversionErrorCode';
import type { ConversionCapability } from '@/types/generated/ConversionCapability';
import type { ConversionRequest } from '@/types/generated/ConversionRequest';
import type { ConversionResult } from '@/types/generated/ConversionResult';
import type { FileInspection } from '@/types/generated/FileInspection';

/**
 * Structured error returned by a backend command.
 *
 * Mirrors `CommandError` in `src-tauri/src/commands/app.rs`.
 */
export interface CommandError {
  readonly code: ConversionErrorCode;
  readonly message: string;
  readonly remediation?: string | null;
  readonly detail?: string | null;
}

/** Version information about the running application. */
export interface AppInfo {
  readonly name: string;
  readonly version: string;
  readonly coreVersion: string;
  readonly hasAvailableConversions: boolean;
}

/** A summary of the conversion capabilities known to the application. */
export interface CapabilitySummary {
  readonly capabilities: readonly ConversionCapability[];
  readonly availableCount: number;
  readonly hasAvailableConversions: boolean;
}

/** Availability of an engine on the current machine. */
export type EngineAvailability = 'available' | 'missing' | 'incompatibleVersion';

/** Where an engine installation was found. */
export type EngineSource = 'bundled' | 'system';

/** Detected installation of a conversion engine. */
export interface EngineStatus {
  readonly engineId: string;
  readonly displayName: string;
  readonly availability: EngineAvailability;
  readonly source?: EngineSource | null;
  readonly version?: string | null;
  readonly installationHint?: string | null;
  readonly reason?: CommandError | null;
}

/** A report about the engines known to the application. */
export interface EngineReport {
  readonly engines: readonly EngineStatus[];
  readonly availableCount: number;
}

/** Error thrown when a backend command fails. */
export class BackendError extends Error {
  readonly code: ConversionErrorCode;
  readonly remediation: string | null;
  readonly detail: string | null;

  constructor(error: CommandError) {
    super(error.message);
    this.name = 'BackendError';
    this.code = error.code;
    this.remediation = error.remediation ?? null;
    this.detail = error.detail ?? null;
  }
}

/**
 * Normalises an unknown rejection value into a `BackendError`.
 *
 * The backend always answers with the structured error shape, but a
 * transport level failure (for example a missing IPC channel) can
 * reject with anything at all. Those are reported as `internal` rather
 * than being swallowed.
 */
export function toBackendError(value: unknown): BackendError {
  if (value !== null && typeof value === 'object' && 'code' in value && 'message' in value) {
    return new BackendError(value as CommandError);
  }

  return new BackendError({
    code: 'internal',
    message: value instanceof Error ? value.message : 'The backend could not be reached.',
  });
}

/** Invokes a backend command and normalises errors. */
async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error: unknown) {
    throw toBackendError(error);
  }
}

/** Returns basic information about the running application. */
export function getAppInfo(): Promise<AppInfo> {
  return call<AppInfo>('get_app_info');
}

/** Returns the conversion capability registry. */
export function getCapabilities(): Promise<CapabilitySummary> {
  return call<CapabilitySummary>('get_capabilities');
}

/** Returns the status of every known conversion engine. */
export function getEngineStatus(): Promise<EngineReport> {
  return call<EngineReport>('get_engine_status');
}

/** Detects the engines again, for example after installing one. */
export function refreshEngineStatus(): Promise<EngineReport> {
  return call<EngineReport>('refresh_engine_status');
}

/**
 * Starts a conversion.
 *
 * The conversion runs in the background. The returned result carries
 * the initial state of the job; every later state change arrives as a
 * `job://updated` event.
 */
export function startConversion(request: ConversionRequest): Promise<ConversionResult> {
  return call<ConversionResult>('start_conversion', { request });
}

/** Returns the state of a job. */
export function getJob(jobId: string): Promise<ConversionResult | null> {
  return call<ConversionResult | null>('get_job', { jobId });
}

/** Returns the state of every job. */
export function listJobs(): Promise<ConversionResult[]> {
  return call<ConversionResult[]>('list_jobs');
}

/** Cancels a running job. */
export function cancelJob(jobId: string): Promise<boolean> {
  return call<boolean>('cancel_job', { jobId });
}

/** Inspects the files that the user selected. */
export function inspectFiles(paths: readonly string[]): Promise<FileInspection[]> {
  return call<FileInspection[]>('inspect_files', { paths: [...paths] });
}

/** The event that carries a state change of a job. */
export const JOB_UPDATED_EVENT = 'job://updated';
