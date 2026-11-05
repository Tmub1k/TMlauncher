/**
 * Test Context & Ephemeral Sandbox Management for OmniLauncher Tests.
 */
const fs = require('node:fs');
const path = require('node:path');
const os = require('node:os');

class TestContext {
  constructor(name = 'test_sandbox') {
    this.name = name;
    this.sandboxDir = path.join(os.tmpdir(), `omnilauncher_${name}_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`);
    this.cleanups = [];
  }

  setup() {
    if (!fs.existsSync(this.sandboxDir)) {
      fs.mkdirSync(this.sandboxDir, { recursive: true });
    }
    return this.sandboxDir;
  }

  createFile(relativePath, content) {
    const fullPath = path.join(this.sandboxDir, relativePath);
    const dir = path.dirname(fullPath);
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
    fs.writeFileSync(fullPath, content, 'utf8');
    return fullPath;
  }

  readFile(relativePath) {
    const fullPath = path.join(this.sandboxDir, relativePath);
    return fs.readFileSync(fullPath, 'utf8');
  }

  fileExists(relativePath) {
    const fullPath = path.join(this.sandboxDir, relativePath);
    return fs.existsSync(fullPath);
  }

  registerCleanup(fn) {
    this.cleanups.push(fn);
  }

  teardown() {
    for (const fn of this.cleanups.reverse()) {
      try {
        fn();
      } catch (err) {
        // Suppress teardown errors to prevent masking test failures
      }
    }
    if (fs.existsSync(this.sandboxDir)) {
      try {
        fs.rmSync(this.sandboxDir, { recursive: true, force: true });
      } catch (err) {
        // Ignore file lock cleanup errors on Windows
      }
    }
  }
}

function withContext(name, testFn) {
  return async () => {
    const ctx = new TestContext(name);
    ctx.setup();
    try {
      await testFn(ctx);
    } finally {
      ctx.teardown();
    }
  };
}

module.exports = {
  TestContext,
  withContext,
};
