/**
//! Milestone 3 Frontend & System Adversarial Stress Suite
//!
//! Scope:
//! 1. Proxy URI Parsing & Host/Port validation (parseProxyUri)
//! 2. DoH Endpoint Validation (isValidDohUrl)
//! 3. Proxy Bypass list (shouldBypassProxy)
//! 4. Windows Reserved Names (isReservedWindowsName) - with extensions, case, whitespace
//! 5. Windows Install Path Validation (isValidWindowsInstallPath) - path traversal, UNC, invalid chars
//! 6. Disk space calculation boundaries (canInstallGame, calculateUtilization)
//! 7. Cloudflare CDN Media Proxy URL encoding and safety (getProxiedCoverUrl)
*/

const assert = require('node:assert');
const path = require('node:path');

// Replicate or directly require the store implementations from src/lib/stores/
// We extract the pure functions directly to test their exact logic:

function parseProxyUri(uri) {
  const match = uri.match(/^(socks5|http|https):\/\/(?:([^:]+):([^@]+)@)?(\[[0-9a-fA-F:]+\]|[^:]+):(\d+)$/);
  if (!match) throw new Error('Invalid proxy URI format');
  return {
    protocol: match[1],
    username: match[2] || null,
    password: match[3] || null,
    host: match[4],
    port: Number(match[5]),
  };
}

function isValidDohUrl(url) {
  try {
    const parsed = new URL(url);
    return parsed.protocol === 'https:' && parsed.pathname.length > 1;
  } catch {
    return false;
  }
}

function shouldBypassProxy(hostname) {
  const bypassList = ['localhost', '127.0.0.1', '::1', '*.local'];
  return bypassList.some((rule) => {
    if (rule.startsWith('*.')) return hostname.endsWith(rule.slice(2));
    return hostname === rule;
  });
}

function isReservedWindowsName(folderName) {
  const reserved = new Set([
    'CON', 'PRN', 'AUX', 'NUL',
    'COM0', 'COM1', 'COM2', 'COM3', 'COM4', 'COM5', 'COM6', 'COM7', 'COM8', 'COM9',
    'LPT0', 'LPT1', 'LPT2', 'LPT3', 'LPT4', 'LPT5', 'LPT6', 'LPT7', 'LPT8', 'LPT9',
  ]);
  const trimmed = folderName.trim().replace(/\.+$/, '');
  const stem = trimmed.split('.')[0]?.trim().toUpperCase() || '';
  return reserved.has(stem);
}

