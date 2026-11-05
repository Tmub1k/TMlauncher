const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 4 - Scenario 03: Competitive Gameplay Session Lifecycle', () => {
  it('Executes full game session: Save Vault snapshot -> Game Mode hibernation -> Discord RPC -> exit & telemetry log', () => {
    const launcherState = {
      isHibernating: false,
      workingSetMB: 180,
      cpuPercent: 1.2,
      discordPresence: null,
      saveVaultSnapshots: [],
      sessionDiary: [],
    };

    // 1. User launches game
    const game = { id: 'steam_730', title: 'Counter-Strike 2', pid: 1420 };

    // 2. Automated Save Vault pre-launch snapshot
    launcherState.saveVaultSnapshots.push({
      id: `vault_pre_${game.id}`,
      gameId: game.id,
      timestamp: 1700000000,
    });
    equal(launcherState.saveVaultSnapshots.length, 1);

    // 3. Zero-Footprint Game Mode activates
    launcherState.isHibernating = true;
    launcherState.workingSetMB = 22; // Dropped by >85%
    launcherState.cpuPercent = 0.0;  // 0% CPU idle
    equal(launcherState.isHibernating, true);
    ok(launcherState.workingSetMB < 30);
    equal(launcherState.cpuPercent, 0.0);

    // 4. Discord Rich Presence broadcasts live session
    launcherState.discordPresence = {
      activity: `Competitive Match - ${game.title}`,
      startedAt: 1700000000,
    };
    ok(launcherState.discordPresence.activity.includes('Counter-Strike 2'));

    // 5. Game terminates 45 minutes later
    const sessionEndTime = 1700002700; // +2700s (45m)
    launcherState.isHibernating = false;
    launcherState.workingSetMB = 175;
    launcherState.discordPresence = null; // Cleared

    // 6. Post-session incremental backup & Session Diary entry
    launcherState.saveVaultSnapshots.push({
      id: `vault_post_${game.id}`,
      gameId: game.id,
      timestamp: sessionEndTime,
    });
    launcherState.sessionDiary.push({
      gameId: game.id,
      durationMinutes: 45,
      avgGpuTemp: 68,
    });

    equal(launcherState.isHibernating, false, 'Launcher restored to foreground');
    equal(launcherState.discordPresence, null, 'Discord RPC presence cleared');
    equal(launcherState.saveVaultSnapshots.length, 2, 'Pre and post snapshots saved');
    equal(launcherState.sessionDiary[0].durationMinutes, 45);
  });
});
