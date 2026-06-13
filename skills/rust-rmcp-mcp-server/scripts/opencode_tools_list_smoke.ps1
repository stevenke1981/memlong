param(
    [Parameter(Mandatory=$true)]
    [string]$Binary,

    [string]$SdkVersion = "1.29.0",

    [int]$TimeoutMs = 120000
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $Binary)) {
    throw "Binary not found: $Binary"
}

$work = Join-Path $env:TEMP "opencode-mcp-sdk-smoke-$SdkVersion"
New-Item -ItemType Directory -Force -Path $work | Out-Null

$pkg = Join-Path $work "package.json"
if (-not (Test-Path -LiteralPath $pkg)) {
    Set-Content -LiteralPath $pkg -Encoding UTF8 -Value '{"type":"module"}'
}

$sdkDir = Join-Path $work "node_modules\@modelcontextprotocol\sdk"
if (-not (Test-Path -LiteralPath $sdkDir)) {
    npm --prefix $work install "@modelcontextprotocol/sdk@$SdkVersion" | Out-Host
}

$script = Join-Path $work "tools-list-smoke.mjs"
$js = @"
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";

const binary = process.env.MCP_BINARY;
const timeout = Number(process.env.MCP_TIMEOUT_MS || "120000");
const transport = new StdioClientTransport({ command: binary, args: [], stderr: "pipe" });
const client = new Client({ name: "opencode-tools-list-smoke", version: "1" });

try {
  await client.connect(transport);
  const capabilities = client.getServerCapabilities();
  const result = await client.listTools(undefined, { timeout });
  console.log(JSON.stringify({
    ok: true,
    serverCapabilities: capabilities,
    toolCount: result.tools.length,
    firstTool: result.tools[0]?.name ?? null,
  }, null, 2));
} catch (error) {
  console.error("OpenCode MCP SDK tools/list failed");
  console.error("name:", error?.name);
  console.error("message:", error?.message);
  if (error?.issues) console.error("issues:", JSON.stringify(error.issues, null, 2));
  process.exitCode = 1;
} finally {
  await client.close().catch(() => {});
}
"@
Set-Content -LiteralPath $script -Encoding UTF8 -Value $js

$env:MCP_BINARY = (Resolve-Path -LiteralPath $Binary).Path
$env:MCP_TIMEOUT_MS = [string]$TimeoutMs
node $script
if ($LASTEXITCODE -ne 0) {
    throw "OpenCode MCP SDK tools/list smoke failed"
}
