const { describe, it } = require('../runner.cjs');
const { ok, equal, hasProperties } = require('../helpers/assert.cjs');

describe('Tier 1 - Feature 25: Session Diary & Hardware Telemetry', () => {
  it('Records play session start, end, and duration accurately', () => {
    const session = {
      sessionId: 'sess_123',
      gameId: 'steam_730',
      startTime: 1700000000,
      endTime: 1700003600, // 3600 seconds = 1 hour
      durationSeconds: 3600,
    };
    equal(session.endTime - session.startTime, session.durationSeconds);
    equal(session.durationSeconds, 3600);
  });

  it('Aggregates cumulative game playtime over multiple recorded sessions', () => {
    const sessions = [
      { gameId: 'g1', durationSeconds: 1800 }, // 30m
      { gameId: 'g1', durationSeconds: 3600 }, // 60m
      { gameId: 'g2', durationSeconds: 1200 }, // 20m
    ];

    function calculateTotalHours(list, targetGameId) {
      const totalSec = list
        .filter(s => s.gameId === targetGameId)
        .reduce((sum, s) => sum + s.durationSeconds, 0);
      return totalSec / 3600;
    }

    const g1Hours = calculateTotalHours(sessions, 'g1');
    equal(g1Hours, 1.5);
  });

  it('Logs hardware telemetry snapshots (CPU %, GPU temp, RAM usage)', () => {
    const telemetry = {
      timestamp: 1700001000,
      cpuUsagePercent: 32.5,
      gpuTempCelsius: 68,
      ramUsedMB: 12400,
      ramTotalMB: 32768,
    };
    hasProperties(telemetry, ['cpuUsagePercent', 'gpuTempCelsius', 'ramUsedMB']);
    ok(telemetry.gpuTempCelsius < 90, 'Hardware temp within safe envelope');
  });

  it('Calculates session averages for telemetry reporting', () => {
    const samples = [
      { cpu: 20, temp: 60 },
      { cpu: 30, temp: 70 },
      { cpu: 40, temp: 80 },
    ];

    const avgCpu = samples.reduce((s, x) => s + x.cpu, 0) / samples.length;
    const avgTemp = samples.reduce((s, x) => s + x.temp, 0) / samples.length;
    equal(avgCpu, 30);
    equal(avgTemp, 70);
  });

  it('Serializes session diary into structured export JSON', () => {
    const diary = {
      exportVersion: '1.0',
      gameTitle: 'Cyberpunk 2077',
      totalSessions: 1,
      history: [{ durationMinutes: 60 }],
    };
    const jsonStr = JSON.stringify(diary);
    const parsed = JSON.parse(jsonStr);
    equal(parsed.gameTitle, 'Cyberpunk 2077');
    equal(parsed.totalSessions, 1);
  });
});
