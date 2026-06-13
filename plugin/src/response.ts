// Response normalisation helpers for OpenCode Memory plugin
// Kept separate to keep index.ts focused on lifecycle hooks only.

import type { Memory } from "./index";

/**
 * Normalise an MCP tool call response into a `Memory[]`.
 * Handles direct arrays, `{ results: [...] }`, and MCP text-content responses.
 */
export function parseMemoriesResponse(value: unknown): Memory[] {
  if (Array.isArray(value)) return normalizeMemories(value);
  if (!isRecord(value)) return [];

  if (Array.isArray(value.results)) return normalizeMemories(value.results);
  if (Array.isArray(value.content)) {
    for (const item of value.content) {
      if (!isRecord(item) || item.type !== "text" || typeof item.text !== "string") continue;
      try {
        const parsed = JSON.parse(item.text) as unknown;
        const memories = parseMemoriesResponse(parsed);
        if (memories.length > 0) return memories;
      } catch {
        // Ignore non-JSON MCP content blocks.
      }
    }
  }
  return [];
}

/**
 * Format memories as a markdown block for system prompt injection.
 */
export function formatMemoriesForInjection(memories: unknown[]): string {
  const normalized = normalizeMemories(memories);
  const lines = normalized.map((memory, i) => `${i + 1}. [${memory.category}] ${memory.content}`);
  return [
    "## Relevant Memory Context",
    "(From past sessions — use as background context)",
    ...lines,
    "",
  ].join("\n");
}

function normalizeMemories(values: unknown[]): Memory[] {
  const memories: Memory[] = [];
  for (const value of values) {
    const candidate = isRecord(value) && isRecord(value.memory) ? value.memory : value;
    if (!isRecord(candidate)) continue;
    if (typeof candidate.id !== "string") continue;
    if (typeof candidate.content !== "string" || typeof candidate.category !== "string") continue;
    const memory: Memory = {
      id: candidate.id,
      content: candidate.content,
      category: candidate.category,
      importance_score:
        typeof candidate.importance_score === "number" ? candidate.importance_score : 0,
    };
    if (typeof candidate.score_final === "number") {
      memory.score_final = candidate.score_final;
    }
    memories.push(memory);
  }
  return memories;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
