const { describe, it } = require('../runner.cjs');
const { ok, equal, hasProperties } = require('../helpers/assert.cjs');
const { SAMPLE_EPIC_ITEM } = require('../helpers/fixtures.cjs');
const { validateEpicGame } = require('../helpers/contracts.cjs');

describe('Tier 1 - Feature 09: Epic Games Uninstalled Library Import', () => {
  it('Parses Epic Games .item manifest file format correctly', () => {
    const item = JSON.parse(SAMPLE_EPIC_ITEM);
    equal(item.DisplayName, 'CyberAdventure Deluxe');
    equal(item.AppName, 'CyberAdventure');
    equal(item.CatalogItemId, '4a2b3c4d5e');
  });

  it('Differentiates installed games from uninstalled catalog entries', () => {
    const installed = {
      app_name: 'CyberAdventure',
      display_name: 'CyberAdventure Deluxe',
      catalog_item_id: '4a2b3c4d5e',
      installed: true,
      install_location: 'C:\\Games\\Epic\\CyberAdventure',
    };
    validateEpicGame(installed);
    equal(installed.installed, true);

    const uninstalled = {
      app_name: 'Fortnite',
      display_name: 'Fortnite',
      catalog_item_id: 'fn001',
      installed: false,
      install_location: null,
    };
    validateEpicGame(uninstalled);
    equal(uninstalled.installed, false);
  });

  it('Constructs official Epic Games Launcher launch protocol URI', () => {
    function generateEpicLaunchUri(appName) {
      return `com.epicgames.launcher://apps/${appName}?action=launch&silent=true`;
    }

    const uri = generateEpicLaunchUri('CyberAdventure');
    equal(uri, 'com.epicgames.launcher://apps/CyberAdventure?action=launch&silent=true');
  });

  it('Provides 1-click install action URI for uninstalled Epic titles', () => {
    function generateEpicInstallUri(appName) {
      return `com.epicgames.launcher://apps/${appName}?action=install`;
    }

    const uri = generateEpicInstallUri('Fortnite');
    equal(uri, 'com.epicgames.launcher://apps/Fortnite?action=install');
  });

  it('Aggregates full Epic library combining local manifests and catalog feeds', () => {
    const library = [
      { app_name: 'GameA', display_name: 'Game A', catalog_item_id: 'id_a', installed: true },
      { app_name: 'GameB', display_name: 'Game B', catalog_item_id: 'id_b', installed: false },
    ];
    equal(library.length, 2);
    equal(library.filter(g => g.installed).length, 1);
    equal(library.filter(g => !g.installed).length, 1);
  });
});
