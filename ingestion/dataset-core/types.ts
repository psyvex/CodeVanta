export type Language = 'javascript' | 'typescript' | 'python';

export type Confidence = 'definite' | 'likely' | 'possible' | 'suggestion';

export type ReviewCategory =
  | 'security'
  | 'bug'
  | 'readability'
  | 'maintainability'
  | 'architecture'
  | 'performance'
  | 'testing'
  | 'standards'
  | 'database'
  | 'orm'
  | 'framework';

export type Severity = 'critical' | 'high' | 'medium' | 'low' | 'info';

export type LicenseStatus = 'allowed' | 'review-required' | 'blocked' | 'unknown';

export interface RepositoryProvenance {
  host: 'github';
  repository: string;
  commitSha: string;
  sourceUrl: string;
  licenseSpdx?: string;
  licenseStatus: LicenseStatus;
  collectedAt: string;
}

export interface CodeChange {
  path: string;
  before: string;
  after: string;
  additions: number;
  deletions: number;
}

export interface ReviewSignal {
  category: ReviewCategory;
  confidence: Confidence;
  severity: Severity;
  summary: string;
  reviewerComment?: string | undefined;
}

export interface DatasetExample {
  id: string;
  language: Language;
  frameworks: string[];
  libraries: string[];
  databases: string[];
  tools: string[];
  changes: CodeChange[];
  signals: ReviewSignal[];
  provenance: RepositoryProvenance;
}
