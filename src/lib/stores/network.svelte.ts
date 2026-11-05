import { invoke } from '@tauri-apps/api/core';

export interface NetworkSettings {
  dns_provider: 'cloudflare' | 'google' | 'quad9' | 'system' | 'custom';
  doh_enabled: boolean;
  custom_doh_url: string | null;
  proxy_enabled: boolean;
  proxy_url: string | null;
  cloudflare_media_enabled: boolean;
  cloudflare_media_endpoint: string | null;
  download_directory: string | null;
}

export interface ConnectionTestResult {
  success: boolean;
  latency_ms: number;
  resolved_ip: string | null;
  doh_active: boolean;
  proxy_active: boolean;
  error?: string;
}

export const CLOUDFLARE_MEDIA_PROXY_BASE = 'https://media.omnilauncher.app/cdn-cgi/image';

export function getProxiedCoverUrl(originalUrl: string, width = 600, format = 'webp'): string {
  if (!originalUrl) return getFallbackPlaceholder();
  if (
    originalUrl.startsWith('data:') ||
    originalUrl.startsWith('asset:') ||
    originalUrl.includes('localhost') ||
    originalUrl.includes('media.omnilauncher.app')
  ) {
    return originalUrl;
  }
  const encoded = encodeURIComponent(originalUrl);
  return `${CLOUDFLARE_MEDIA_PROXY_BASE}/width=${width},format=${format}/${encoded}`;
}

export function getFallbackPlaceholder(): string {
  return 'assets/placeholders/fallback_vector.svg';
}

export function resolveMediaWithFallback(cdnUrl: string, fallbackPath: string): string {
  if (!cdnUrl || cdnUrl.includes('error_404') || cdnUrl.includes('502') || cdnUrl.includes('504')) {
    return fallbackPath;
  }
  return cdnUrl;
}

export function pickOptimalFormat(acceptHeader: string): string {
  if (acceptHeader.includes('image/avif')) return 'avif';
  if (acceptHeader.includes('image/webp')) return 'webp';
  return 'jpeg';
}

export function isValidDohUrl(url: string): boolean {
  try {
    const parsed = new URL(url);
    return parsed.protocol === 'https:' && parsed.pathname.length > 1;
  } catch {
    return false;
  }
}

export function parseProxyUri(uri: string) {
  const match = uri.match(/^(socks5|http|https):\/\/(?:([^:]+):([^@]+)@)?(\[[0-9a-fA-F:]+\]|[^:]+):(\d+)$/);
  if (!match) throw new Error('Invalid proxy URI format');
  return {
    protocol: match[1],
    username: match[2] || null,
    password: match[3] || null,
    host: match[4],
    port: Number(match[5]),
  };
}

export function shouldBypassProxy(hostname: string): boolean {
  const bypassList = ['localhost', '127.0.0.1', '::1', '*.local'];
  return bypassList.some((rule) => {
    if (rule.startsWith('*.')) return hostname.endsWith(rule.slice(2));
    return hostname === rule;
  });
}

class NetworkStore {
  settings = $state<NetworkSettings>({
    dns_provider: 'cloudflare',
    doh_enabled: true,
    custom_doh_url: null,
    proxy_enabled: false,
    proxy_url: null,
    cloudflare_media_enabled: true,
    cloudflare_media_endpoint: CLOUDFLARE_MEDIA_PROXY_BASE,
    download_directory: null,
  });

  testing = $state<boolean>(false);
  testResult = $state<ConnectionTestResult | null>(null);

  async init() {
    await this.loadSettings();
  }

  async loadSettings() {
    try {
      const res = await invoke<NetworkSettings>('get_network_settings');
      this.settings = res;
    } catch (err) {
      console.warn('Could not load network settings from backend:', err);
    }
  }

  async saveSettings(updated: Partial<NetworkSettings>) {
    this.settings = { ...this.settings, ...updated };
    try {
      await invoke('update_network_settings', { settings: this.settings });
    } catch (err) {
      console.error('Failed to update network settings:', err);
      throw err;
    }
  }

  async testConnection(): Promise<ConnectionTestResult> {
    this.testing = true;
    this.testResult = null;
    try {
      const res = await invoke<ConnectionTestResult>('test_network_connection', {
        provider: this.settings.dns_provider,
        proxyUrl: this.settings.proxy_enabled ? this.settings.proxy_url : null,
      });
      this.testResult = res;
      return res;
    } catch (err) {
      const fail: ConnectionTestResult = {
        success: false,
        latency_ms: 0,
        resolved_ip: null,
        doh_active: false,
        proxy_active: false,
        error: String(err),
      };
      this.testResult = fail;
      return fail;
    } finally {
      this.testing = false;
    }
  }
}

export const networkStore = new NetworkStore();
