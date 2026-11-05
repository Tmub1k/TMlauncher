<!--
  TitleBar.svelte — Sleek frameless header (height: 44px)
  Provides window dragging, brand identity, live status, quick theme switcher,
  and minimal window controls (minimize, maximize, close).
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { themeEngine } from '../themes/theme-engine';
  import type { ThemeDefinition } from '../themes/types';
  import { fetchSteamProfile, openInAppStore, type SteamProfile } from '../stores/library';
  import { ecoStore } from '../stores/eco.svelte';

  let currentTheme = $state<ThemeDefinition>(themeEngine.getActiveTheme());
  let availableThemes = $state<ThemeDefinition[]>(themeEngine.getAvailableThemes());
  let themeMenuOpen = $state(false);
  let steamProfile = $state<SteamProfile | null>(null);

  onMount(() => {
    const unsubscribe = themeEngine.subscribe((theme) => {
      currentTheme = theme;
      availableThemes = themeEngine.getAvailableThemes();
    });

    void (async () => {
      try {
        steamProfile = await fetchSteamProfile();
      } catch {
        // Offline or not linked
      }
    })();

    return () => {
      unsubscribe();
    };
  });

  function selectTheme(themeId: string) {
    themeEngine.applyTheme(themeId);
    themeMenuOpen = false;
  }

  async function handleMinimize() {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().minimize();
    } catch {
      // Browser preview mode
    }
  }

  async function handleToggleMaximize() {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().toggleMaximize();
    } catch {
      // Browser preview mode
    }
  }

  async function handleClose() {
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      await getCurrentWindow().close();
    } catch {
      // Browser preview mode
    }
  }
</script>

<header
  data-tauri-drag-region
  class="relative z-50 flex h-11 w-full select-none items-center justify-between border-b border-[var(--border-hairline)] bg-[var(--bg-canvas)]/85 px-3.5 backdrop-blur-xl"