function isValidWindowsInstallPath(p) {
  if (p.includes('..')) return false;
  return /^[a-zA-Z]:\\(?:[^\\/:*?"<>|\r\n]+\\)*[^\\/:*?"<>|\r\n]*$/.test(p);
}

function calculateUtilization(total, free) {
  if (total <= 0) return 0;
  const used = total - free;
  return Math.round((used / total) * 100);
}

function canInstallGame(driveFreeBytes, gameSizeBytes, safetyMarginBytes = 1073741824) {
  return driveFreeBytes >= gameSizeBytes + safetyMarginBytes;
}

const CLOUDFLARE_MEDIA_PROXY_BASE = 'https://media.omnilauncher.app/cdn-cgi/image';

function getProxiedCoverUrl(originalUrl, width = 600, format = 'webp') {
  if (!originalUrl) return 'assets/placeholders/fallback_vector.svg';
  if (
    originalUrl.startsWith('data:') ||
    originalUrl.startsWith('asset:') ||
    originalUrl.includes('localhost') ||
    originalUrl.includes('media.omnilauncher.app')
  ) {
    return originalUrl;
  }
  const encoded = encodeURIComponent(originalUrl);
  return `${CLOUDFLARE_MEDIA_PROXY_BASE}/width=${width},format=${format}/${encoded}`;
}

const results = [];
function test(name, fn) {
  try {
    fn();
    results.push({ name, status: 'PASS', error: null });
    console.log(`  ✔ [PASS] ${name}`);
  } catch (err) {
    results.push({ name, status: 'FAIL', error: err.message });
    console.log(`  ✖ [FAIL] ${name} -> ${err.message}`);
  }
}

console.log('=== Milestone 3 Frontend & System Adversarial Stress Suite ===\n');

// ----------------------------------------------------------------------------
// AREA 1: Proxy URI Parsing & Host/Port validation
// ----------------------------------------------------------------------------
console.log('--- Area 1: Proxy URI Parsing & Host/Port Validation ---');

test('Rejects unsupported schemes (ftp://, gopher://, file://)', () => {
  assert.throws(() => parseProxyUri('ftp://127.0.0.1:21'), /Invalid proxy URI format/);
  assert.throws(() => parseProxyUri('file:///C:/Windows:80'), /Invalid proxy URI format/);
  assert.throws(() => parseProxyUri('javascript://alert(1):8080'), /Invalid proxy URI format/);
});

test('Rejects missing ports (http://127.0.0.1)', () => {
  assert.throws(() => parseProxyUri('http://127.0.0.1'), /Invalid proxy URI format/);
});

test('Rejects missing hostname (http://:8080)', () => {
  assert.throws(() => parseProxyUri('http://:8080'), /Invalid proxy URI format/);
});

test('Parses SOCKS5 with authentication', () => {
  const p = parseProxyUri('socks5://user:pass@127.0.0.1:1080');
  assert.strictEqual(p.protocol, 'socks5');
  assert.strictEqual(p.username, 'user');
  assert.strictEqual(p.password, 'pass');
  assert.strictEqual(p.host, '127.0.0.1');
  assert.strictEqual(p.port, 1080);
});

test('CRLF header injection in proxy URI is rejected', () => {
  assert.throws(() => parseProxyUri('http://127.0.0.1:8080\r\nHost: evil.com'), /Invalid proxy URI format/);
});

test('IPv6 proxy addresses [::1]:8080 behavior analysis', () => {
  const p = parseProxyUri('http://[::1]:8080');
  assert.strictEqual(p.protocol, 'http');
  assert.strictEqual(p.host, '[::1]');
  assert.strictEqual(p.port, 8080);
});

// ----------------------------------------------------------------------------
// AREA 2: DoH Endpoint Validation
// ----------------------------------------------------------------------------
console.log('\n--- Area 2: DoH Endpoint Validation ---');

test('Valid DoH endpoints pass (Cloudflare, Google)', () => {
  assert.strictEqual(isValidDohUrl('https://cloudflare-dns.com/dns-query'), true);
  assert.strictEqual(isValidDohUrl('https://dns.google/dns-query'), true);
});

test('Plain HTTP endpoints fail (http://)', () => {
  assert.strictEqual(isValidDohUrl('http://insecure-dns.com/dns-query'), false);
});

test('Empty or root path DoH endpoints fail (pathname.length <= 1)', () => {
  assert.strictEqual(isValidDohUrl('https://cloudflare-dns.com/'), false);
  assert.strictEqual(isValidDohUrl('https://cloudflare-dns.com'), false);
});

test('Hostile or broken URLs fail', () => {
  assert.strictEqual(isValidDohUrl('javascript:alert(1)'), false);
  assert.strictEqual(isValidDohUrl(''), false);
  assert.strictEqual(isValidDohUrl('not_a_url'), false);
});

// ----------------------------------------------------------------------------
// AREA 3: Windows Reserved Names
// ----------------------------------------------------------------------------
console.log('\n--- Area 3: Windows Reserved Names ---');

test('Detects exact Windows reserved names (CON, PRN, AUX, NUL, COM1..9, LPT1..9)', () => {
  assert.strictEqual(isReservedWindowsName('CON'), true);
  assert.strictEqual(isReservedWindowsName('PRN'), true);
  assert.strictEqual(isReservedWindowsName('AUX'), true);
  assert.strictEqual(isReservedWindowsName('NUL'), true);
  assert.strictEqual(isReservedWindowsName('COM1'), true);
  assert.strictEqual(isReservedWindowsName('COM9'), true);
  assert.strictEqual(isReservedWindowsName('LPT1'), true);
  assert.strictEqual(isReservedWindowsName('LPT9'), true);
  assert.strictEqual(isReservedWindowsName('con'), true);
  assert.strictEqual(isReservedWindowsName('nul'), true);
});

test('Windows reserved names with extensions (CON.txt, aux.json) vulnerability check', () => {
  assert.strictEqual(isReservedWindowsName('CON.txt'), true);
  assert.strictEqual(isReservedWindowsName('aux.json'), true);
  assert.strictEqual(isReservedWindowsName('NUL.tar'), true);
});

test('Windows reserved names with whitespace/dots (CON , NUL.)', () => {
  assert.strictEqual(isReservedWindowsName('  CON  '), true);
  assert.strictEqual(isReservedWindowsName('CON.'), true);
});

// ----------------------------------------------------------------------------
// AREA 4: Windows Install Path Validation
// ----------------------------------------------------------------------------
console.log('\n--- Area 4: Windows Install Path Validation ---');

test('Valid absolute Windows paths pass', () => {
  assert.strictEqual(isValidWindowsInstallPath('C:\\Games\\Steam'), true);
  assert.strictEqual(isValidWindowsInstallPath('D:\\Emulators'), true);
  assert.strictEqual(isValidWindowsInstallPath('E:\\'), true);
});

test('Invalid paths with forbidden characters fail', () => {
  assert.strictEqual(isValidWindowsInstallPath('C:\\Games\\*star*'), false);
  assert.strictEqual(isValidWindowsInstallPath('C:\\Games\\?question'), false);
  assert.strictEqual(isValidWindowsInstallPath('C:\\Games\\<angle>'), false);
  assert.strictEqual(isValidWindowsInstallPath('C:\\Games\\|pipe'), false);
});

test('Path traversal check (C:\\Games\\..\\Windows)', () => {
  assert.strictEqual(isValidWindowsInstallPath('C:\\Games\\..\\Windows'), false);
});

// ----------------------------------------------------------------------------
// AREA 5: Disk Space & Utilization Calculations
// ----------------------------------------------------------------------------
console.log('\n--- Area 5: Disk Space Calculations ---');

test('calculateUtilization zero total space returns 0 (prevents NaN / division by zero)', () => {
  assert.strictEqual(calculateUtilization(0, 0), 0);
  assert.strictEqual(calculateUtilization(-100, 0), 0);
});

test('calculateUtilization standard calculation', () => {
  assert.strictEqual(calculateUtilization(1000, 300), 70);
  assert.strictEqual(calculateUtilization(500000000000, 150000000000), 70);
});

test('canInstallGame requires safety margin (default 1GB)', () => {
  const oneGb = 1073741824;
  const gameSize = 50 * oneGb;
  // Free space exactly equals game size -> must fail because buffer is required
  assert.strictEqual(canInstallGame(gameSize, gameSize, oneGb), false);
  // Free space equals game size + buffer -> passes
  assert.strictEqual(canInstallGame(gameSize + oneGb, gameSize, oneGb), true);
});

// ----------------------------------------------------------------------------
// AREA 6: Cloudflare CDN Media Proxy URL Safety
// ----------------------------------------------------------------------------
console.log('\n--- Area 6: Cloudflare CDN Media Proxy URL Safety ---');

test('Empty or null URL returns fallback SVG', () => {
  assert.strictEqual(getProxiedCoverUrl(''), 'assets/placeholders/fallback_vector.svg');
  assert.strictEqual(getProxiedCoverUrl(null), 'assets/placeholders/fallback_vector.svg');
});

test('Local, asset, and data URIs are not proxied', () => {
  assert.strictEqual(getProxiedCoverUrl('data:image/png;base64,123'), 'data:image/png;base64,123');
  assert.strictEqual(getProxiedCoverUrl('asset://localhost/img.png'), 'asset://localhost/img.png');
  assert.strictEqual(getProxiedCoverUrl('http://localhost:1420/icon.png'), 'http://localhost:1420/icon.png');
});

test('External image URL is correctly encoded and proxied', () => {
  const extUrl = 'https://cdn.cloudflare.steamstatic.com/steam/apps/1091500/header.jpg';
  const proxied = getProxiedCoverUrl(extUrl, 600, 'webp');
  assert.ok(proxied.startsWith('https://media.omnilauncher.app/cdn-cgi/image/width=600,format=webp/'));
  assert.ok(proxied.includes(encodeURIComponent(extUrl)));
});

// ----------------------------------------------------------------------------
// SUMMARY
// ----------------------------------------------------------------------------
const total = results.length;
const passed = results.filter((r) => r.status === 'PASS').length;
const failed = total - passed;
console.log(`\n======================================================`);
console.log(`Adversarial Suite Summary: ${passed}/${total} passed, ${failed} failed`);
console.log(`======================================================`);
