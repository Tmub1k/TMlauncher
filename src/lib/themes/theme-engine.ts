/**
 * Modular Theme Engine — OmniLauncher Flagship
 * Runtime theme switching, CSS custom properties injection, hot-reloading drop-in themes.
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  ThemeDefinition,
  ThemeTokens,
  ThemeMedia,
  ThemeSummary,
  WindowEffectKind,
  ThemeChangeListener,
} from './types';

import turaGraphiteJson from './tura-graphite.json';
import liquidChromeJson from './liquid-chrome.json';
import obsidianPureJson from './obsidian-pure.json';
import cyberAuroraJson from './cyber-aurora.json';
import glassJson from './glass.json';
import whiteGlassJson from './white-glass.json';
import greenJson from './green.json';
import yellowJson from './yellow.json';

export const BUILTIN_THEMES: Record<string, ThemeDefinition> = {
  'tura-graphite': turaGraphiteJson as unknown as ThemeDefinition,
  'liquid-chrome': liquidChromeJson as unknown as ThemeDefinition,
  'obsidian-pure': obsidianPureJson as unknown as ThemeDefinition,
  'cyber-aurora': cyberAuroraJson as unknown as ThemeDefinition,
  'glass': glassJson as unknown as ThemeDefinition,
  'white-glass': whiteGlassJson as unknown as ThemeDefinition,
  'green': greenJson as unknown as ThemeDefinition,
  'yellow': yellowJson as unknown as ThemeDefinition,
};

export const DEFAULT_THEME_ID = 'tura-graphite';
export const DEFAULT_THEME: ThemeDefinition = BUILTIN_THEMES[DEFAULT_THEME_ID];

function deepMerge<T extends Record<string, any>>(base: T, override?: Record<string, any>): T {
  const result: any = { ...base };
  if (!override || typeof override !== 'object') return result;
  for (const [key, val] of Object.entries(override)) {
    if (val !== undefined && val !== null) {
      if (
        typeof val === 'object' &&
        !Array.isArray(val) &&
        typeof result[key] === 'object' &&
        !Array.isArray(result[key])
      ) {
        result[key] = deepMerge(result[key], val);
      } else {
        result[key] = val;
      }
    }
  }
  return result;
}

const STORAGE_KEY_THEME_ID = 'omnilauncher.theme_id';
const STORAGE_KEY_VIBRANCY = 'omnilauncher.vibrancy';

export class ThemeEngine {
  private activeTheme: ThemeDefinition = DEFAULT_THEME;
  private availableThemes = new Map<string, ThemeDefinition>();
  private subscribers = new Set<ThemeChangeListener>();
  private ambientAudio: HTMLAudioElement | null = null;
  private initialized = false;
  private injectedCustomProperties = new Set<string>();

  constructor() {
    // Register built-ins by default
    for (const [id, theme] of Object.entries(BUILTIN_THEMES)) {
      this.availableThemes.set(id, theme);
    }
  }

  /**
   * Initializes theme engine, loads saved preference, queries custom themes from disk,
   * and applies CSS custom properties.
   */
  async init(): Promise<void> {
    if (this.initialized) return;
    this.initialized = true;

    // Load custom themes from Tauri backend if available
    await this.scanCustomThemes();

    // Determine initial theme ID from storage or fallback to TURA Graphite
    let initialThemeId = 'tura-graphite';
    if (typeof window !== 'undefined') {
      try {
        const savedId = localStorage.getItem(STORAGE_KEY_THEME_ID);
        if (savedId && this.availableThemes.has(savedId)) {
          initialThemeId = savedId;
        }
      } catch {
        // Ignore localStorage restrictions
      }
    }

    const themeToApply = this.availableThemes.get(initialThemeId) || BUILTIN_THEMES['tura-graphite'];
    this.applyTheme(themeToApply);
  }

  /**
   * Scans backend for custom drop-in themes in `themes/` and `%APPDATA%/themes/`.
   */
  async scanCustomThemes(): Promise<void> {
    try {
      const customSummaries = await invoke<ThemeSummary[]>('get_available_themes');
      if (Array.isArray(customSummaries)) {
        for (const summary of customSummaries) {
          if (!this.availableThemes.has(summary.id)) {
            try {
              const fullManifest = await invoke<ThemeDefinition>('load_theme_manifest', {
                themeId: summary.id,
              });
              if (fullManifest && fullManifest.tokens) {
                fullManifest.isCustom = true;
                this.availableThemes.set(summary.id, fullManifest);
              }
            } catch {
              // Non-fatal if specific manifest fails to load
            }
          }
        }
      }
    } catch {
      // In web dev mode or when command is not yet ready, built-in themes are used
    }
  }

  /**
   * Applies the selected theme definition or theme ID at runtime.
   * Injects all CSS custom properties onto `document.documentElement`.
   */
  applyTheme(themeOrId?: string | ThemeDefinition | null): void {
    let resolvedTheme: ThemeDefinition | undefined;

    if (!themeOrId) {
      console.warn(`[ThemeEngine] Received nullish theme, defaulting to ${DEFAULT_THEME_ID}.`);
      resolvedTheme = DEFAULT_THEME;
    } else if (typeof themeOrId === 'string') {
      resolvedTheme = this.availableThemes.get(themeOrId) || BUILTIN_THEMES[themeOrId];
      if (!resolvedTheme) {
        console.warn(`[ThemeEngine] Theme '${themeOrId}' not found. Falling back to ${DEFAULT_THEME_ID}.`);
        resolvedTheme = DEFAULT_THEME;
      }
    } else if (typeof themeOrId === 'object') {
      if (!themeOrId.id) {
        console.warn(`[ThemeEngine] Theme definition missing id, defaulting to ${DEFAULT_THEME_ID}.`);
        resolvedTheme = DEFAULT_THEME;
      } else {
        resolvedTheme = themeOrId;
        this.availableThemes.set(resolvedTheme.id, resolvedTheme);
      }
    } else {
      resolvedTheme = DEFAULT_THEME;
    }

    // Deep-merge target theme tokens on top of DEFAULT_THEME.tokens
    const defaultTokens = (DEFAULT_THEME.tokens || {}) as Record<string, string>;
    const targetTokens = (resolvedTheme.tokens || {}) as Record<string, string>;
    const mergedTokens = deepMerge(defaultTokens, targetTokens);

    resolvedTheme = {
      ...resolvedTheme,
      tokens: mergedTokens,
    };

    this.activeTheme = resolvedTheme;

    if (typeof document !== 'undefined') {
      const root = document.documentElement;
      root.dataset.theme = resolvedTheme.id;

      // Clear previously injected custom CSS property names from root.style
      for (const prop of this.injectedCustomProperties) {
        root.style.removeProperty(prop);
      }
      this.injectedCustomProperties.clear();

      for (const [prop, val] of Object.entries(mergedTokens)) {
        if (typeof val === 'string' && prop.startsWith('--')) {
          root.style.setProperty(prop, val);
          this.injectedCustomProperties.add(prop);
        }
      }

      // Also ensure backward-compatible surface tokens exist
      const tokenMap = mergedTokens as Record<string, string | undefined>;
      if (!tokenMap['--bg-surface-1'] && tokenMap['--bg-surface-base']) {
        root.style.setProperty('--bg-surface-1', tokenMap['--bg-surface-base']!);
        this.injectedCustomProperties.add('--bg-surface-1');
      }
      if (!tokenMap['--bg-surface-2'] && tokenMap['--bg-surface-elevated']) {
        root.style.setProperty('--bg-surface-2', tokenMap['--bg-surface-elevated']!);
        this.injectedCustomProperties.add('--bg-surface-2');
      }
      if (!tokenMap['--bg-surface-3'] && tokenMap['--bg-surface-floating']) {
        root.style.setProperty('--bg-surface-3', tokenMap['--bg-surface-floating']!);
        this.injectedCustomProperties.add('--bg-surface-3');
      }

      this.syncAmbientMedia(resolvedTheme.media);
    }

    if (typeof window !== 'undefined') {
      try {
        localStorage.setItem(STORAGE_KEY_THEME_ID, resolvedTheme.id);
        localStorage.setItem(
          'omnilauncher.theme',
          JSON.stringify({
            id: resolvedTheme.id,
            name: resolvedTheme.name,
            author: resolvedTheme.author,
            version: resolvedTheme.version,
            variables: resolvedTheme.tokens,
            media: resolvedTheme.media,
          })
        );
      } catch {
        // Ignore storage exceptions
      }
    }

    // Apply vibrancy if specified
    if (resolvedTheme.vibrancy) {
      void this.setVibrancy(resolvedTheme.vibrancy);
    }

    this.notify();
  }

  /**
   * Sets Windows 11 DWM backdrop material via Tauri IPC.
   */
  async setVibrancy(effect: WindowEffectKind): Promise<void> {
    if (typeof document !== 'undefined') {
      document.documentElement.dataset.vibrancy = effect;
    }

    try {
      if (typeof window !== 'undefined') {
        localStorage.setItem(STORAGE_KEY_VIBRANCY, effect);
      }
      await invoke('set_window_effect', { effect });
    } catch {
      // Ignored when running outside Tauri or on unsupported OS versions
    }
  }

  /**
   * Returns list of all available themes (built-ins + discovered custom).
   */
  getAvailableThemes(): ThemeDefinition[] {
    return Array.from(this.availableThemes.values());
  }

  /**
   * Returns currently active theme definition.
   */
  getActiveTheme(): ThemeDefinition {
    return this.activeTheme;
  }

  /**
   * Subscribes to active theme changes.
   */
  subscribe(listener: ThemeChangeListener): () => void {
    this.subscribers.add(listener);
    try {
      listener(this.activeTheme);
    } catch (err) {
      console.error('[ThemeEngine] Initial subscriber callback error:', err);
    }
    return () => {
      this.subscribers.delete(listener);
    };
  }

  private notify(): void {
    for (const listener of this.subscribers) {
      try {
        listener(this.activeTheme);
      } catch (err) {
        console.error('[ThemeEngine] Subscriber notification error:', err);
      }
    }
  }

  private syncAmbientMedia(media?: ThemeMedia): void {
    if (this.ambientAudio) {
      this.ambientAudio.pause();
      this.ambientAudio = null;
    }

    if (media?.ambientAudio && typeof Audio !== 'undefined') {
      const audio = new Audio(media.ambientAudio);
      audio.loop = true;
      audio.volume = Math.min(1, Math.max(0, media.audioVolume ?? 0.2));
      audio.play().catch(() => {
        // Awaiting user gesture
      });
      this.ambientAudio = audio;
    }
  }
}

export const themeEngine = new ThemeEngine();
