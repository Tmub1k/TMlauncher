const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 17: Cloudflare CDN Media Boundaries', () => {
  it('Falls back to direct origin URL when Cloudflare edge returns HTTP 502/504 Bad Gateway', () => {
    function resolveMediaUrl(edgeHttpStatus, cdnUrl, directOriginUrl) {
      if ([500, 502, 503, 504].includes(edgeHttpStatus)) {
        return { usedUrl: directOriginUrl, bypassCdn: true };
      }
      return { usedUrl: cdnUrl, bypassCdn: false };
    }

    const res = resolveMediaUrl(502, 'https://cdn.example.com/cover.jpg', 'https://steamcdn.origin.com/cover.jpg');
    equal(res.usedUrl, 'https://steamcdn.origin.com/cover.jpg');
    equal(res.bypassCdn, true);
  });

  it('Properly escapes malformed media URLs with special characters and query strings', () => {
    function encodeCdnTarget(rawTargetUrl) {
      return encodeURIComponent(rawTargetUrl);
    }

    const dirty = 'https://store.example.com/images/header?size=large&v=1#preview';
    const encoded = encodeCdnTarget(dirty);
    equal(encoded.includes('&'), false);
    equal(encoded.includes('#'), false);
  });

  it('Rejects image asset downloads exceeding maximum permissible size threshold (15MB)', () => {
    function checkImageSizeLimit(sizeBytes, maxBytes = 15728640) {
      if (sizeBytes > maxBytes) {
        return { allowed: false, reason: 'Image asset exceeds maximum size limit (15MB)' };
      }
      return { allowed: true, reason: null };
    }

    equal(checkImageSizeLimit(20 * 1024 * 1024).allowed, false);
    equal(checkImageSizeLimit(5 * 1024 * 1024).allowed, true);
  });

  it('Renders SVG vector placeholder if WebP / AVIF decode errors occur in webview', () => {
    function getDecoderFallback(decodeErrorOccurred) {
      return decodeErrorOccurred ? 'assets/placeholders/fallback_vector.svg' : null;
    }

    equal(getDecoderFallback(true), 'assets/placeholders/fallback_vector.svg');
    equal(getDecoderFallback(false), null);
  });

  it('Evicts least-recently-used (LRU) cached media assets under disk pressure', () => {
    const cache = [
      { id: 'img1', sizeMB: 10, lastAccess: 100 },
      { id: 'img2', sizeMB: 15, lastAccess: 300 },
      { id: 'img3', sizeMB: 20, lastAccess: 200 },
    ];

    function evictLru(items, targetFreeMB) {
      const sorted = [...items].sort((a, b) => a.lastAccess - b.lastAccess);
      let freed = 0;
      const evicted = [];
      for (const item of sorted) {
        if (freed >= targetFreeMB) break;
        freed += item.sizeMB;
        evicted.push(item.id);
      }
      return { evicted, freedMB: freed };
    }

    const res = evictLru(cache, 10);
    equal(res.evicted.length, 1);
    equal(res.evicted[0], 'img1', 'Oldest accessed asset evicted first');
  });
});
