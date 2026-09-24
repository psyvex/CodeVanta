import type { JavaScriptEcosystemContext } from './detector.js';

export interface RustEcosystemContext {
  runtime?: string;
  technologies: string[];
}

export function toEngineEcosystemContext(
  context: JavaScriptEcosystemContext,
): RustEcosystemContext {
  return {
    ...(context.runtime ? { runtime: context.runtime } : {}),
    technologies: context.technologies.map(({ id }) => id),
  };
}
