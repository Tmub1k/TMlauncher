/**
 * Steam Material Noir theme manager & Windows 11 Mica / Acrylic vibrancy bridge.
 *
 * Injects CSS custom properties onto `:root`, supports hot-reloading user theme
 * manifests (`theme.json`), optional live WebM/MP4 background video & OST audio
 * loops, and synchronizes Windows 11 DWM backdrop materials via Tauri IPC.
 */

import { invoke } from '@tauri-apps/api/core';

export type WindowEffectKind = 'mica' | 'tabbed' | 'acrylic' | 'none';

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
  /** Optional loopable background video (`.webm` / `.mp4`). */
  videoUrl?: string;
  /** Optional ambient OST loop (`.ogg` / `.mp3`). */
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

/** Canonical «Steam Material Noir» (True Monochrome Material 3) theme tokens. */
export const STEAM_MATERIAL_NOIR: ThemeManifest = {
  id: 'steam-material-noir',
  name: 'Steam Material Noir',
  author: 'OmniLauncher Community',
  version: '1.0.0',
  variables: {
    '--bg-canvas': '#09090b',
    '--bg-surface-1': '#121215',
    '--bg-surface-2': '#18181b',
    '--bg-surface-3': '#27272a',
    '--border-subtle': 'rgba(255, 255, 255, 0.08)',
    '--border-active': 'rgba(255, 255, 255, 0.25)',
    '--text-primary': '#fafafa',
    '--text-secondary': '#a1a1aa',
    '--text-muted': '#52525b',
    '--accent': '#ffffff',
    '--accent-contrast': '#000000',
    '--elevation-shadow': '0 10px 30px -10px rgba(0, 0, 0, 0.8)',
    '--glow-hover': '0 0 20px rgba(255, 255, 255, 0.12)',
    '--card-radius': '14px',
    '--poster-filter': 'grayscale(85%) contrast(110%)',
    '--poster-filter-hover': 'grayscale(0%) contrast(100%)',
  },
};

const STORAGE_KEY_THEME = 'omnilauncher.theme';
const STORAGE_KEY_EFFECT = 'omnilauncher.vibrancy';

export type ThemeSubscriber = (snapshot: {
  theme: ThemeManifest;
  vibrancy: WindowEffectKind;
}) => void;

class ThemeController {
  private currentTheme: ThemeManifest = STEAM_MATERIAL_NOIR;
  private currentVibrancy: WindowEffectKind = 'mica';
  private subscribers = new Set<ThemeSubscriber>();
  private ostAudio: HTMLAudioElement | null = null;

  /** Apply saved (or default) theme tokens & Windows 11 Mica backdrop on boot. */
  async initialize(): Promise<void> {
    if (typeof window === 'undefined') return;

    try {
      const cachedTheme = localStorage.getItem(STORAGE_KEY_THEME);
      if (cachedTheme) {
        this.currentTheme = JSON.parse(cachedTheme) as ThemeManifest;
      }
      const cachedEffect = localStorage.getItem(STORAGE_KEY_EFFECT) as WindowEffectKind | null;
      if (cachedEffect) {
        this.currentVibrancy = cachedEffect;
      }
    } catch {
      // Ignore storage errors in restricted webviews.
    }

    this.applyTheme(this.currentTheme);
    await this.setVibrancy(this.currentVibrancy);
  }

  /** Inject all `--*` CSS custom properties onto `document.documentElement`. */
  applyTheme(manifest: ThemeManifest): void {
    this.currentTheme = manifest;

    if (typeof document !== 'undefined') {
      const root = document.documentElement;
      root.dataset.theme = manifest.id;

      for (const [key, value] of Object.entries(manifest.variables)) {
        if (key.startsWith('--')) {
          root.style.setProperty(key, value);
        }
      }

      this.syncAmbientAudio(manifest.media);
    }

    try {
      localStorage.setItem(STORAGE_KEY_THEME, JSON.stringify(manifest));
    } catch {
      // Ignore storage quota issues.
    }

    this.notify();
  }

  /** Switch Windows 11 DWM window vibrancy (`mica` | `tabbed` | `acrylic` | `none`). */
  async setVibrancy(effect: WindowEffectKind): Promise<void> {
    this.currentVibrancy = effect;

    if (typeof document !== 'undefined') {
      document.documentElement.dataset.vibrancy = effect;
    }

    try {
      localStorage.setItem(STORAGE_KEY_EFFECT, effect);
      await invoke('set_window_effect', { effect });
    } catch {
      // Running in browser dev mode or on a non-Windows 11 host; CSS fallback handles it.
    }

    this.notify();
  }

  get theme(): ThemeManifest {
    return this.currentTheme;
  }

  get vibrancy(): WindowEffectKind {
    return this.currentVibrancy;
  }

  subscribe(fn: ThemeSubscriber): () => void {
    this.subscribers.add(fn);
    fn({ theme: this.currentTheme, vibrancy: this.currentVibrancy });
    return () => this.subscribers.delete(fn);
  }

  private notify(): void {
    const payload = { theme: this.currentTheme, vibrancy: this.currentVibrancy };
    for (const fn of this.subscribers) {
      fn(payload);
    }
  }

  private syncAmbientAudio(media?: ThemeMedia): void {
    if (this.ostAudio) {
      this.ostAudio.pause();
      this.ostAudio = null;
    }
    if (media?.audioUrl && typeof Audio !== 'undefined') {
      const audio = new Audio(media.audioUrl);
      audio.loop = true;
      audio.volume = Math.min(1, Math.max(0, media.audioVolume ?? 0.25));
      audio.play().catch(() => {
        // Autoplay awaits first user interaction.
      });
      this.ostAudio = audio;
    }
  }
}

export const themeStore = new ThemeController();
