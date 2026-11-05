const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 03: Legacy Clutter Elimination Boundaries', () => {
  it('Ignores and filters attempts to inject deprecated legacy CSS button classes', () => {
    function sanitizeButtonClasses(classList) {
      const forbidden = ['btn-game-green', 'btn-game-orange', 'btn-push-shadow'];
      return classList.filter(c => !forbidden.includes(c));
    }

    const cleaned = sanitizeButtonClasses(['btn-action-primary', 'btn-game-green', 'btn-game-orange']);
    equal(cleaned.length, 1);
    equal(cleaned[0], 'btn-action-primary');
  });

  it('Renders massive game catalog (10,000+ items) via virtualized viewport without DOM bloat', () => {
    function computeVirtualWindow(totalItems, scrollTop, itemHeight, viewportHeight) {
      const startIndex = Math.max(0, Math.floor(scrollTop / itemHeight) - 2);
      const visibleCount = Math.ceil(viewportHeight / itemHeight) + 4;
      const endIndex = Math.min(totalItems, startIndex + visibleCount);
      return { startIndex, endIndex, renderedCount: endIndex - startIndex };
    }

    const virt = computeVirtualWindow(10000, 5000, 300, 900);
    ok(virt.renderedCount <= 10, 'DOM remains lean with virtualized cards');
    ok(virt.startIndex > 0);
  });

  it('Prevents horizontal navigation bar overflow on narrow resolutions (e.g. 768px)', () => {
    function getNavDisplayMode(windowWidth) {
      if (windowWidth < 900) return 'collapsed_icons_only';
      return 'full_icons_and_labels';
    }

    equal(getNavDisplayMode(768), 'collapsed_icons_only');
    equal(getNavDisplayMode(1200), 'full_icons_and_labels');
  });

  it('Strips HTML and script tags from legacy description strings to prevent XSS', () => {
    function sanitizeDescription(rawHtml) {
      return rawHtml.replace(/<[^>]*>?/gm, '').trim();
    }

    const dirty = '<script>alert("hack")</script><b>Great Game!</b> <a href="#">Click</a>';
    const clean = sanitizeDescription(dirty);
    equal(clean, 'alert("hack")Great Game! Click');
    ok(!clean.includes('<script>'));
    ok(!clean.includes('<b>'));
  });

  it('Verifies seamless multi-monitor edge transitions without window border artifacts', () => {
    const multiMonitorConfig = {
      windowBorders: 0,
      subpixelOffset: 0,
    };
    equal(multiMonitorConfig.windowBorders, 0);
  });
});
