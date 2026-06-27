import type { Memory } from "./index";
/**
 * Normalise an MCP tool call response into a `Memory[]`.
 * Handles direct arrays, `{ results: [...] }`, and MCP text-content responses.
 */
export declare function parseMemoriesResponse(value: unknown): Memory[];
/**
 * Format memories as a markdown block for system prompt injection.
 */
export declare function formatMemoriesForInjection(memories: unknown[]): string;
