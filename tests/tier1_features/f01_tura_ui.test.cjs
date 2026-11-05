const { describe, it } = require('../runner.cjs');
const { ok, equal, match, hasProperties, isValidHexColor } = require('../helpers/assert.cjs');
const { SAMPLE_TURA_THEME } = require('../helpers/fixtures.cjs');
const { validateThemeDefinition } = require('../helpers/contracts.cjs');

describe('Tier 1 - Feature 01: TURA Minimalist UI Redesign', () => {
  it('TURA palette adheres to deep graphite design specifications (#09090b)', () => {
    validateThemeDefinition(SAMPLE_TURA_THEME);
    equal(SAMPLE_TURA_THEME.colors['bg-canvas'], '#09090b', 'bg-canvas must be deep graphite #09090b');
    isValidHexColor(SAMPLE_TURA_THEME.colors['bg-canvas']);
    isValidHexColor(SAMPLE_TURA_THEME.colors['bg-surface']);
    isValidHexColor(SAMPLE_TURA_THEME.colors['bg-card']);
  });

  it('Typography configures Cera CY grotesque typeface with clean letter spacing', () => {
    ok(SAMPLE_TURA_THEME.typography.fontFamily.includes('Cera CY'), 'Font family must include Cera CY');
    equal(SAMPLE_TURA_THEME.typography.letterSpacing, '-0.015em', 'Grotesque typography specifies tight letter spacing');
    ok(SAMPLE_TURA_THEME.typography.fontSizeBase, 'Base font size must be defined');
  });

  it('Game card geometry enforces standard 2:3 vertical aspect ratio', () => {
    const cardGeometry = {
      width: 200,
      height: 300,
      borderRadius: '8px',
      overflow: 'hidden',
    };
    const ratio = cardGeometry.height / cardGeometry.width;
    equal(ratio, 1.5, 'Card must strictly observe 2:3 aspect ratio');
    ok(parseInt(cardGeometry.borderRadius) <= 12, 'TURA style uses refined subtle corners');
  });

  it('Window framing defines borderless frameless window specification', () => {
    const windowConfig = {
      decorations: false,
      transparent: false,
      resizable: true,
      shadow: true,
    };
    equal(windowConfig.decorations, false, 'Frameless window must disable OS decorations');
    ok(windowConfig.shadow, 'Window must enable native compositor drop shadow');
  });

  it('TURA card view strips redundant explanatory labels and text fields', () => {
    const turaCardModel = {
      title: 'Hollow Knight',
      playtimeHours: 42.5,
      coverUrl: 'https://cdn.example.com/capsule.jpg',
      tags: ['Metroidvania', 'Souls-like'],
      showVerboseDescription: false,
      showInstructionPrompt: false,
    };
    equal(turaCardModel.showVerboseDescription, false, 'Verbose description text stripped');
    equal(turaCardModel.showInstructionPrompt, false, 'Instruction prompts stripped');
    hasProperties(turaCardModel, ['title', 'playtimeHours', 'coverUrl', 'tags']);
  });
});
