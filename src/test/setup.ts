import '@testing-library/jest-dom/vitest';

/**
 * Global test setup.
 *
 * The frontend tests run in a jsdom environment without a Tauri IPC
 * channel. Tests that need backend data mock the `ipc` module instead
 * of relying on a real backend.
 */