>
  <!-- Left Zone: Identity & Live Connection Status -->
  <div data-tauri-drag-region class="flex items-center gap-2.5">
    <!-- Brand Monogram -->
    <div
      class="flex h-6 w-6 items-center justify-center rounded-md bg-white font-mono text-[10px] font-black text-black shadow-sm"
    >
      OL
    </div>

    <!-- App Title & Subtitle -->
    <span class="font-sans text-xs font-black tracking-widest text-white">OMNI</span>

    <!-- Hairline Separator -->
    <div class="h-3 w-px bg-white/10"></div>

    <!-- Live Status Pip -->
    <div class="flex items-center gap-1.5 font-mono text-[10px] tracking-wider text-zinc-400">
      <span class="h-1.5 w-1.5 rounded-full bg-emerald-400 shadow-[0_0_6px_#34d399]"></span>
      <span class="hidden sm:inline">ONLINE</span>
    </div>
  </div>

  <!-- Center Zone: Quick Theme Switcher Pill -->
  <div class="relative flex items-center">
    <button
      type="button"
      onclick={() => (themeMenuOpen = !themeMenuOpen)}
      class="flex items-center gap-1.5 rounded-full border border-white/10 bg-white/[0.04] px-3 py-1 font-mono text-[10px] uppercase tracking-wider text-zinc-300 transition-all hover:border-white/25 hover:bg-white/10 hover:text-white"
      title="Switch Theme"
    >
      <span class="h-1.5 w-1.5 rounded-full bg-white/70"></span>
      <span class="max-w-[120px] truncate">{currentTheme.name}</span>
      <svg class="h-3 w-3 fill-current opacity-60" viewBox="0 0 20 20">
        <path d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"/>
      </svg>
    </button>

    {#if themeMenuOpen}
      <!-- Theme Selection Dropdown -->
      <div
        class="absolute left-1/2 top-full mt-1.5 w-52 -translate-x-1/2 overflow-hidden rounded-xl border border-white/15 bg-[var(--bg-surface-elevated)] p-1 shadow-2xl backdrop-blur-2xl z-50"
      >
        <div class="px-2.5 py-1 text-[9px] font-bold uppercase tracking-widest text-zinc-500">
          Themes
        </div>
        {#each availableThemes as theme}
          <button
            type="button"
            onclick={() => selectTheme(theme.id)}
            class="flex w-full items-center justify-between rounded-lg px-2.5 py-1.5 text-left text-xs transition-colors {theme.id ===
            currentTheme.id
              ? 'bg-white text-black font-semibold'
              : 'text-zinc-300 hover:bg-white/10 hover:text-white'}"
          >
            <span class="truncate">{theme.name}</span>
            {#if theme.id === currentTheme.id}
              <span class="h-1.5 w-1.5 rounded-full bg-black"></span>
            {/if}
          </button>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Right Zone: Steam Profile & Window Controls -->
  <div class="flex items-center gap-2">
    {#if steamProfile}
      <a
        href={steamProfile.profile_url}
        target="_blank"
        rel="noreferrer"
        onclick={(e) => {
          e.preventDefault();
          openInAppStore(steamProfile?.profile_url || '', steamProfile?.persona_name || 'Steam Profile');
        }}
        title={`Steam: ${steamProfile.persona_name} (${steamProfile.steam_id})`}
        class="flex items-center gap-1.5 rounded-full border border-white/10 bg-white/[0.04] px-2 py-0.5 text-xs text-zinc-300 hover:border-white/20 hover:text-white hover:bg-white/10 transition-all"
      >
        <img
          src={steamProfile.avatar_url}
          alt={steamProfile.persona_name}
          class="h-3.5 w-3.5 rounded-full object-cover"
          onerror={(e) => {
            (e.currentTarget as HTMLImageElement).src = 'https://avatars.steamstatic.com/sample_full.jpg';
          }}
        />
        <span class="max-w-[90px] truncate text-[10px] font-medium font-mono">{steamProfile.persona_name}</span>
      </a>
    {/if}

    <!-- Handheld & Laptop Eco Mode Toggle -->
    <button
      type="button"
      onclick={() => ecoStore.toggle()}
      title={ecoStore.enabled ? 'Eco Mode Active (Battery Saver)' : 'Enable Eco Mode'}
      class="flex items-center gap-1 rounded-full border px-2 py-0.5 font-mono text-[10px] font-semibold transition-all {ecoStore.enabled
        ? 'border-emerald-500/50 bg-emerald-500/15 text-emerald-300'
        : 'border-white/10 bg-white/[0.04] text-zinc-400 hover:border-white/20 hover:text-white'}"
    >
      <span>🍃</span>
      <span>ECO</span>
      {#if ecoStore.batteryLevel !== null}
        <span class="text-[9px] opacity-75">{ecoStore.batteryLevel}%</span>
      {/if}
    </button>

    <div class="flex items-center gap-1">
    <!-- Minimize -->
    <button
      type="button"
      onclick={handleMinimize}
      class="flex h-7 w-8 items-center justify-center rounded text-zinc-400 transition-colors hover:bg-white/10 hover:text-white"
      title="Minimize"
      aria-label="Minimize"
    >
      <svg class="h-3 w-3 fill-current" viewBox="0 0 16 16">
        <path d="M2 8h12v1H2z"/>
      </svg>
    </button>

    <!-- Maximize / Restore -->
    <button
      type="button"
      onclick={handleToggleMaximize}
      class="flex h-7 w-8 items-center justify-center rounded text-zinc-400 transition-colors hover:bg-white/10 hover:text-white"
      title="Maximize"
      aria-label="Maximize"
    >
      <svg class="h-3 w-3 fill-current" viewBox="0 0 16 16">
        <path d="M3 3v10h10V3H3zm1 1h8v8H4V4z"/>
      </svg>
    </button>

    <!-- Close -->
    <button
      type="button"
      onclick={handleClose}
      class="flex h-7 w-8 items-center justify-center rounded text-zinc-400 transition-colors hover:bg-red-600/80 hover:text-white"
      title="Close"
      aria-label="Close"
    >
      <svg class="h-3 w-3 fill-current" viewBox="0 0 16 16">
        <path d="M3.72 3.72a.75.75 0 0 1 1.06 0L8 6.94l3.22-3.22a.75.75 0 1 1 1.06 1.06L9.06 8l3.22 3.22a.75.75 0 1 1-1.06 1.06L8 9.06l-3.22 3.22a.75.75 0 0 1-1.06-1.06L6.94 8 3.72 4.78a.75.75 0 0 1 0-1.06z"/>
      </svg>
    </button>
    </div>
  </div>
</header>
