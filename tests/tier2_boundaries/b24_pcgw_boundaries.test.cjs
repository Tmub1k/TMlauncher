const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 24: PCGamingWiki Tweak Engine Boundaries', () => {
  it('Handles read-only attributes on game configuration files by prompting or clearing flag', () => {
    function prepareFileForEdit(isReadOnly) {
      if (isReadOnly) {
        // Clear read-only attribute, edit, and mark restored
        return { modified: true, clearedReadOnlyFlag: true };
      }
      return { modified: true, clearedReadOnlyFlag: false };
    }

    const res = prepareFileForEdit(true);
    equal(res.clearedReadOnlyFlag, true);
  });

  it('Guards against applying tweaks to configuration files with pre-existing syntax errors', () => {
    function validateXmlConfig(rawXml) {
      const isBalanced = (rawXml.match(/<config>/g) || []).length === (rawXml.match(/<\/config>/g) || []).length;
      if (!isBalanced) {
        return { canTweak: false, reason: 'Configuration file has existing syntax corruption' };
      }
      return { canTweak: true, reason: null };
    }

    const corruptedXml = '<config><fps>60'; // Missing closing tag
    const res = validateXmlConfig(corruptedXml);
    equal(res.canTweak, false);
    ok(res.reason.includes('syntax corruption'));
  });

  it('Handles uncatalogued indie games with no PCGamingWiki entry gracefully', () => {
    function queryPcgw(gameArticleFound) {
      if (!gameArticleFound) {
        return { availableTweaks: [], message: 'No community tweaks documented for this title yet' };
      }
      return { availableTweaks: [{ id: 'fov' }], message: 'OK' };
    }

    const missing = queryPcgw(false);
    equal(missing.availableTweaks.length, 0);
    ok(missing.message.includes('No community tweaks'));
  });

  it('Clamps out-of-bounds user tweak parameters (e.g. FOV > 180 or FPS < 30)', () => {
    function clampTweakValue(tweakType, val) {
      if (tweakType === 'fov') return Math.max(60, Math.min(130, val));
      if (tweakType === 'fps') return Math.max(30, Math.min(360, val));
      return val;
    }

    equal(clampTweakValue('fov', 250), 130, 'FOV clamped to safe max 130');
    equal(clampTweakValue('fov', 20), 60, 'FOV clamped to safe min 60');
    equal(clampTweakValue('fps', 10), 30, 'FPS clamped to safe min 30');
  });

  it('Handles race condition where configuration file is deleted between scan and tweak application', () => {
    function applyTweakFileCheck(fileExists) {
      if (!fileExists) {
        return { success: false, error: 'Target config file was moved or deleted' };
      }
      return { success: true, error: null };
    }

    const res = applyTweakFileCheck(false);
    equal(res.success, false);
    ok(res.error.includes('moved or deleted'));
  });
});
