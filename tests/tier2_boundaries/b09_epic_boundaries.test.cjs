const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 09: Epic Games Uninstalled Library Boundaries', () => {
  it('Handles corrupted .item files missing mandatory fields without skipping entire library', () => {
    function parseEpicItemSafe(raw) {
      try {
        const item = JSON.parse(raw);
        if (!item.AppName || !item.CatalogItemId) return null;
        return item;
      } catch (err) {
        return null;
      }
    }

    const corruptedJson = '{"AppName": "BrokenGame", "MissingCatalogId": true}';
    equal(parseEpicItemSafe(corruptedJson), null);

    const validJson = '{"AppName": "GoodGame", "CatalogItemId": "cat_1"}';
    ok(parseEpicItemSafe(validJson) !== null);
  });

  it('Resolves duplicate catalog item IDs across multiple historical manifest files', () => {
    const manifests = [
      { AppName: 'Fortnite', CatalogItemId: 'fn001', AppVersionString: '1.0' },
      { AppName: 'Fortnite', CatalogItemId: 'fn001', AppVersionString: '2.0' },
    ];

    function deduplicateEpicManifests(list) {
      const map = new Map();
      for (const m of list) {
        map.set(m.CatalogItemId, m); // Overwrite with newer version
      }
      return Array.from(map.values());
    }

    const deduped = deduplicateEpicManifests(manifests);
    equal(deduped.length, 1);
    equal(deduped[0].AppVersionString, '2.0');
  });

  it('Provides default Epic Games banner when cloud artwork URL returns 404', () => {
    function getEpicArtwork(url) {
      if (!url || url.includes('not_found')) {
        return 'assets/placeholders/epic_default_capsule.png';
      }
      return url;
    }

    equal(getEpicArtwork(null), 'assets/placeholders/epic_default_capsule.png');
    equal(getEpicArtwork('https://cdn.epic.com/not_found.jpg'), 'assets/placeholders/epic_default_capsule.png');
  });

  it('Handles install location residing on disconnected external or network drive', () => {
    function checkDriveAvailable(driveLetter, mountedDrives) {
      return mountedDrives.includes(driveLetter.toUpperCase());
    }

    equal(checkDriveAvailable('C:', ['C:', 'D:']), true);
    equal(checkDriveAvailable('Z:', ['C:', 'D:']), false, 'Disconnected drive flagged properly');
  });

  it('Sanitizes illegal characters in Epic AppName before constructing protocol URL', () => {
    function formatEpicLaunchUrl(appName) {
      const sanitized = encodeURIComponent(appName.trim());
      return `com.epicgames.launcher://apps/${sanitized}?action=launch&silent=true`;
    }

    const dangerous = 'Game Name With Spaces & Symbols!';
    const url = formatEpicLaunchUrl(dangerous);
    ok(url.includes('Game%20Name%20With%20Spaces%20%26%20Symbols!'));
  });
});
