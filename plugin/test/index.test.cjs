const test = require("node:test");
const assert = require("node:assert/strict");
const { parseMemoriesResponse, formatMemoriesForInjection } = require("../dist/index.js");

const memory = {
  id: "1",
  content: "User prefers Rust.",
  category: "Preference",
  importance_score: 0.8,
};

test("parses direct, wrapped, and rmcp text responses", () => {
  assert.deepEqual(parseMemoriesResponse([memory]), [memory]);
  assert.deepEqual(parseMemoriesResponse({ results: [memory] }), [memory]);
  assert.deepEqual(
    parseMemoriesResponse({ content: [{ type: "text", text: JSON.stringify([memory]) }] }),
    [memory],
  );
});

test("formats nested search results without undefined fields", () => {
  const text = formatMemoriesForInjection([{ memory }]);
  assert.match(text, /\[Preference\] User prefers Rust\./);
  assert.doesNotMatch(text, /undefined/);
});
