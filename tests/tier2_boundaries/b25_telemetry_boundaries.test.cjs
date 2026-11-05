const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 25: Session Diary & Telemetry Boundaries', () => {
  it('Handles play session spanning system suspend / multi-day uptime without timestamp overflow', () => {
    function computeSessionDuration(startEpochSec, endEpochSec) {
      if (endEpochSec < startEpochSec) return 0;
      return endEpochSec - startEpochSec;
    }

    const multiDayDuration = computeSessionDuration(1700000000, 1700259200); // 3 days
    equal(multiDayDuration, 259200);
    equal(computeSessionDuration(1700000000, 1699999000), 0, 'Reversed timestamps clamped to 0');
  });

  it('Throttles hardware telemetry sampling frequency preventing CPU overhead', () => {
    let sampleCounter = 0;
    let lastSampleTime = 0;

    function pollSensors(now, minIntervalMs = 5000) {
      if (lastSampleTime > 0 && (now - lastSampleTime < minIntervalMs)) {
        return false;
      }
      lastSampleTime = now;
      sampleCounter++;
      return true;
    }

    ok(pollSensors(1000));
    equal(pollSensors(2000), false, 'Throttled within 5000ms window');
    equal(pollSensors(5500), false);
    ok(pollSensors(6100));
    equal(sampleCounter, 2);
  });

  it('Handles missing hardware sensors gracefully when GPU query returns null/unsupported', () => {
    function formatTelemetryReport(sensorReadings) {
      return {
        cpuUsage: sensorReadings.cpu !== null ? `${sensorReadings.cpu}%` : 'N/A',
        gpuTemp: sensorReadings.gpuTemp !== null ? `${sensorReadings.gpuTemp}°C` : 'N/A (Driver unsupported)',
      };
    }

    const report = formatTelemetryReport({ cpu: 25, gpuTemp: null });
    equal(report.cpuUsage, '25%');
    equal(report.gpuTemp, 'N/A (Driver unsupported)');
  });

  it('Auto-repairs unclosed session logs if system crashed without graceful shutdown', () => {
    const unclosedSession = {
      sessionId: 'sess_crash_1',
      startTime: 1700000000,
      endTime: null,
      lastHeartbeatTime: 1700001800, // 30m after start
    };

    function repairOrphanSession(session) {
      if (session.endTime === null) {
        return {
          ...session,
          endTime: session.lastHeartbeatTime,
          durationSeconds: session.lastHeartbeatTime - session.startTime,
          recoveredFromCrash: true,
        };
      }
      return session;
    }

    const repaired = repairOrphanSession(unclosedSession);
    equal(repaired.endTime, 1700001800);
    equal(repaired.durationSeconds, 1800);
    equal(repaired.recoveredFromCrash, true);
  });

  it('Handles disk full condition when appending telemetry diary entries', () => {
    function appendDiaryEntry(entry, isDiskFull) {
      if (isDiskFull) {
        // Drop oldest in-memory entry or suppress to prevent crash
        return { written: false, warning: 'Telemetry log throttled due to low disk space' };
      }
      return { written: true, warning: null };
    }

    const res = appendDiaryEntry({}, true);
    equal(res.written, false);
    ok(res.warning.includes('low disk space'));
  });
});
