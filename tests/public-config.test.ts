import { test } from "node:test";
import assert from "node:assert/strict";
import {
  DEFAULT_P2P_PORT,
  DEFAULT_POOL_STRATUM_URL,
  POOL_STRATUM_URL,
  DEFAULT_SEED_PEERS,
  P2P_PORT,
  PRIMARY_SEED,
  SEED_PEERS,
  SEED_PEERS_CSV,
  parsePeerList,
  parsePort,
} from "../src/lib/public-config";

const IPV4 = /\b\d{1,3}(?:\.\d{1,3}){3}\b/;

test("parsePeerList splits on commas/whitespace and drops empties", () => {
  assert.deepEqual(parsePeerList(" a.example:1, b.example:2  c.example:3,,", "x:1"), [
    "a.example:1",
    "b.example:2",
    "c.example:3",
  ]);
});

test("parsePeerList falls back when unset or blank", () => {
  assert.deepEqual(parsePeerList(undefined, "d.example:9"), ["d.example:9"]);
  assert.deepEqual(parsePeerList(" , ", "d.example:9"), ["d.example:9"]);
});

test("parsePort accepts 1..65535 and rejects everything else", () => {
  assert.equal(parsePort("9100", 1), 9100);
  assert.equal(parsePort(undefined, 9000), 9000);
  assert.equal(parsePort("0", 9000), 9000);
  assert.equal(parsePort("70000", 9000), 9000);
  assert.equal(parsePort("90a", 9000), 9000);
});

test("defaults are DNS names, never IP literals", () => {
  assert.ok(!IPV4.test(DEFAULT_SEED_PEERS));
  assert.equal(DEFAULT_P2P_PORT, 9000);
});

test("resolved config is consistent", () => {
  assert.ok(SEED_PEERS.length > 0);
  assert.equal(SEED_PEERS_CSV, SEED_PEERS.join(","));
  assert.equal(PRIMARY_SEED, SEED_PEERS[0]);
  assert.ok(P2P_PORT >= 1 && P2P_PORT <= 65535);
});

test("pool stratum endpoint is a stratum URL on a DNS name", () => {
  assert.ok(!IPV4.test(DEFAULT_POOL_STRATUM_URL));
  assert.match(POOL_STRATUM_URL, /^stratum\+tcp:\/\/[^\s/]+:\d+$/);
});
