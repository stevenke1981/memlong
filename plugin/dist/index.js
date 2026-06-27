"use strict";
// TypeScript Thin Shim — Delegate all memory orchestration to the Rust MCP Server
// Lifecycle-only. Response normalisation lives in ./response.ts.
Object.defineProperty(exports, "__esModule", { value: true });
const response_1 = require("./response");
// ─────────────────────────────────────────────────────────────
// OpenCode Plugin Core
// ─────────────────────────────────────────────────────────────
exports.default = {
    name: "ams",
    version: "1.0.0",
    hooks: {
        /**
         * Session Start: retrieve relevant memories and inject into System Prompt
         */
        onChatStart: async (ctx) => {
            try {
                const queryText = ctx.initialQuery ?? ctx.projectPath ?? "";
                if (!queryText)
                    return;
                const result = await ctx.mcp.call("search_memories", {
                    query: queryText,
                    top_k: 10,
                    scope: ctx.projectId ? "Project" : "Global",
                    project_id: ctx.projectId,
                    min_importance: 0.3,
                });
                const memories = (0, response_1.parseMemoriesResponse)(result);
                if (memories.length === 0)
                    return;
                ctx.injectSystemPrompt((0, response_1.formatMemoriesForInjection)(memories));
            }
            catch (err) {
                console.error("[ams] onChatStart error:", err);
            }
        },
        /**
         * Turn Complete: extract and save new memories asynchronously
         */
        onMessageComplete: async (ctx) => {
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
                }
                catch (err) {
                    console.error("[ams] onMessageComplete error:", err);
                }
            });
        },
        /**
         * Session End: trigger batch consolidation (decay and deduplication checks)
         */
        onSessionEnd: async (ctx) => {
            try {
                await ctx.mcp.call("consolidate_memories", {
                    scope: ctx.projectId ? "Project" : "Global",
                    project_id: ctx.projectId,
                });
            }
            catch (err) {
                console.error("[ams] onSessionEnd error:", err);
            }
        },
    },
};
