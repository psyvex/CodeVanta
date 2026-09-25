import type { RawPullRequest } from './github-types.js';
import type { Confidence, ReviewCategory, ReviewSignal, Severity } from './types.js';

/**
 * A signal a rule-based heuristic proposed from mined PR evidence -- never a
 * finished ReviewSignal. `confidence` is capped below 'definite' because no
 * heuristic here reads the actual code change, only PR title and review
 * comment text; `rule` and `evidence` exist so a human reviewer can check
 * the proposal against its source in seconds instead of re-deriving it.
 * Promoting a ProposedSignal into a ReviewSignal (dropping these two
 * fields) is a deliberate, separate, human step -- see promote.ts.
 */
export interface ProposedSignal {
  category: ReviewCategory;
  confidence: Confidence;
  severity: Severity;
  summary: string;
  reviewerComment?: string | undefined;
  rule: string;
  evidence: string;
}

interface KeywordRule {
  category: ReviewCategory;
  severity: Severity;
  keywords: string[];
}

// Confidence and severity here are deliberately conservative starting
// points, not measurements: a human reviewer is expected to correct them,
// not merely rubber-stamp them.
const TITLE_RULES: KeywordRule[] = [
  {
    category: 'security',
    severity: 'high',
    keywords: ['security', 'vulnerability', 'cve-', 'xss', 'sql injection', 'csrf', 'exploit'],
  },
  {
    category: 'bug',
    severity: 'medium',
    keywords: ['fix', 'bug', 'crash', 'regression', 'null pointer', 'npe'],
  },
  {
    category: 'performance',
    severity: 'medium',
    keywords: ['performance', 'perf', 'slow', 'optimi', 'n+1'],
  },
  {
    category: 'testing',
    severity: 'low',
    keywords: ['test', 'coverage', 'spec'],
  },
];

// The same keyword families, but matched against a human reviewer's own
// comment text rather than a PR title -- stronger evidence, since a person
// already flagged the issue, so these get a higher starting confidence.
const REVIEW_COMMENT_RULES: KeywordRule[] = TITLE_RULES;

function matchKeyword(text: string, keywords: string[]): string | null {
  const lower = text.toLowerCase();
  return keywords.find((keyword) => lower.includes(keyword)) ?? null;
}

function truncate(text: string, maxLength: number): string {
  return text.length > maxLength ? `${text.slice(0, maxLength)}...` : text;
}

function proposeFromTitle(raw: RawPullRequest): ProposedSignal[] {
  const proposals: ProposedSignal[] = [];

  for (const rule of TITLE_RULES) {
    const matched = matchKeyword(raw.title, rule.keywords);
    if (!matched) continue;

    proposals.push({
      category: rule.category,
      confidence: 'possible',
      severity: rule.severity,
      summary: `PR title suggests a ${rule.category} change: "${raw.title}"`,
      rule: `title-keyword:${matched}`,
      evidence: raw.title,
    });
  }

  return proposals;
}

function proposeFromReviewComments(raw: RawPullRequest): ProposedSignal[] {
  const proposals: ProposedSignal[] = [];

  for (const comment of raw.reviewComments) {
    for (const rule of REVIEW_COMMENT_RULES) {
      const matched = matchKeyword(comment.body, rule.keywords);
      if (!matched) continue;

      proposals.push({
        category: rule.category,
        confidence: 'likely',
        severity: rule.severity,
        summary: `Reviewer comment suggests a ${rule.category} finding${comment.path ? ` in ${comment.path}` : ''}.`,
        reviewerComment: comment.body,
        rule: `review-comment-keyword:${matched}`,
        evidence: truncate(comment.body, 300),
      });
    }
  }

  return proposals;
}

function deduplicate(proposals: ProposedSignal[]): ProposedSignal[] {
  const seen = new Set<string>();
  return proposals.filter((proposal) => {
    const key = `${proposal.category}:${proposal.rule}:${proposal.evidence}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
}

/**
 * Proposes candidate ReviewSignals from a mined pull request's title and
 * review comments. Every proposal must be reviewed by a human (or an
 * explicitly human-supervised process) before it can become part of a
 * DatasetExample -- this function only narrows down what to look at.
 */
export function proposeSignals(raw: RawPullRequest): ProposedSignal[] {
  return deduplicate([...proposeFromTitle(raw), ...proposeFromReviewComments(raw)]);
}

/**
 * Strips a proposal's rule/evidence bookkeeping to produce the ReviewSignal
 * it becomes once a human has approved (and, typically, corrected) it. Used
 * by promote.ts once a reviewer has edited a label.ts review file in place.
 */
export function toReviewSignal(proposal: ProposedSignal): ReviewSignal {
  const { rule: _rule, evidence: _evidence, ...signal } = proposal;
  return signal;
}
