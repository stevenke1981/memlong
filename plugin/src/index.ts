// TypeScript Thin Shim — Delegate all memory orchestration to the Rust MCP Server

interface ChatContext {
  projectPath?: string;
  projectId?: string;
  initialQuery?: string;
  mcp: McpClient;
  injectSystemPrompt: (text: string) => void;
}

interface MessageContext {
  userMessage: string;
  assistantMessage: string;
  projectId?: string;
  sessionId: string;
  mcp: McpClient;
}

interface SessionContext {
  projectId?: string;
  sessionId: string;
  mcp: McpClient;
}

export interface Memory {
  id: string;
  content: string;
  category: string;
  importance_score: number;
  score_final?: number;
}

interface McpClient {
  call(tool: string, params: Record<string, unknown>): Promise<unknown>;
}

// ─────────────────────────────────────────────────────────────
// OpenCode Plugin Core
// ─────────────────────────────────────────────────────────────
export default {
  name: "opencode-memory",
  version: "1.0.0",

  hooks: {
    /**
     * Session Start: retrieve relevant memories and inject into System Prompt
     */
    onChatStart: async (ctx: ChatContext): Promise<void> => {
      try {
        const queryText = ctx.initialQuery ?? ctx.projectPath ?? "";
        if (!queryText) return;

        const result = await ctx.mcp.call("search_memories", {
          query: queryText,
          top_k: 10,
          scope: ctx.projectId ? "Project" : "Global",
          project_id: ctx.projectId,
          min_importance: 0.3,
        });

        const memories = parseMemoriesResponse(result);
        if (memories.length === 0) return;

        ctx.injectSystemPrompt(formatMemoriesForInjection(memories));
      } catch (err) {
        console.error("[opencode-memory] onChatStart error:", err);
      }
    },

    /**
     * Turn Complete: extract and save new memories asynchronously
     */
    onMessageComplete: async (ctx: MessageContext): Promise<void> => {
      // Non-blocking: background run
      queueMicrotask(async () => {
        try {
          const conversationTurn = [
            `User: ${ctx.userMessage}`,
            `Assistant: ${ctx.assistantMessage}`,
          ].join("\n\n");

          await ctx.mcp.call("add_memory", {
            content: conversationTurn,
            scope: ctx.projectId ? "Project" : "Global",
            project_id: ctx.projectId,
            session_id: ctx.sessionId,
          });
        } catch (err) {
          console.error("[opencode-memory] onMessageComplete error:", err);
        }
      });
    },

    /**
     * Session End: trigger batch consolidation (decay and deduplication checks)
     */
    onSessionEnd: async (ctx: SessionContext): Promise<void> => {
      try {
        await ctx.mcp.call("consolidate_memories", {
          scope: ctx.projectId ? "Project" : "Global",
          project_id: ctx.projectId,
        });
      } catch (err) {
        console.error("[opencode-memory] onSessionEnd error:", err);
      }
    },
  },
};

/**
 * Formats memory context block for insertion into System Prompt
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

export function formatMemoriesForInjection(memories: unknown[]): string {
  const normalized = normalizeMemories(memories);
  const lines = normalized.map((memory, i) =>
    `${i + 1}. [${memory.category}] ${memory.content}`
  );
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
