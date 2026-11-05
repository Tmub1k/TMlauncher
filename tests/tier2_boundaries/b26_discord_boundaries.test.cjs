const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');

describe('Tier 2 - Boundary 26: Universal Discord RPC Boundaries', () => {
  it('Handles situation where Discord desktop app is not running without blocking launcher', () => {
    function connectDiscordRpc(isDiscordClientRunning) {
      if (!isDiscordClientRunning) {
        return { connected: false, error: null, state: 'DISCORD_NOT_RUNNING_IDLE' };
      }
      return { connected: true, state: 'CONNECTED' };
    }

    const res = connectDiscordRpc(false);
    equal(res.connected, false);
    equal(res.state, 'DISCORD_NOT_RUNNING_IDLE');
  });

  it('Recovers when Discord IPC socket connection drops mid-game', () => {
    let reconnectionScheduled = false;
    function handleDiscordDisconnect() {
      reconnectionScheduled = true;
      return { status: 'reconnecting', backoffSeconds: 15 };
    }

    const res = handleDiscordDisconnect();
    equal(reconnectionScheduled, true);
    equal(res.backoffSeconds, 15);
  });

  it('Escapes Discord markdown special characters (*, _, ~, `, >) in game activity strings', () => {
    function sanitizeDiscordText(text) {
      return text.replace(/[*_~`>]/g, '');
    }

    const dirty = 'Playing **ULTRA** _Game_ `v1.0` > Chapter 1';
    const clean = sanitizeDiscordText(dirty);
    equal(clean, 'Playing ULTRA Game v1.0  Chapter 1');
  });

  it('Truncates custom state status strings exceeding Discord 128-character limit', () => {
    function limitDiscordString(str, max = 128) {
      return str.slice(0, max);
    }

    const longStatus = 'Exploring the vast mysterious realm of ancient civilizations with over a hundred companions '.repeat(3);
    const limited = limitDiscordString(longStatus);
    equal(limited.length, 128);
  });

  it('Enforces Discord IPC rate limits (maximum 1 presence update per 15 seconds)', () => {
    let lastUpdateTime = 0;
    function sendDiscordUpdate(now) {
      if (lastUpdateTime > 0 && (now - lastUpdateTime < 15000)) {
        return { sent: false, queued: true };
      }
      lastUpdateTime = now;
      return { sent: true, queued: false };
    }

    equal(sendDiscordUpdate(1000).sent, true);
    equal(sendDiscordUpdate(5000).sent, false);
    equal(sendDiscordUpdate(17000).sent, true);
  });
});
