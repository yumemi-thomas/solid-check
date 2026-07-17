"use strict";

const assert = require("node:assert/strict");
const test = require("node:test");
const migration = require("../reactivity-v2-migration.json");

test("every legacy semantic Reactivity v2 rule maps to canonical engine findings", () => {
  const semantic = migration.rules.filter(rule => rule.tier === "semantic");
  assert.deepEqual(semantic.map(rule => rule.legacyRule).sort(), [
    "components-return-once", "no-destructure", "no-leaf-owner-operations", "no-owned-scope-writes",
    "no-reactive-read-after-await", "no-stale-props-alias",
    "no-untracked-read-in-effect-apply"
  ]);
  for (const rule of semantic) {
    assert.ok(rule.canonicalRules.length > 0, rule.legacyRule);
    assert.ok(rule.fixtureFiles.length > 0, rule.legacyRule);
  }
});

test("the migrated source corpus records positive and sound-negative expectations", () => {
  assert.equal(migration.sourceFixtures.length, 35);
  assert.equal(migration.sourceFixtures.filter(fixture => fixture.canonicalRule).length, 18);
  assert.equal(migration.sourceFixtures.filter(fixture => fixture.absentRule).length, 17);
});

test("non-catalog type-aware rules have an explicit non-duplicating migration path", () => {
  for (const rule of migration.rules.filter(rule => rule.tier !== "semantic")) {
    assert.equal(rule.migration, "retain-eslint-plugin-solid-2");
    assert.deepEqual(rule.canonicalRules, []);
  }
});
