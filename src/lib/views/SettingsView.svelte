<!--
  SettingsView.svelte — Comprehensive Settings for OmniLauncher (Svelte 5 Runes).
  TURA & Liquid Chrome aesthetic:
  - Network & DNS: Cloudflare 1.1.1.1, Google 8.8.8.8, Quad9, DoH, SOCKS5/HTTP Proxy, Cloudflare CDN
  - Storage & Drives: Native drive space visualizer & in-launcher directory picker
  - Currency Switcher: USD ($) & KZT (₸)
  - Themes & Vibrancy: Modular theme engine
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { networkStore } from '$lib/stores/network';
  import { directoryPicker, fetchSystemDrives, calculateUtilization, type DriveInfo } from '$lib/stores/filesystem';
  import { currencyStore } from '$lib/stores/store';
  import { themeEngine } from '$lib/themes/theme-engine';
  import { ecoStore } from '$lib/stores/eco.svelte';
  import { storeLinkManager, type StoreOpenMode } from '$lib/utils/storeLink';
  import { invoke } from '@tauri-apps/api/core';

  let activeTab = $state<'network' | 'storage' | 'appearance' | 'accounts'>('network');
  let systemDrives = $state<DriveInfo[]>([]);
  let saveFeedback = $state<string | null>(null);
  let customAccentInput = $state(themeEngine.customAccent || '#6366f1');
  let currentThemeId = $state(themeEngine.activeThemeId);

  // Sync theme changes
  onMount(() => {
    return themeEngine.subscribe((theme) => {
      currentThemeId = theme.id;
      if (themeEngine.customAccent) {
        customAccentInput = themeEngine.customAccent;
      }
    });
  });

  // Steam Account Binding state
  let steamProfile = $state<any>(null);
  let steamIdInput = $state('');
  let steamApiKeyInput = $state('');
  let steamSyncing = $state(false);

  // Proxy form temporary fields
  let proxyProtocol = $state<'socks5' | 'http' | 'https'>('socks5');
  let proxyHost = $state('127.0.0.1');
  let proxyPort = $state(1080);
  let proxyUser = $state('');
  let proxyPass = $state('');

  onMount(async () => {
    await networkStore.init();
    systemDrives = await fetchSystemDrives();

    // Parse existing proxy URL if present
    if (networkStore.settings.proxy_url) {
      const match = networkStore.settings.proxy_url.match(/^(socks5|http|https):\/\/(?:([^:]+):([^@]+)@)?(\[[0-9a-fA-F:]+\]|[^:]+):(\d+)$/);
      if (match) {
        proxyProtocol = match[1] as any;
        proxyUser = match[2] || '';
        proxyPass = match[3] || '';
        proxyHost = match[4];
        proxyPort = Number(match[5]);
      }
    }

    await loadSteamInfo();
  });

  async function loadSteamInfo() {
    try {
      const [prof, creds] = await Promise.all([
        invoke<any>('get_steam_profile').catch(() => null),
        invoke<any>('get_steam_credentials').catch(() => null),
      ]);
      steamProfile = prof;
      if (creds) {
        if (creds.steamId64) steamIdInput = creds.steamId64;
        if (creds.apiKey) steamApiKeyInput = creds.apiKey;
      } else if (prof?.steam_id) {
        steamIdInput = prof.steam_id;
      }
    } catch (e) {
      console.warn('Could not load steam profile/creds:', e);
    }
  }

  async function handleSaveSteamAccount() {
    steamSyncing = true;
    saveFeedback = null;
    try {
      await invoke('set_steam_credentials', {
        credentials: {
          apiKey: steamApiKeyInput.trim() || null,
          steamId64: steamIdInput.trim() || null,
          steamgriddbApiKey: null,
        },
      });
      await loadSteamInfo();
      saveFeedback = 'Steam аккаунт успешно подключен и сохранён!';
      setTimeout(() => (saveFeedback = null), 4000);
    } catch (err: any) {
      saveFeedback = `Ошибка подключения Steam: ${err?.message || String(err)}`;
    } finally {
      steamSyncing = false;
    }
  }

  async function handleAutoDetectSteamAccount() {
    steamSyncing = true;
    saveFeedback = null;
    try {
      const res: any = await invoke('auto_detect_steam_credentials');
      if (res?.steamId64) {
        steamIdInput = res.steamId64;
      }
      if (res?.apiKey) {
        steamApiKeyInput = res.apiKey;
      }
      await loadSteamInfo();
      saveFeedback = 'Steam аккаунт и API ключ успешно обнаружены и подключены автоматически!';
      setTimeout(() => (saveFeedback = null), 4000);
    } catch (err: any) {
      saveFeedback = `Ошибка авто-определения: ${err?.message || String(err)}`;
    } finally {
      steamSyncing = false;
    }
  }

  async function handleUnlinkSteamAccount() {
    steamSyncing = true;
    try {
      await invoke('set_steam_credentials', {
        credentials: {
          apiKey: null,
          steamId64: null,
          steamgriddbApiKey: null,
        },
      });
      steamIdInput = '';
      steamApiKeyInput = '';
      steamProfile = null;
      saveFeedback = 'Аккаунт Steam отвязан';
      setTimeout(() => (saveFeedback = null), 3000);
    } finally {
      steamSyncing = false;
    }
  }

  async function handleSaveNetwork() {
    saveFeedback = null;
    let computedProxyUrl: string | null = null;

    if (networkStore.settings.proxy_enabled) {
      let formattedHost = proxyHost.trim();
      if (formattedHost.includes(':') && !formattedHost.startsWith('[')) {
        formattedHost = `[${formattedHost}]`;
      }
      if (proxyUser && proxyPass) {
        computedProxyUrl = `${proxyProtocol}://${proxyUser}:${proxyPass}@${formattedHost}:${proxyPort}`;
      } else {
        computedProxyUrl = `${proxyProtocol}://${formattedHost}:${proxyPort}`;
      }
    }

    try {
      await networkStore.saveSettings({
        proxy_url: computedProxyUrl,
      });
      saveFeedback = 'Settings saved successfully';
      setTimeout(() => (saveFeedback = null), 3000);
    } catch (err: any) {
      saveFeedback = `Error: ${err?.message || String(err)}`;
    }
  }

  async function handlePickDownloadDir() {
    const selected = await directoryPicker.open(
      'Choose Default Game Installation Directory',
      networkStore.settings.download_directory || undefined
    );
    if (selected) {
      await networkStore.saveSettings({ download_directory: selected });
      saveFeedback = 'Download directory updated';
      setTimeout(() => (saveFeedback = null), 3000);
    }
  }
