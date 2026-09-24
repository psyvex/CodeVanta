import type { DatasetExample } from './types.js';

export function serializeDataset(examples: DatasetExample[]): string {
  return examples.map((example) => JSON.stringify(example)).join('\n') + (examples.length ? '\n' : '');
}

export function parseDataset(input: string): DatasetExample[] {
  return input
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line, index) => parseExample(line, index + 1));
}

function parseExample(line: string, lineNumber: number): DatasetExample {
  try {
    return JSON.parse(line) as DatasetExample;
  } catch (error) {
    throw new Error(`Invalid dataset JSON on line ${lineNumber}`, { cause: error });
  }
}
