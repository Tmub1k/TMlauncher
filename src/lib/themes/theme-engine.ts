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
import whiteGlassJson from './white-glass.json';
import cyberAuroraJson from './cyber-aurora.json';
import greenJson from './green.json';
import yellowJson from './yellow.json';

export const BUILTIN_THEMES: Record<string, ThemeDefinition> = {
  'tura-graphite': turaGraphiteJson as unknown as ThemeDefinition,
  'white-glass': whiteGlassJson as unknown as ThemeDefinition,
  'cyber-aurora': cyberAuroraJson as unknown as ThemeDefinition,
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
const STORAGE_KEY_THEME_ALT = 'omnilauncher_theme';
const STORAGE_KEY_ACCENT = 'omnilauncher_accent';
const STORAGE_KEY_VIBRANCY = 'omnilauncher.vibrancy';

export class ThemeEngine {
  private activeTheme: ThemeDefinition = DEFAULT_THEME;
  private availableThemesMap = new Map<string, ThemeDefinition>();
  private subscribers = new Set<ThemeChangeListener>();
  private ambientAudio: HTMLAudioElement | null = null;
  private initialized = false;
  private injectedCustomProperties = new Set<string>();
  private _customAccent: string | null = null;
  private _currentVibrancy: WindowEffectKind = 'mica';

  constructor() {
    for (const [id, theme] of Object.entries(BUILTIN_THEMES)) {
      this.availableThemesMap.set(id, theme);
    }
  }

  get activeThemeId(): string {
    return this.activeTheme.id;
  }

  get availableThemes(): ThemeDefinition[] {
    return Array.from(this.availableThemesMap.values());
  }

  get customAccent(): string | null {
    return this._customAccent;
  }

  get vibrancy(): WindowEffectKind {
    return this._currentVibrancy;
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

    // Determine initial theme ID from storage
    let initialThemeId = DEFAULT_THEME_ID;
    if (typeof window !== 'undefined') {
      try {
        const savedId = localStorage.getItem(STORAGE_KEY_THEME_ALT) || localStorage.getItem(STORAGE_KEY_THEME_ID);
        if (savedId && this.availableThemesMap.has(savedId)) {
          initialThemeId = savedId;
        }
        const savedAccent = localStorage.getItem(STORAGE_KEY_ACCENT);
        if (savedAccent) {
          this._customAccent = savedAccent;
        }
        const savedVibrancy = localStorage.getItem(STORAGE_KEY_VIBRANCY) as WindowEffectKind | null;
        if (savedVibrancy) {
          this._currentVibrancy = savedVibrancy;
        }
      } catch {
        // Ignore localStorage restrictions
      }
    }

    const themeToApply = this.availableThemesMap.get(initialThemeId) || BUILTIN_THEMES[DEFAULT_THEME_ID];
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
          if (!this.availableThemesMap.has(summary.id)) {
            try {
              const fullManifest = await invoke<ThemeDefinition>('load_theme_manifest', {
                themeId: summary.id,
              });
              if (fullManifest && fullManifest.tokens) {
                fullManifest.isCustom = true;
                this.availableThemesMap.set(summary.id, fullManifest);
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
      resolvedTheme = this.availableThemesMap.get(themeOrId) || BUILTIN_THEMES[themeOrId];
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
        this.availableThemesMap.set(resolvedTheme.id, resolvedTheme);
      }
    } else {
      resolvedTheme = DEFAULT_THEME;
    }

    // Deep-merge target theme tokens on top of DEFAULT_THEME.tokens
    const defaultTokens = (DEFAULT_THEME.tokens || {}) as Record<string, string>;
    const targetTokens = (resolvedTheme.tokens || {}) as Record<string, string>;
    const mergedTokens = deepMerge(defaultTokens, targetTokens);

    // Apply custom accent if active
    if (this._customAccent) {
      mergedTokens['--accent-primary'] = this._customAccent;
      mergedTokens['--accent'] = this._customAccent;
      mergedTokens['--text-accent'] = this._customAccent;
      mergedTokens['--border-active'] = this._customAccent;
      mergedTokens['--accent-glow'] = `0 0 24px ${this._customAccent}40`;
    }

    resolvedTheme = {
      ...resolvedTheme,
      tokens: mergedTokens,
    };

    this.activeTheme = resolvedTheme;

    if (typeof document !== 'undefined') {
      const root = document.documentElement;
      root.dataset.theme = resolvedTheme.id;
      root.setAttribute('data-theme', resolvedTheme.id);

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

      // Backward-compatible surface & text tokens
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
        localStorage.setItem(STORAGE_KEY_THEME_ALT, resolvedTheme.id);
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
   * Sets a custom user accent color across the UI.
   */
  setCustomAccent(color: string | null): void {
    this._customAccent = color && color.trim().length > 0 ? color.trim() : null;
    if (typeof window !== 'undefined') {
      try {
        if (this._customAccent) {
          localStorage.setItem(STORAGE_KEY_ACCENT, this._customAccent);
        } else {
          localStorage.removeItem(STORAGE_KEY_ACCENT);
        }
      } catch {}
    }
    this.applyTheme(this.activeTheme);
  }

  /**
   * Sets Windows 11 DWM backdrop material via Tauri IPC.
   */
  async setVibrancy(effect: WindowEffectKind): Promise<void> {
    this._currentVibrancy = effect;
    if (typeof document !== 'undefined') {
      document.documentElement.dataset.vibrancy = effect;
      document.documentElement.setAttribute('data-vibrancy', effect);
    }

    try {
      if (typeof window !== 'undefined') {
        localStorage.setItem(STORAGE_KEY_VIBRANCY, effect);
      }
      await invoke('set_window_effect', { effect });
    } catch {
      // Ignored when running outside Tauri or on unsupported OS versions
    }
    this.notify();
  }

  /**
   * Returns list of all available themes (built-ins + discovered custom).
   */
  getAvailableThemes(): ThemeDefinition[] {
    return Array.from(this.availableThemesMap.values());
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
