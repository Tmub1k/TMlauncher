/**
 * Unified Theme Store & Bridge to Modular Theme Engine.
 * Synchronizes with themeEngine to ensure reactive theme switching and Windows 11 Mica/Acrylic support.
 */

import { themeEngine, type ThemeDefinition, type WindowEffectKind } from '../themes';

export type { WindowEffectKind };

export interface ThemeVariables {
  '--bg-canvas': string;
  '--bg-surface-1': string;
  '--bg-surface-2': string;
  '--bg-surface-3': string;
  '--border-subtle': string;
  '--border-active': string;
  '--text-primary': string;
  '--text-secondary': string;
  '--text-muted': string;
  '--accent': string;
  '--accent-contrast': string;
  '--elevation-shadow': string;
  '--glow-hover': string;
  '--card-radius': string;
  '--poster-filter': string;
  '--poster-filter-hover': string;
  [customProperty: `--${string}`]: string;
}

export interface ThemeMedia {
  videoUrl?: string;
  audioUrl?: string;
  audioVolume?: number;
}

export interface ThemeManifest {
  id: string;
  name: string;
  author: string;
  version: string;
  variables: ThemeVariables;
  media?: ThemeMedia;
}

export type ThemeSubscriber = (snapshot: {
  theme: ThemeManifest | ThemeDefinition;
  vibrancy: WindowEffectKind;
  activeThemeId: string;
  availableThemes: ThemeDefinition[];
}) => void;

class ThemeStoreWrapper {
  get availableThemes(): ThemeDefinition[] {
    return themeEngine.getAvailableThemes();
  }

  get activeThemeId(): string {
    return themeEngine.activeThemeId;
  }

  get activeTheme(): ThemeDefinition {
    return themeEngine.getActiveTheme();
  }

  get theme(): ThemeDefinition {
    return themeEngine.getActiveTheme();
  }

  get vibrancy(): WindowEffectKind {
    return themeEngine.vibrancy;
  }

  get customAccent(): string | null {
    return themeEngine.customAccent;
  }

  async initialize(): Promise<void> {
    await themeEngine.init();
  }

  applyTheme(themeIdOrDef: string | ThemeDefinition): void {
    themeEngine.applyTheme(themeIdOrDef);
  }

  setCustomAccent(color: string | null): void {
    themeEngine.setCustomAccent(color);
  }

  async setVibrancy(effect: WindowEffectKind): Promise<void> {
    await themeEngine.setVibrancy(effect);
  }

  subscribe(fn: ThemeSubscriber): () => void {
    return themeEngine.subscribe((currentTheme) => {
      fn({
        theme: currentTheme,
        vibrancy: themeEngine.vibrancy,
        activeThemeId: currentTheme.id,
        availableThemes: themeEngine.getAvailableThemes(),
      });
    });
  }
}

export const themeStore = new ThemeStoreWrapper();
export { themeEngine };
