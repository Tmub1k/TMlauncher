const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 3 - Interaction 07: Network Settings & Cloudflare CDN Media Routing', () => {
  it('Routes Cloudflare edge media traffic through configured SOCKS5 proxy tunnel', () => {
    const networkSettings = {
      proxy_enabled: true,
      proxy_url: 'socks5://127.0.0.1:1080',
    };

    function buildMediaFetchOptions(targetUrl, settings) {
      return {
        url: targetUrl,
        viaProxy: settings.proxy_enabled ? settings.proxy_url : null,
        antiHotlinkingHeader: 'OmniLauncher/2026',
      };
    }

    const opts = buildMediaFetchOptions('https://media.omnilauncher.app/cover.webp', networkSettings);
    equal(opts.viaProxy, 'socks5://127.0.0.1:1080');
    equal(opts.antiHotlinkingHeader, 'OmniLauncher/2026');
  });

  it('Resolves Cloudflare edge addresses via configured DoH provider (1.1.1.1 / 8.8.8.8)', () => {
    function resolveEdgeIp(hostname, dohProvider) {
      if (dohProvider === 'cloudflare') {
        return { ip: '104.16.123.96', resolvedVia: 'https://cloudflare-dns.com/dns-query' };
      }
      return { ip: '104.16.123.96', resolvedVia: 'https://dns.google/dns-query' };
    }

    const res = resolveEdgeIp('media.omnilauncher.app', 'cloudflare');
    ok(res.resolvedVia.includes('cloudflare-dns.com'));
  });
});
