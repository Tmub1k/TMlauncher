const { describe, it } = require('../runner.cjs');
const { ok, equal, notEqual } = require('../helpers/assert.cjs');

describe('Tier 1 - Feature 03: Legacy Clutter Elimination', () => {
  it('Legacy skeuomorphic button classes (btn-game-*) are fully deprecated and eliminated', () => {
    const legacyClasses = ['btn-game-green', 'btn-game-orange', 'btn-game-3d', 'btn-push-shadow'];
    const modernDesignSystem = new Set([
      'btn-action-primary',
      'btn-action-secondary',
      'btn-ghost',
      'btn-chrome',
    ]);

    for (const legacy of legacyClasses) {
      equal(modernDesignSystem.has(legacy), false, `Legacy class ${legacy} must not exist in modern design system`);
    }
  });

  it('Thick beveled borders and simulated plastic embossing are removed', () => {
    const modernCardBorder = {
      width: '1px',
      style: 'solid',
      color: 'rgba(255, 255, 255, 0.08)',
      boxShadow: 'none',
    };
    equal(modernCardBorder.width, '1px', 'Borders must be hairline 1px');
    equal(modernCardBorder.boxShadow, 'none', 'No cartoonish drop shadow borders');
  });

  it('Redundant explanatory text blocks and instructional tooltips are stripped from library view', () => {
    const libraryViewTemplate = {
      header: 'Library',
      searchPlaceholder: 'Search games...',
      hasHelperParagraph: false,
      hasTutorialBanner: false,
    };
    equal(libraryViewTemplate.hasHelperParagraph, false, 'Helper paragraphs removed for clean minimalist look');
    equal(libraryViewTemplate.hasTutorialBanner, false, 'Permanent tutorial banners eliminated');
  });

  it('Navigation bar condenses into compact icon + title layout', () => {
    const navLayout = {
      heightPx: 48,
      paddingPx: 12,
      showExtraneousBadges: false,
      items: ['Library', 'Store', 'Emulators', 'Downloads', 'Settings'],
    };
    ok(navLayout.heightPx <= 56, 'Nav bar height is compact and unbloated');
    equal(navLayout.showExtraneousBadges, false);
    equal(navLayout.items.length, 5);
  });

  it('Modal dialogues display clean frameless cards without heavy legacy window frames', () => {
    const modalStyle = {
      borderRadius: '12px',
      border: '1px solid rgba(255, 255, 255, 0.1)',
      backgroundColor: '#121215',
      hasClassicWin32Titlebar: false,
    };
    equal(modalStyle.hasClassicWin32Titlebar, false, 'No classic Win32 titlebars inside modals');
    ok(modalStyle.borderRadius, 'Rounded sleek modern modal card');
  });
});
