"use strict";

const { test } = require("node:test");
const assert = require("node:assert");
const { Impact } = require("../index.js");

test("the Impact surface exposes command and version", () => {
  const impact = new Impact("{}");
  assert.strictEqual(typeof impact.command, "function");
  assert.strictEqual(typeof impact.version, "function");
});
