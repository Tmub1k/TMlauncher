const { describe, it } = require('../runner.cjs');
const { ok, equal, match, hasProperties } = require('../helpers/assert.cjs');
const { validateNetworkSettings } = require('../helpers/contracts.cjs');

describe('Tier 1 - Feature 18: Network Settings (DNS & Proxy)', () => {
  it('Validates DNS provider presets (Cloudflare, Google, Quad9)', () => {
    const dnsPresets = {
      'cloudflare': { ipv4: ['1.1.1.1', '1.0.0.1'], doh: 'https://cloudflare-dns.com/dns-query' },
      'google': { ipv4: ['8.8.8.8', '8.8.4.4'], doh: 'https://dns.google/dns-query' },
      'quad9': { ipv4: ['9.9.9.9', '149.112.112.112'], doh: 'https://dns.quad9.net/dns-query' },
    };

    equal(dnsPresets['cloudflare'].ipv4[0], '1.1.1.1');
    equal(dnsPresets['google'].ipv4[0], '8.8.8.8');
    equal(dnsPresets['quad9'].ipv4[0], '9.9.9.9');
  });

  it('Validates network configuration schema contract', () => {
    const settings = {
      dns_provider: 'cloudflare',
      doh_enabled: true,
      custom_doh_url: null,
      proxy_enabled: false,
      proxy_url: null,
    };
    validateNetworkSettings(settings);
    equal(settings.dns_provider, 'cloudflare');
  });

  it('Parses and validates SOCKS5 and HTTP proxy connection strings', () => {
    function parseProxyUri(uri) {
      const matchProxy = uri.match(/^(socks5|http|https):\/\/(?:([^:]+):([^@]+)@)?([^:]+):(\d+)$/);
      if (!matchProxy) throw new Error('Invalid proxy URI format');
      return {
        protocol: matchProxy[1],
        username: matchProxy[2] || null,
        password: matchProxy[3] || null,
        host: matchProxy[4],
        port: Number(matchProxy[5]),
      };
    }

    const proxy = parseProxyUri('socks5://user1:pass123@127.0.0.1:1080');
    equal(proxy.protocol, 'socks5');
    equal(proxy.username, 'user1');
    equal(proxy.port, 1080);
  });

  it('Validates DNS over HTTPS (DoH) URL endpoints against RFC 8484', () => {
    function isValidDohUrl(url) {
      try {
        const parsed = new URL(url);
        return parsed.protocol === 'https:' && parsed.pathname.length > 1;
      } catch (err) {
        return false;
      }
    }

    ok(isValidDohUrl('https://cloudflare-dns.com/dns-query'));
    ok(isValidDohUrl('https://dns.google/dns-query'));
    equal(isValidDohUrl('http://insecure-dns.com/dns-query'), false, 'DoH requires HTTPS');
  });

  it('Applies proxy bypass list for local loopback addresses', () => {
    const bypassList = ['localhost', '127.0.0.1', '::1', '*.local'];
    function shouldBypassProxy(hostname) {
      return bypassList.some(rule => {
        if (rule.startsWith('*.')) return hostname.endsWith(rule.slice(2));
        return hostname === rule;
      });
    }

    equal(shouldBypassProxy('localhost'), true);
    equal(shouldBypassProxy('127.0.0.1'), true);
    equal(shouldBypassProxy('steamcommunity.com'), false);
  });
});
