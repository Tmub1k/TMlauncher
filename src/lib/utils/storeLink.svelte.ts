/**
 * Store Link Manager & Open Mode Handler
 * Modes:
 * - 'client': Opens game in installed Steam/Epic client (user already logged in)
 * - 'launcher': Opens embedded WebView window inside launcher
 * - 'browser': Opens in default OS browser
 */

import { invoke } from '@tauri-apps/api/core';
import { inAppStore } from '../stores/library';

export type StoreOpenMode = 'client' | 'launcher' | 'browser';

const STORAGE_KEY_STORE_MODE = 'omni_store_open_mode';

class StoreLinkController {
  mode = $state<StoreOpenMode>('client');

  constructor() {
    if (typeof localStorage !== 'undefined') {
      const saved = localStorage.getItem(STORAGE_KEY_STORE_MODE) as StoreOpenMode | null;
      if (saved === 'client' || saved === 'launcher' || saved === 'browser') {
        this.mode = saved;
      }
    }
  }

  setMode(newMode: StoreOpenMode) {
    this.mode = newMode;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(STORAGE_KEY_STORE_MODE, newMode);
    }
  }

  extractSteamAppId(url?: string, appId?: number | string | null): string | null {
    if (appId !== undefined && appId !== null && String(appId).trim().length > 0) {
      return String(appId).trim();
    }
    if (!url) return null;
    const match = url.match(/(?:store\.steampowered\.com\/app|steamcommunity\.com\/app)\/(\d+)/i);
    return match ? match[1] : null;
  }

  extractPlatform(url?: string, platform?: string): 'steam' | 'epic' | 'itch' | 'other' {
    if (platform) {
      const p = platform.toLowerCase();
      if (p.includes('steam')) return 'steam';
      if (p.includes('epic')) return 'epic';
      if (p.includes('itch')) return 'itch';
    }
    if (!url) return 'other';
    const lower = url.toLowerCase();
    if (lower.includes('steampowered.com') || lower.includes('steamcommunity.com')) return 'steam';
    if (lower.includes('epicgames.com')) return 'epic';
    if (lower.includes('itch.io')) return 'itch';
    return 'other';
  }

  async open(options: {
    url: string;
    title?: string;
    appId?: number | string | null;
    platform?: string;
    installUri?: string;
    mode?: StoreOpenMode;
  }): Promise<void> {
    const targetMode = options.mode || this.mode;
    const cleanUrl = options.url.trim();
    const plat = this.extractPlatform(cleanUrl, options.platform);
    const steamId = this.extractSteamAppId(cleanUrl, options.appId);

    // 1. Try backend open_store_link command first
    try {
      await invoke('open_store_link', {
        url: cleanUrl,
        mode: targetMode,
        appId: steamId || (options.appId ? String(options.appId) : undefined),
        platform: plat,
      });
      return;
    } catch {
      // Backend command doesn't exist or errored, proceed with frontend fallback logic
    }

    // 2. Client Mode Fallback
    if (targetMode === 'client') {
      if (plat === 'steam' && steamId) {
        const steamUri = `steam://store/${steamId}`;
        try {
          await invoke('open_external_url', { url: steamUri });
          return;
        } catch {
          window.location.href = steamUri;
          return;
        }
      } else if (plat === 'epic') {
        const epicUri = options.appId
          ? `com.epicgames.launcher://apps/${options.appId}?action=browse`
          : 'com.epicgames.launcher://';
        try {
          await invoke('open_external_url', { url: epicUri });
          return;
        } catch {
          window.location.href = epicUri;
          return;
        }
      }
      // Non-Steam/Epic or failed client URI fallback to browser
      try {
        await invoke('open_external_url', { url: cleanUrl });
      } catch {
        window.open(cleanUrl, '_blank');
      }
      return;
    }

    // 3. Launcher Window Fallback
    if (targetMode === 'launcher') {
      try {
        await invoke('open_in_app_store_window', {
          url: cleanUrl,
          title: options.title || 'Store',
        });
        return;
      } catch {
        // Fallback to launcher modal
        inAppStore.open(cleanUrl, options.title || 'Store', options.installUri);
        return;
      }
    }

    // 4. Browser Fallback
    try {
      await invoke('open_external_url', { url: cleanUrl });
    } catch {
      window.open(cleanUrl, '_blank');
    }
  }
}

export const storeLinkManager = new StoreLinkController();
export const openStoreLink = (options: {
  url: string;
  title?: string;
  appId?: number | string | null;
  platform?: string;
  installUri?: string;
  mode?: StoreOpenMode;
}) => storeLinkManager.open(options);
