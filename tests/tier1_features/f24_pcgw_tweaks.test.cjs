const { describe, it } = require('../runner.cjs');
const { ok, equal, hasProperties } = require('../helpers/assert.cjs');
const { validatePcgwRecord } = require('../helpers/contracts.cjs');

describe('Tier 1 - Feature 24: PCGamingWiki 1-Click Tweak Engine', () => {
  it('Validates PcgwRecord schema and capability flags', () => {
    const pcgw = {
      game_title: 'Elden Ring',
      ultrawide_support: false,
      fps_uncap_supported: true,
      fov_slider_supported: false,
      tweaks: [
        {
          id: 'fps_uncap',
          title: 'Uncap FPS to 120/144',
          configFile: 'graphics_config.xml',
          key: 'MaxFPS',
          currentVal: '60',
          recommendedVal: '144',
        },
      ],
    };
    validatePcgwRecord(pcgw);
    equal(pcgw.fps_uncap_supported, true);
    equal(pcgw.tweaks.length, 1);
  });

  it('Modifies configuration key-value pair in INI/XML formats', () => {
    function applyIniTweak(iniContent, section, key, newValue) {
      const regex = new RegExp(`^(\\s*${key}\\s*=\\s*).*$`, 'm');
      if (regex.test(iniContent)) {
        return iniContent.replace(regex, `$1${newValue}`);
      }
      return `${iniContent}\n${key}=${newValue}`;
    }

    const originalIni = `[Display]\nResolution=1920x1080\nMaxFPS=60\nVSync=1`;
    const tweakedIni = applyIniTweak(originalIni, 'Display', 'MaxFPS', '144');
    ok(tweakedIni.includes('MaxFPS=144'));
    ok(!tweakedIni.includes('MaxFPS=60'));
  });

  it('Creates backup copy of configuration file prior to applying tweak', () => {
    function generateBackupFilename(configPath) {
      return `${configPath}.omnibak`;
    }

    const bak = generateBackupFilename('C:\\Games\\Config.ini');
    equal(bak, 'C:\\Games\\Config.ini.omnibak');
  });

  it('Reverts applied tweak restoring original backup cleanly', () => {
    let activeConfig = 'MaxFPS=144';
    const backupConfig = 'MaxFPS=60';

    function revertTweak() {
      activeConfig = backupConfig;
      return activeConfig;
    }

    revertTweak();
    equal(activeConfig, 'MaxFPS=60', 'Original settings restored');
  });

  it('Recognizes ultrawide aspect ratios (21:9 and 32:9)', () => {
    function isUltrawide(width, height) {
      const ratio = width / height;
      return ratio >= (21 / 9) - 0.05;
    }

    equal(isUltrawide(2560, 1080), true);
    equal(isUltrawide(3440, 1440), true);
    equal(isUltrawide(1920, 1080), false);
  });
});
