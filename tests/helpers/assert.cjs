/**
 * Pure Node.js assertion library for OmniLauncher E2E testing.
 * Compatible with Node.js 26+ strict argument checking.
 */
const assert = require('node:assert');

function ok(value, message) {
  if (message !== undefined) {
    assert.ok(value, message);
  } else {
    assert.ok(value);
  }
}

function equal(actual, expected, message) {
  if (message !== undefined) {
    assert.strictEqual(actual, expected, message);
  } else {
    assert.strictEqual(actual, expected);
  }
}

function notEqual(actual, expected, message) {
  if (message !== undefined) {
    assert.notStrictEqual(actual, expected, message);
  } else {
    assert.notStrictEqual(actual, expected);
  }
}

function deepEqual(actual, expected, message) {
  if (message !== undefined) {
    assert.deepStrictEqual(actual, expected, message);
  } else {
    assert.deepStrictEqual(actual, expected);
  }
}

function match(string, regex, message) {
  if (message !== undefined) {
    assert.match(String(string), regex, message);
  } else {
    assert.match(String(string), regex);
  }
}

function throws(fn, expected, message) {
  if (message !== undefined) {
    assert.throws(fn, expected, message);
  } else {
    assert.throws(fn, expected);
  }
}

function doesNotThrow(fn, message) {
  if (message !== undefined) {
    assert.doesNotThrow(fn, message);
  } else {
    assert.doesNotThrow(fn);
  }
}

async function rejects(asyncFnOrPromise, expected, message) {
  if (message !== undefined) {
    await assert.rejects(asyncFnOrPromise, expected, message);
  } else {
    await assert.rejects(asyncFnOrPromise, expected);
  }
}

async function resolves(asyncFnOrPromise, message) {
  try {
    if (typeof asyncFnOrPromise === 'function') {
      await asyncFnOrPromise();
    } else {
      await asyncFnOrPromise;
    }
  } catch (err) {
    assert.fail(message || `Expected promise to resolve, but rejected with: ${err.message}`);
  }
}

function hasProperties(obj, expectedProps, message) {
  assert.ok(obj && typeof obj === 'object', message || 'Target is not an object');
  for (const prop of expectedProps) {
    assert.ok(prop in obj, message || `Missing expected property: '${prop}'`);
  }
}

function isValidHexColor(color, message) {
  const hexRegex = /^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/;
  assert.match(String(color), hexRegex, message || `Invalid hex color: ${color}`);
}

module.exports = {
  ok,
  equal,
  notEqual,
  deepEqual,
  match,
  throws,
  doesNotThrow,
  rejects,
  resolves,
  hasProperties,
  isValidHexColor,
};
