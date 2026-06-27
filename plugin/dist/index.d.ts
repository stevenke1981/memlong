export interface Memory {
    id: string;
    content: string;
    category: string;
    importance_score: number;
    score_final?: number;
}
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
interface McpClient {
    call(tool: string, params: Record<string, unknown>): Promise<unknown>;
}
declare const _default: {
    name: string;
    version: string;
    hooks: {
        /**
         * Session Start: retrieve relevant memories and inject into System Prompt
         */
        onChatStart: (ctx: ChatContext) => Promise<void>;
        /**
         * Turn Complete: extract and save new memories asynchronously
         */
        onMessageComplete: (ctx: MessageContext) => Promise<void>;
        /**
         * Session End: trigger batch consolidation (decay and deduplication checks)
         */
        onSessionEnd: (ctx: SessionContext) => Promise<void>;
    };
};
export default _default;
