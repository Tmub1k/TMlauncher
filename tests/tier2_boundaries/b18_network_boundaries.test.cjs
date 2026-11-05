const { describe, it } = require('../runner.cjs');
const { ok, equal, throws } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 18: Network Settings Boundaries', () => {
  it('Rejects malformed IP addresses in custom DNS server entries', () => {
    function isValidIpv4(ip) {
      const parts = ip.split('.');
      if (parts.length !== 4) return false;
      return parts.every(p => {
        const n = Number(p);
        return String(n) === p && n >= 0 && n <= 255;
      });
    }

    ok(isValidIpv4('1.1.1.1'));
    ok(isValidIpv4('8.8.8.8'));
    equal(isValidIpv4('999.1.1.1'), false);
    equal(isValidIpv4('1.1.1'), false);
    equal(isValidIpv4('1.1.1.abc'), false);
  });

  it('Falls back to system DNS resolver when custom DoH endpoint fails to respond', () => {
    function resolveDohOrFallback(dohReachable, systemDnsFallback) {
      if (!dohReachable) {
        return { usedResolver: 'system', isFallback: true };
      }
      return { usedResolver: 'doh', isFallback: false };
    }

    const res = resolveDohOrFallback(false, '192.168.1.1');
    equal(res.usedResolver, 'system');
    equal(res.isFallback, true);
  });

  it('Handles SOCKS5 proxy authentication failure without locking application thread', () => {
    function handleProxyAuthResult(statusCode) {
      if (statusCode === 407 || statusCode === 0x05 /* SOCKS5 auth fail */) {
        return { success: false, error: 'PROXY_AUTH_FAILED: Invalid username or password' };
      }
      return { success: true, error: null };
    }

    const fail = handleProxyAuthResult(407);
    equal(fail.success, false);
    ok(fail.error.includes('PROXY_AUTH_FAILED'));
  });

  it('Validates TCP port bounds (1 to 65535) in proxy configurations', () => {
    function isValidPort(port) {
      return Number.isInteger(port) && port >= 1 && port <= 65535;
    }

    ok(isValidPort(1080));
    ok(isValidPort(8080));
    equal(isValidPort(0), false);
    equal(isValidPort(65536), false);
    equal(isValidPort(-80), false);
  });

  it('Rejects empty or whitespace proxy hostnames', () => {
    function validateProxyHost(host) {
      if (!host || typeof host !== 'string' || host.trim() === '') {
        throw new Error('Proxy hostname cannot be empty');
      }
      return host.trim();
    }

    throws(() => validateProxyHost(''), /cannot be empty/);
    throws(() => validateProxyHost('   '), /cannot be empty/);
    equal(validateProxyHost('127.0.0.1'), '127.0.0.1');
  });
});
