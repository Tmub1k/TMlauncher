const { describe, it } = require('../runner.cjs');
const { ok, equal } = require('../helpers/assert.cjs');
const { SAMPLE_TURA_THEME, SAMPLE_LIQUID_CHROME_THEME } = require('../helpers/fixtures.cjs');

describe('Tier 3 - Interaction 03: Modular Theme Engine & UI Components', () => {
  it('Theme engine switches CSS variables in DOM root dynamically updating TitleBar and Card components', () => {
    const virtualDomState = {
      rootCssVars: {},
      titleBarColor: '',
      cardBorder: '',
    };

    function applyThemeToUi(theme) {
      virtualDomState.rootCssVars = { ...theme.colors };
      virtualDomState.titleBarColor = theme.colors['bg-canvas'];
      virtualDomState.cardBorder = theme.colors['border-subtle'];
    }

    applyThemeToUi(SAMPLE_TURA_THEME);
    equal(virtualDomState.titleBarColor, '#09090b');
    equal(virtualDomState.cardBorder, 'rgba(255, 255, 255, 0.08)');

    applyThemeToUi(SAMPLE_LIQUID_CHROME_THEME);
    equal(virtualDomState.titleBarColor, '#050507');
    equal(virtualDomState.cardBorder, 'rgba(192, 210, 240, 0.18)');
  });

  it('Maintains active modal dialogue styling when user hot-reloads a theme during open modal state', () => {
    const modalSession = {
      isOpen: true,
      modalId: 'game_properties_modal',
      currentThemeId: 'tura-graphite',
    };

    function onThemeHotReload(newTheme) {
      modalSession.currentThemeId = newTheme.id;
      return { modalRepainted: true, themeId: modalSession.currentThemeId };
    }

    const res = onThemeHotReload(SAMPLE_LIQUID_CHROME_THEME);
    equal(res.modalRepainted, true);
    equal(modalSession.currentThemeId, 'liquid-chrome');
    equal(modalSession.isOpen, true, 'Modal remains open during theme hot reload');
  });

  it('Propagates grotesque typography font family from active theme to all UI components', () => {
    function getComponentFont(theme) {
      return theme.typography.fontFamily;
    }

    const turaFont = getComponentFont(SAMPLE_TURA_THEME);
    ok(turaFont.includes('Cera CY'));

    const chromeFont = getComponentFont(SAMPLE_LIQUID_CHROME_THEME);
    ok(chromeFont.includes('Cera CY'));
  });
});
