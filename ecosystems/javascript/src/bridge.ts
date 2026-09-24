import type { JavaScriptEcosystemContext } from './detector.js';

export interface RustTechnologyEvidence {
  kind: string;
  source: string;
  detail: string;
}

export interface RustTechnologyContext {
  id: string;
  confidence: 'definite' | 'likely' | 'possible';
  evidence: RustTechnologyEvidence[];
}

export interface RustEcosystemContext {
  runtime?: string;
  technologies: RustTechnologyContext[];
}

export function toEngineEcosystemContext(
  context: JavaScriptEcosystemContext,
): RustEcosystemContext {
  return {
    ...(context.runtime ? { runtime: context.runtime } : {}),
    technologies: context.technologies.map(({ id, confidence, evidence }) => ({
      id,
      confidence,
      evidence,
    })),
  };
}
