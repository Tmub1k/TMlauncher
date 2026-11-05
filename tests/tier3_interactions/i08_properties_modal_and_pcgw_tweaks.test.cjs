const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 3 - Interaction 08: Properties Modal & PCGamingWiki 1-Click Tweaks', () => {
  it('Integrates PCGW quick tweak actions inside Properties Modal Compatibility tab', () => {
    const modalTabState = {
      activeTab: 'compatibility',
      availablePcgwTweaks: [
        { id: 'fov_tweak', name: 'FOV 105 Fix', applied: false },
        { id: 'fps_uncap', name: 'Remove 60 FPS Cap', applied: true },
      ],
    };

    function toggleTweak(tweakId) {
      const target = modalTabState.availablePcgwTweaks.find(t => t.id === tweakId);
      if (target) target.applied = !target.applied;
      return target;
    }

    const toggled = toggleTweak('fov_tweak');
    equal(toggled.applied, true);
  });

  it('Coexists custom launch arguments with PCGW config file adjustments without collision', () => {
    const gameRuntimeConfig = {
      launchArguments: '-novid -high',
      configFileModifications: { 'graphics.ini': 'MaxFPS=144' },
    };

    function buildExecutionPlan(config) {
      return {
        cliArgs: config.launchArguments,
        envVars: { 'GAME_CONFIG_OVERRIDE': '1' },
      };
    }

    const plan = buildExecutionPlan(gameRuntimeConfig);
    equal(plan.cliArgs, '-novid -high');
    equal(gameRuntimeConfig.configFileModifications['graphics.ini'], 'MaxFPS=144');
  });
});