</script>

<div class="space-y-6 pb-12">
  <!-- Header Bar -->
  <div class="flex flex-wrap items-center justify-between gap-4 rounded-2xl border border-white/10 bg-[#121215]/80 p-5 backdrop-blur-xl">
    <div>
      <h2 class="text-lg font-black tracking-tight text-white flex items-center gap-2">
        SETTINGS
        <span class="rounded-full bg-white/10 px-2 py-0.5 font-mono text-[10px] uppercase tracking-widest text-zinc-300">
          Preferences
        </span>
      </h2>
      <p class="text-xs text-zinc-400">Network, DoH, Cloudflare CDN, drive storage & appearances</p>
    </div>

    <!-- Navigation Tabs -->
    <div class="flex items-center rounded-xl border border-white/15 bg-black/60 p-1">
      <button
        type="button"
        onclick={() => (activeTab = 'network')}
        class="rounded-lg px-3 py-1 font-mono text-xs font-semibold transition-all {activeTab === 'network'
          ? 'bg-white text-black shadow-md'
          : 'text-zinc-400 hover:text-white'}"
      >
        Network & DNS
      </button>
      <button
        type="button"
        onclick={() => (activeTab = 'storage')}
        class="rounded-lg px-3 py-1 font-mono text-xs font-semibold transition-all {activeTab === 'storage'
          ? 'bg-white text-black shadow-md'
          : 'text-zinc-400 hover:text-white'}"
      >
        Storage & Drives
      </button>
      <button
        type="button"
        onclick={() => (activeTab = 'accounts')}
        class="rounded-lg px-3 py-1 font-mono text-xs font-semibold transition-all {activeTab === 'accounts'
          ? 'bg-white text-black shadow-md'
          : 'text-zinc-400 hover:text-white'}"
      >
        Steam & Accounts
      </button>
      <button
        type="button"
        onclick={() => (activeTab = 'appearance')}
        class="rounded-lg px-3 py-1 font-mono text-xs font-semibold transition-all {activeTab === 'appearance'
          ? 'bg-white text-black shadow-md'
          : 'text-zinc-400 hover:text-white'}"
      >
        Themes & UI
      </button>
    </div>
  </div>

  {#if saveFeedback}
    <div class="rounded-xl border border-white/20 bg-white/10 p-3 text-xs text-white font-mono animate-in fade-in">
      {saveFeedback}
    </div>
  {/if}

  <!-- TAB 1: NETWORK & DNS -->
  {#if activeTab === 'network'}
    <div class="space-y-6">
      <!-- 1. DNS & DoH Provider Selection -->
      <div class="rounded-2xl border border-white/10 bg-[#121215]/90 p-6 space-y-4">
        <div class="flex items-center justify-between border-b border-white/10 pb-3">
          <div>
            <h3 class="text-sm font-bold text-white tracking-tight">DNS & Encryption (DoH)</h3>
            <p class="text-xs text-zinc-400">Select authoritative DNS resolver for encrypted domain lookup</p>
          </div>
          <label class="flex items-center gap-2 cursor-pointer">
            <span class="text-xs font-mono text-zinc-300">DNS-over-HTTPS</span>
            <input
              type="checkbox"
              bind:checked={networkStore.settings.doh_enabled}
              onchange={handleSaveNetwork}
              class="h-4 w-4 rounded border-white/20 bg-black/60 text-white focus:ring-0"
            />
          </label>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-3">
          {#each [
            { id: 'cloudflare', name: 'Cloudflare', ip: '1.1.1.1', desc: 'Fastest & privacy focused' },
            { id: 'google', name: 'Google DNS', ip: '8.8.8.8', desc: 'Global high availability' },
            { id: 'quad9', name: 'Quad9', ip: '9.9.9.9', desc: 'Built-in malware blocking' },
            { id: 'system', name: 'System Default', ip: 'Auto', desc: 'Use Windows system adapter' },
          ] as prov}
            <button
              type="button"
              onclick={() => {
                networkStore.settings.dns_provider = prov.id as any;
                handleSaveNetwork();
              }}
              class="flex flex-col justify-between rounded-xl border p-3 text-left transition-all {networkStore.settings.dns_provider === prov.id
                ? 'border-white/50 bg-white/10 shadow-md'
                : 'border-white/10 bg-white/[0.02] hover:border-white/25 hover:bg-white/[0.05]'}"
            >
              <div>
                <span class="font-bold text-xs text-white">{prov.name}</span>
                <p class="text-[11px] text-zinc-400 mt-0.5">{prov.desc}</p>
              </div>
              <span class="mt-2 font-mono text-[10px] text-zinc-500">IP: {prov.ip}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- 2. Cloudflare CDN Media Routing -->
      <div class="rounded-2xl border border-white/10 bg-[#121215]/90 p-6 space-y-4">
        <div class="flex items-center justify-between border-b border-white/10 pb-3">
          <div>
            <h3 class="text-sm font-bold text-white tracking-tight">Cloudflare CDN Media Routing</h3>
            <p class="text-xs text-zinc-400">Stream game covers and trailers via high-speed edge proxy with WebP optimization</p>
          </div>
          <label class="flex items-center gap-2 cursor-pointer">
            <span class="text-xs font-mono text-zinc-300">Proxy Media</span>
            <input
              type="checkbox"
              bind:checked={networkStore.settings.cloudflare_media_enabled}
              onchange={handleSaveNetwork}
              class="h-4 w-4 rounded border-white/20 bg-black/60 text-white focus:ring-0"
            />
          </label>
        </div>

        <div class="rounded-xl border border-white/10 bg-black/40 p-3 font-mono text-xs text-zinc-400">
          <div class="flex items-center justify-between">
            <span>Edge Gateway:</span>
            <span class="text-white">media.omnilauncher.app</span>
          </div>
          <div class="flex items-center justify-between mt-1 text-[11px]">
            <span>Cache-Control Policy:</span>
            <span class="text-emerald-400">public, max-age=604800, stale-while-revalidate=86400</span>
          </div>
        </div>
      </div>

      <!-- 3. Proxy Tunnel Configuration -->
      <div class="rounded-2xl border border-white/10 bg-[#121215]/90 p-6 space-y-4">
        <div class="flex items-center justify-between border-b border-white/10 pb-3">
          <div>
            <h3 class="text-sm font-bold text-white tracking-tight">Proxy Tunnel (SOCKS5 / HTTP)</h3>
            <p class="text-xs text-zinc-400">Route all store and media requests through a secure proxy</p>
          </div>
          <label class="flex items-center gap-2 cursor-pointer">
            <span class="text-xs font-mono text-zinc-300">Enable Proxy</span>
            <input
              type="checkbox"
              bind:checked={networkStore.settings.proxy_enabled}
              onchange={handleSaveNetwork}
              class="h-4 w-4 rounded border-white/20 bg-black/60 text-white focus:ring-0"
            />
          </label>
        </div>

        {#if networkStore.settings.proxy_enabled}
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
            <div>
              <label for="proxy-protocol" class="block text-[11px] font-mono text-zinc-400 mb-1">Protocol</label>
              <select
                id="proxy-protocol"
                bind:value={proxyProtocol}
                onchange={handleSaveNetwork}
                class="w-full rounded-xl border border-white/10 bg-black/60 px-3 py-1.5 text-xs text-white outline-none focus:border-white/30"
              >
                <option value="socks5">SOCKS5</option>
                <option value="http">HTTP</option>
                <option value="https">HTTPS</option>
              </select>
            </div>
            <div>
              <label for="proxy-host" class="block text-[11px] font-mono text-zinc-400 mb-1">Host / IP</label>
              <input
                id="proxy-host"
                type="text"
                bind:value={proxyHost}
                onblur={handleSaveNetwork}
                placeholder="127.0.0.1"
                class="w-full rounded-xl border border-white/10 bg-black/60 px-3 py-1.5 text-xs text-white outline-none focus:border-white/30"
              />
            </div>
            <div>
              <label for="proxy-port" class="block text-[11px] font-mono text-zinc-400 mb-1">Port</label>
              <input
                id="proxy-port"
                type="number"
                bind:value={proxyPort}
                onblur={handleSaveNetwork}
                min="1"
                max="65535"
                placeholder="1080"
                class="w-full rounded-xl border border-white/10 bg-black/60 px-3 py-1.5 text-xs text-white outline-none focus:border-white/30"
              />
            </div>
          </div>
        {/if}

        <!-- Connection Test Button -->
        <div class="flex items-center justify-between border-t border-white/10 pt-4">
          <div class="flex items-center gap-3">
            <button
              type="button"
              disabled={networkStore.testing}
              onclick={() => networkStore.testConnection()}
              class="rounded-xl border border-white/15 bg-white/10 px-4 py-2 font-mono text-xs font-bold text-white hover:bg-white/20 transition-all disabled:opacity-50"
            >
              {networkStore.testing ? 'Testing...' : 'Test Connection'}
            </button>

            {#if networkStore.testResult}
              <span
                class="font-mono text-xs {networkStore.testResult.success ? 'text-emerald-400' : 'text-rose-400'}"
              >
                {networkStore.testResult.success
                  ? `✓ Connected (${networkStore.testResult.latency_ms} ms)`
                  : `✕ Failed: ${networkStore.testResult.error || 'Connection timed out'}`}
              </span>
            {/if}
          </div>

          <button
            type="button"
            onclick={handleSaveNetwork}
            class="rounded-xl bg-white px-5 py-2 text-xs font-bold text-black shadow-lg hover:bg-zinc-200 transition-all"
          >
            Save Network Settings
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- TAB 2: STORAGE & DRIVES -->
  {#if activeTab === 'storage'}
    <div class="space-y-6">
      <div class="rounded-2xl border border-white/10 bg-[#121215]/90 p-6 space-y-4">
        <div class="border-b border-white/10 pb-3">
          <h3 class="text-sm font-bold text-white tracking-tight">System Storage Drives</h3>
          <p class="text-xs text-zinc-400">Available storage volumes detected on this machine</p>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-3.5">
          {#each systemDrives as drive}
            {@const util = calculateUtilization(drive.totalBytes, drive.freeBytes)}
            <div class="rounded-xl border border-white/10 bg-black/40 p-4 space-y-2">
              <div class="flex items-center justify-between">
                <span class="font-bold text-sm text-white">{drive.label}</span>
                <span class="font-mono text-xs text-zinc-400">{drive.drive}</span>
              </div>
              <div class="h-2 w-full overflow-hidden rounded-full bg-white/10">
                <div
                  class="h-full rounded-full transition-all {util > 90
                    ? 'bg-rose-500'
                    : util > 80
                    ? 'bg-amber-400'
                    : 'bg-emerald-400'}"
                  style="width: {util}%"
                ></div>
              </div>
              <div class="flex items-center justify-between font-mono text-[11px] text-zinc-400">
                <span>{drive.freeGb.toFixed(1)} GB Free</span>
                <span>{drive.totalGb.toFixed(1)} GB Total</span>
              </div>
            </div>
          {/each}
        </div>
      </div>

      <!-- Default Download Directory -->
      <div class="rounded-2xl border border-white/10 bg-[#121215]/90 p-6 space-y-4">
        <div class="border-b border-white/10 pb-3">
          <h3 class="text-sm font-bold text-white tracking-tight">Default Installation Directory</h3>
          <p class="text-xs text-zinc-400">Where emulators and games are downloaded without prompting</p>
        </div>

        <div class="flex items-center gap-3">
          <input
            type="text"
            readonly
            value={networkStore.settings.download_directory || 'C:\\OmniLauncher\\Games'}
            class="flex-1 rounded-xl border border-white/10 bg-black/60 px-3.5 py-2 font-mono text-xs text-white outline-none"
          />
          <button
            type="button"
            onclick={handlePickDownloadDir}
            class="rounded-xl bg-white px-4 py-2 text-xs font-bold text-black hover:bg-zinc-200 transition-colors shadow-md"
          >
            Change Folder...
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- TAB 3: THEMES & UI -->
  {#if activeTab === 'appearance'}
    <div class="space-y-6">
      <!-- Unified Theme Selection: 5 Distinct Themes -->
      <div class="rounded-2xl border border-[var(--border-subtle)] bg-[var(--bg-surface-base)] p-6 space-y-4">
        <div class="border-b border-[var(--border-subtle)] pb-3 flex items-center justify-between">
          <div>
            <h3 class="text-sm font-bold text-[var(--text-primary)] tracking-tight">Темы оформления OmniLauncher</h3>
            <p class="text-xs text-[var(--text-secondary)]">5 тщательно настроенных цветовых стилей с динамической адаптацией интерфейса</p>
          </div>
          <span class="font-mono text-xs text-[var(--accent-primary)] font-bold">
            Текущая: {themeEngine.getActiveTheme().name}
          </span>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-3">
          {#each themeEngine.getAvailableThemes() as theme}
            {@const isSelected = currentThemeId === theme.id}
            {@const bg = (theme.tokens as any)?.['--bg-canvas'] || '#09090b'}
            {@const surface = (theme.tokens as any)?.['--bg-surface-base'] || '#121215'}
            {@const accent = (theme.tokens as any)?.['--accent-primary'] || '#6366f1'}
            {@const text = (theme.tokens as any)?.['--text-primary'] || '#ffffff'}
            <button
              type="button"
              onclick={() => {
                themeEngine.applyTheme(theme.id);
                currentThemeId = theme.id;
              }}
              class="relative flex flex-col justify-between rounded-xl border p-4 text-left transition-all duration-200 {isSelected
                ? 'border-[var(--accent-primary)] ring-2 ring-[var(--accent-primary)]/40 shadow-lg scale-[1.02]'
                : 'border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] hover:border-[var(--border-active)] hover:bg-[var(--bg-surface-floating)]'}"
              style="background-color: {surface};"
            >
              <div>
                <div class="flex items-center justify-between">
                  <span class="font-bold text-xs" style="color: {text}">{theme.name}</span>
                  {#if isSelected}
                    <span class="flex h-2 w-2 rounded-full" style="background-color: {accent}; box-shadow: 0 0 8px {accent};"></span>
                  {/if}
                </div>
                <p class="text-[10px] opacity-75 mt-1 line-clamp-2" style="color: {text}">
                  {theme.description || 'Флагманская тема оформления'}
                </p>
              </div>

              <div class="mt-4 pt-3 border-t border-black/10 dark:border-white/10 flex items-center justify-between">
                <!-- Color Swatches -->
                <div class="flex items-center gap-1.5">
                  <span class="h-3.5 w-3.5 rounded-full border border-black/20 dark:border-white/20 shadow-sm" style="background-color: {bg}" title="Фон"></span>
                  <span class="h-3.5 w-3.5 rounded-full border border-black/20 dark:border-white/20 shadow-sm" style="background-color: {surface}" title="Поверхность"></span>
                  <span class="h-3.5 w-3.5 rounded-full border border-black/20 dark:border-white/20 shadow-sm" style="background-color: {accent}" title="Акцент"></span>
                </div>
                <span class="font-mono text-[9px] opacity-60" style="color: {text}">{theme.id}</span>
              </div>
            </button>
          {/each}
        </div>

        <!-- Custom Accent Color Picker -->
        <div class="mt-6 pt-4 border-t border-[var(--border-hairline)] flex flex-wrap items-center justify-between gap-4">
          <div class="space-y-0.5">
            <span class="text-xs font-bold text-[var(--text-primary)]">Пользовательский цвет акцента</span>
            <p class="text-[11px] text-[var(--text-secondary)]">Переопределяет цвет кнопок, бейджей и свечения для выбранной темы</p>
          </div>

          <div class="flex flex-wrap items-center gap-2">
            <!-- Preset Palette -->
            {#each ['#6366f1', '#06b6d4', '#10b981', '#f59e0b', '#ec4899', '#8b5cf6'] as preset}
              <button
                type="button"
                onclick={() => {
                  customAccentInput = preset;
                  themeEngine.setCustomAccent(preset);
                }}
                class="h-6 w-6 rounded-full border border-white/20 transition-transform hover:scale-110 shadow-sm"
                style="background-color: {preset};"
                title={`Выбрать акцент ${preset}`}
              ></button>
            {/each}

            <!-- Color Input Picker -->
            <div class="flex items-center gap-2 ml-2">
              <input
                type="color"
                bind:value={customAccentInput}
                onchange={(e) => themeEngine.setCustomAccent((e.currentTarget as HTMLInputElement).value)}
                class="h-7 w-8 cursor-pointer rounded border border-[var(--border-subtle)] bg-transparent p-0.5"
                title="Выбрать свой цвет"
              />
              <span class="font-mono text-[11px] text-[var(--text-secondary)]">{customAccentInput}</span>
            </div>

            <!-- Reset Button -->
            {#if themeEngine.customAccent}
              <button
                type="button"
                onclick={() => {
                  themeEngine.setCustomAccent(null);
                  customAccentInput = (themeEngine.getActiveTheme().tokens as any)?.['--accent-primary'] || '#6366f1';
                }}
                class="btn-minimal px-2.5 py-1 text-[11px] font-mono ml-2"
                title="Сбросить акцент к цветам темы по умолчанию"
              >
                Сброс
              </button>
            {/if}
          </div>
        </div>
      </div>

      <!-- Store Link Opening Preference -->
      <div class="rounded-2xl border border-[var(--border-subtle)] bg-[var(--bg-surface-base)] p-6 space-y-4">
        <div class="border-b border-[var(--border-subtle)] pb-3">
          <h3 class="text-sm font-bold text-[var(--text-primary)] tracking-tight">Способ открытия ссылок магазинов (Steam / Epic)</h3>
          <p class="text-xs text-[var(--text-secondary)]">Где открывать страницы игр при клике на карточки, скидки и каталог</p>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <button
            type="button"
            onclick={() => storeLinkManager.setMode('client')}
            class="flex flex-col justify-between rounded-xl border p-4 text-left transition-all {storeLinkManager.mode === 'client'
              ? 'border-[var(--accent-primary)] bg-[var(--bg-surface-elevated)] ring-1 ring-[var(--accent-primary)]/50 shadow-md'
              : 'border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)]/40 hover:border-[var(--border-active)]'}"
          >
            <div>
              <div class="flex items-center justify-between">
                <span class="font-bold text-xs text-[var(--text-primary)] flex items-center gap-1.5">
                  <span>🎮</span> В клиенте Steam / Epic
                </span>
                <span class="rounded bg-emerald-500/20 text-emerald-400 font-mono text-[8px] px-1 py-0.5 font-bold">
                  АККАУНТ
                </span>
              </div>
              <p class="text-[11px] text-[var(--text-secondary)] mt-1.5">
                Мгновенно открывает в приложении Steam/Epic. Вы уже авторизованы в своём аккаунте.
              </p>
            </div>
            <span class="font-mono text-[10px] text-emerald-400 mt-3 font-semibold">
              {storeLinkManager.mode === 'client' ? '✓ Активно' : 'Выбрать'}
            </span>
          </button>

          <button
            type="button"
            onclick={() => storeLinkManager.setMode('launcher')}
            class="flex flex-col justify-between rounded-xl border p-4 text-left transition-all {storeLinkManager.mode === 'launcher'
              ? 'border-[var(--accent-primary)] bg-[var(--bg-surface-elevated)] ring-1 ring-[var(--accent-primary)]/50 shadow-md'
              : 'border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)]/40 hover:border-[var(--border-active)]'}"
          >
            <div>
              <span class="font-bold text-xs text-[var(--text-primary)] flex items-center gap-1.5">
                <span>❐</span> В окне лаунчера
              </span>
              <p class="text-[11px] text-[var(--text-secondary)] mt-1.5">
                Отдельное встроенное окно WebView внутри OmniLauncher с сохранением cookies и сессий.
              </p>
            </div>
            <span class="font-mono text-[10px] text-[var(--accent-primary)] mt-3 font-semibold">
              {storeLinkManager.mode === 'launcher' ? '✓ Активно' : 'Выбрать'}
            </span>
          </button>

          <button
            type="button"
            onclick={() => storeLinkManager.setMode('browser')}
            class="flex flex-col justify-between rounded-xl border p-4 text-left transition-all {storeLinkManager.mode === 'browser'
              ? 'border-[var(--accent-primary)] bg-[var(--bg-surface-elevated)] ring-1 ring-[var(--accent-primary)]/50 shadow-md'
              : 'border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)]/40 hover:border-[var(--border-active)]'}"
          >
            <div>
              <span class="font-bold text-xs text-[var(--text-primary)] flex items-center gap-1.5">
                <span>🌐</span> В браузере Windows
              </span>
              <p class="text-[11px] text-[var(--text-secondary)] mt-1.5">
                Открывает новую вкладку в вашем основном системном веб-браузере (Edge, Chrome, Firefox).
              </p>
            </div>
            <span class="font-mono text-[10px] text-[var(--accent-primary)] mt-3 font-semibold">
              {storeLinkManager.mode === 'browser' ? '✓ Активно' : 'Выбрать'}
            </span>
          </button>
        </div>
      </div>

      <!-- Currency Switcher -->
      <div class="rounded-2xl border border-[var(--border-subtle)] bg-[var(--bg-surface-base)] p-6 space-y-4">
        <div class="border-b border-[var(--border-subtle)] pb-3">
          <h3 class="text-sm font-bold text-[var(--text-primary)] tracking-tight">Валюта магазина и цен</h3>
          <p class="text-xs text-[var(--text-secondary)]">Основная валюта для каталога Steam, радара скидок и бесплатных раздач</p>
        </div>

        <div class="flex flex-wrap items-center gap-3">
          <button
            type="button"
            onclick={() => currencyStore.setCurrency('USD')}
            class="rounded-xl border px-4 py-2 font-mono text-xs font-bold transition-all {currencyStore.code === 'USD'
              ? 'border-[var(--accent-primary)] bg-[var(--accent-primary)] text-white shadow-md'
              : 'border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
          >
            $ USD (United States Dollar)
          </button>
          <button
            type="button"
            onclick={() => currencyStore.setCurrency('KZT')}
            class="rounded-xl border px-4 py-2 font-mono text-xs font-bold transition-all {currencyStore.code === 'KZT'
              ? 'border-[var(--accent-primary)] bg-[var(--accent-primary)] text-white shadow-md'
              : 'border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
          >
            ₸ KZT (Казахстанский тенге • Курс 495.0)
          </button>
        </div>
      </div>

      <!-- Eco Mode Switcher -->
      <div class="rounded-2xl border border-[var(--border-subtle)] bg-[var(--bg-surface-base)] p-6 space-y-4">
        <div class="flex items-center justify-between border-b border-[var(--border-subtle)] pb-3">
          <div>
            <h3 class="text-sm font-bold text-[var(--text-primary)] tracking-tight flex items-center gap-2">
              <span>🍃</span>
              <span>Эко-режим для ноутбуков и портативок</span>
              {#if ecoStore.enabled}
                <span class="rounded bg-emerald-500/20 px-2 py-0.5 font-mono text-[10px] text-emerald-300 font-bold">АКТИВЕН</span>
              {/if}
            </h3>
            <p class="text-xs text-[var(--text-secondary)]">Отключает анимации и размытие для экономии батареи (OLED Black)</p>
          </div>
          <button
            type="button"
            onclick={() => ecoStore.toggle()}
            class="rounded-xl px-4 py-2 font-mono text-xs font-bold transition-all {ecoStore.enabled
              ? 'bg-emerald-500 text-black shadow-lg shadow-emerald-500/20'
              : 'border border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] text-[var(--text-secondary)] hover:bg-[var(--bg-surface-floating)]'}"
          >
            {ecoStore.enabled ? 'Включен' : 'Выключен'}
          </button>
        </div>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 font-mono text-xs text-[var(--text-secondary)]">
          <div class="rounded-lg bg-[var(--bg-surface-elevated)] p-3 border border-[var(--border-hairline)]">
            <span class="text-zinc-500 block text-[10px] uppercase">Уровень батареи</span>
            <span class="text-[var(--text-primary)] font-bold">{ecoStore.batteryLevel !== null ? `${ecoStore.batteryLevel}%` : 'Подключено к сети'}</span>
          </div>
          <div class="rounded-lg bg-[var(--bg-surface-elevated)] p-3 border border-[var(--border-hairline)]">
            <span class="text-zinc-500 block text-[10px] uppercase">Источник питания</span>
            <span class="text-[var(--text-primary)] font-bold">{ecoStore.isCharging ? 'Зарядка / Сеть 220V' : 'Работа от аккумулятора'}</span>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- TAB 4: ACCOUNTS & STEAM -->
  {#if activeTab === 'accounts'}
    <div class="space-y-6">
      <!-- Active Steam Account Card -->
      <div class="rounded-2xl border border-white/10 bg-[#121215]/90 p-6 space-y-4">
        <div class="flex items-center justify-between border-b border-white/10 pb-3">
          <div>
            <h3 class="text-sm font-bold text-white tracking-tight">Активный профиль Steam</h3>
            <p class="text-xs text-zinc-400">Текущий синхронизированный профиль игрока и достижения</p>
          </div>
          <button
            type="button"
            onclick={handleAutoDetectSteamAccount}
            disabled={steamSyncing}
            class="btn-minimal px-3 py-1.5 text-xs font-mono"
            title="Автоматически определить профиль из запущенного клиента Steam на этом компьютере"
          >
            {steamSyncing ? 'Поиск…' : '🔄 Авто-определение'}
          </button>
        </div>

        {#if steamProfile}
          <div class="flex flex-wrap items-center justify-between gap-4 rounded-xl border border-white/10 bg-black/40 p-4">
            <div class="flex items-center gap-4">
              <img
                src={steamProfile.avatar_url || 'https://avatars.steamstatic.com/sample_full.jpg'}
                alt=""
                class="h-14 w-14 rounded-full border-2 border-white/20 shadow-md object-cover"
              />
              <div>
                <div class="flex items-center gap-2">
                  <h4 class="text-base font-bold text-white">{steamProfile.persona_name}</h4>
                  <span class="rounded-md border border-emerald-500/30 bg-emerald-500/10 px-2 py-0.5 font-mono text-[10px] text-emerald-300">
                    Подключен
                  </span>
                </div>
                <p class="font-mono text-xs text-zinc-400 mt-0.5">SteamID64: {steamProfile.steam_id}</p>
                {#if steamProfile.profile_url}
                  <a
                    href={steamProfile.profile_url}
                    target="_blank"
                    class="font-mono text-[11px] text-sky-400 hover:underline mt-0.5 inline-block"
                  >
                    Открыть профиль Steam ↗
                  </a>
                {/if}
              </div>
            </div>

            <button
              type="button"
              onclick={handleUnlinkSteamAccount}
              disabled={steamSyncing}
              class="rounded-xl border border-rose-500/30 bg-rose-500/10 px-3 py-1.5 font-mono text-xs text-rose-300 hover:bg-rose-500/20 transition-all"
            >
              Отвязать аккаунт
            </button>
          </div>
        {:else}
          <div class="rounded-xl border border-white/10 bg-white/[0.02] p-5 text-center text-xs text-zinc-400">
            Аккаунт Steam не привязан вручную. Нажмите «Авто-определение» или введите данные ниже.
          </div>
        {/if}
      </div>

      <!-- Bind Another Steam Account Form -->
      <div class="rounded-2xl border border-white/10 bg-[#121215]/90 p-6 space-y-4">
        <div class="border-b border-white/10 pb-3">
          <h3 class="text-sm font-bold text-white tracking-tight">Привязать другой аккаунт Steam</h3>
          <p class="text-xs text-zinc-400">Позволяет другу подключить свой SteamID и API ключ на любом компьютере</p>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label for="steam-id-input" class="block font-mono text-xs text-zinc-300">SteamID64 друга или ваш</label>
            <input
              id="steam-id-input"
              type="text"
              bind:value={steamIdInput}
              placeholder="76561198000000000"
              class="w-full rounded-xl border border-white/10 bg-black/50 px-3.5 py-2 font-mono text-xs text-white placeholder-zinc-500 outline-none transition-colors focus:border-white/30"
            />
            <p class="font-mono text-[10px] text-zinc-500">
              17-значный цифровой ID Steam (из URL профиля steamcommunity.com/profiles/ID)
            </p>
          </div>

          <div class="space-y-1.5">
            <label for="steam-api-input" class="block font-mono text-xs text-zinc-300">Steam Web API Key (опционально)</label>
            <input
              id="steam-api-input"
              type="password"
              bind:value={steamApiKeyInput}
              placeholder="32-значный ключ API"
              class="w-full rounded-xl border border-white/10 bg-black/50 px-3.5 py-2 font-mono text-xs text-white placeholder-zinc-500 outline-none transition-colors focus:border-white/30"
            />
            <p class="font-mono text-[10px] text-zinc-500">
              Нужен для скрытых профилей и списка друзей (получить: steamcommunity.com/dev/apikey)
            </p>
          </div>
        </div>

        <div class="flex items-center justify-between pt-2">
          <div class="text-[11px] text-zinc-400">
            ℹ️ <span class="text-zinc-300">Подсказка:</span> Если лаунчер открыт на компьютере друга со запущенным Steam, кликните «Авто-определение» выше — он определит аккаунт без ручного ввода!
          </div>

          <button
            type="button"
            onclick={handleSaveSteamAccount}
            disabled={steamSyncing}
            class="btn-chrome px-5 py-2 text-xs font-bold"
          >
            {steamSyncing ? 'Синхронизация…' : 'Сохранить и подключить'}
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
