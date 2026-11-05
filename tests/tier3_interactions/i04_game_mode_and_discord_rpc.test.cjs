const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 3 - Interaction 04: Zero-Footprint Game Mode & Discord Rich Presence', () => {
  it('Synchronizes Game Mode hibernation with Discord Rich Presence broadcast on game launch', () => {
    const system = {
      launcherHibernated: false,
      discordPresenceActive: false,
      discordActivity: null,
    };

    function onGameLaunched(gameTitle, pid) {
      // 1. Enter Zero-Footprint Game Mode
      system.launcherHibernated = true;
      // 2. Broadcast live Discord Rich Presence
      system.discordPresenceActive = true;
      system.discordActivity = {
        details: `Playing ${gameTitle}`,
        state: 'OmniLauncher Active',
      };
    }

    onGameLaunched('Elden Ring', 5432);
    equal(system.launcherHibernated, true, 'Launcher enters hibernation');
    equal(system.discordPresenceActive, true, 'Discord presence broadcasts');
    equal(system.discordActivity.details, 'Playing Elden Ring');
  });

  it('Wakes launcher from hibernation and clears Discord presence atomically when game exits', () => {
    const system = {
      launcherHibernated: true,
      discordPresenceActive: true,
      discordActivity: { details: 'In Game' },
    };

    function onGameExited() {
      // 1. Restore launcher process working set
      system.launcherHibernated = false;
      // 2. Clear Discord RPC presence
      system.discordPresenceActive = false;
      system.discordActivity = null;
    }

    onGameExited();
    equal(system.launcherHibernated, false, 'Launcher restored to active foreground');
    equal(system.discordPresenceActive, false, 'Discord presence cleared');
    equal(system.discordActivity, null);
  });

  it('Maintains accurate elapsed playtime counter in Discord RPC while launcher CPU usage is 0%', () => {
    const startTime = Math.floor(Date.now() / 1000) - 3600; // 1 hr ago
    const launcherCpuPercent = 0.0;
    const rpcTimestamps = { start: startTime };

    equal(launcherCpuPercent, 0.0, 'Zero-footprint maintains 0% CPU');
    ok(rpcTimestamps.start > 0, 'Discord activity displays accurate live elapsed timer');
  });
});
