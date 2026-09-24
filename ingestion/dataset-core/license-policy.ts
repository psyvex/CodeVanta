import type { LicenseStatus } from './types.js';

/**
 * Conservative default policy. Dataset ingestion should never silently treat an
 * unknown or custom license as training-safe.
 */
export const DEFAULT_ALLOWED_SPDX = new Set([
  'MIT',
  'Apache-2.0',
  'BSD-2-Clause',
  'BSD-3-Clause',
  'ISC',
]);

export function classifyLicense(spdx: string | undefined): LicenseStatus {
  if (!spdx) return 'unknown';
  if (DEFAULT_ALLOWED_SPDX.has(spdx)) return 'allowed';
  if (spdx === 'NOASSERTION' || spdx === 'LicenseRef') return 'review-required';
  if (spdx.startsWith('GPL-') || spdx.startsWith('AGPL-')) return 'review-required';
  return 'review-required';
}
