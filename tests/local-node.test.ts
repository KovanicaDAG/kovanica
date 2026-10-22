import { test } from "node:test";
import assert from "node:assert/strict";
import { DEFAULT_NODE_API_URL, localNodeTarget, nodeApiBase } from "../src/lib/api/local-node";

test("nodeApiBase defaults to loopback when unset or blank", () => {
  assert.equal(nodeApiBase(""), DEFAULT_NODE_API_URL);
  assert.equal(nodeApiBase("   "), DEFAULT_NODE_API_URL);
});

test("nodeApiBase normalises and validates", () => {
  assert.equal(nodeApiBase("http://node.internal:8181/"), "http://node.internal:8181");
  assert.equal(nodeApiBase("https://node.example/base/"), "https://node.example/base");
  assert.equal(nodeApiBase("ftp://node.example"), DEFAULT_NODE_API_URL);
  assert.equal(nodeApiBase("not a url"), DEFAULT_NODE_API_URL);
});

test("localNodeTarget keeps path and query", () => {
  assert.equal(
    localNodeTarget("/api/head", "?x=1", "http://node.internal:8181"),
    "http://node.internal:8181/api/head?x=1",
  );
});
