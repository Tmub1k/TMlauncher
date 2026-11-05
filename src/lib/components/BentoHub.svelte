<!--
  BentoHub.svelte — OmniLauncher Master Shell.
  - Home: Live Steam Store Showcase & Sync
  - Library: Unified Catalog (Installed & Uninstalled Games across all platforms)
  - Roblox: Experience Modes Hub, 1-Click Launch, FastFlags & Multi-Instance
  - itch.io: DRM-Free Indie Store Sync
  - Deals: Deals Radar & Epic Freebies
  - Media & Friends
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import Navigation, { type NavSection } from './Navigation.svelte';
  import GameCard from './GameCard.svelte';
  import RouletteModal from './RouletteModal.svelte';
  import DealsRadar from './DealsRadar.svelte';
  import FriendsList from './FriendsList.svelte';
  import MediaGallery from './MediaGallery.svelte';
  import RobloxStudioPanel from './RobloxStudioPanel.svelte';
  import SteamStoreShowcase from './SteamStoreShowcase.svelte';
  import ItchStoreShowcase from './ItchStoreShowcase.svelte';
  import GameDetailsModal from './GameDetailsModal.svelte';
  import StoreBrowserModal from './StoreBrowserModal.svelte';
  import GamePropertiesModal from './GamePropertiesModal.svelte';
  import DirectoryPickerModal from './DirectoryPickerModal.svelte';
  import SaveVaultModal from './SaveVaultModal.svelte';
  import StorageManagerModal from './StorageManagerModal.svelte';
  import StoreView from '../views/StoreView.svelte';
  import EmulatorsView from '../views/EmulatorsView.svelte';
  import SettingsView from '../views/SettingsView.svelte';
  import DownloadsView from '../views/DownloadsView.svelte';
  import {
    fetchLibrary,
    fetchMediaGallery,
    fetchUnifiedLibrary,
    fetchRobloxRecentPlaces,
    inAppStore,
    onLibraryUpdated,
    rescanLibrary,
    type Game,
    type InAppStoreState,
    type MediaItem,
    type PlatformKind,
    type RobloxPlace,
    type UnifiedGame,
  } from '../stores/library';
  import { fetchUnifiedFriends, type UnifiedFriend } from '../stores/friends';
  import { themeStore, type WindowEffectKind } from '../stores/theme';

  // ---- Reactive State (Svelte 5 Runes) --------------------------------------
  let games = $state<Game[]>([]);
  let unifiedGames = $state<UnifiedGame[]>([]);
  let robloxRecentPlaces = $state<RobloxPlace[]>([]);
  let mediaItems = $state<MediaItem[]>([]);
  let friends = $state<UnifiedFriend[]>([]);
  let loading = $state(true);

  // Navigation & Filtering
  let activeSection = $state<NavSection>('home');
  let selectedPlatform = $state<'all' | PlatformKind>('all');
  let installedFilter = $state<'all' | 'installed' | 'uninstalled'>('all');
  let smartFilter = $state<'all' | 'installed' | 'unplayed' | 'favorites' | 'steam' | 'epic' | 'roblox' | 'emulator'>('all');
  let favoriteIds = $state<string[]>([]);
  let searchQuery = $state('');
  let sortMode = $state<'title' | 'playtime' | 'recent'>('title');
  let dedupEnabled = $state(true);
  let rouletteOpen = $state(false);
  let saveVaultOpen = $state(false);
  let saveVaultGame = $state<Game | null>(null);
  let storageManagerOpen = $state(false);
  let selectedGame = $state<Game | null>(null);
  let propertiesGame = $state<any | null>(null);
  let storeModal = $state<InAppStoreState>({ open: false, url: '', title: '' });

  // Vibrancy selector
  let vibrancy = $state<WindowEffectKind>(themeStore.vibrancy);

  // ---- Derived State --------------------------------------------------------
  const displayGames = $derived.by(() => {
    if (dedupEnabled && selectedPlatform === 'all' && unifiedGames.length > 0) {
      return unifiedGames.map((ug) => {
        const preferredSource = ug.sources.find((s) => s.platform === ug.preferred_platform) || ug.sources[0];
        return {
          id: preferredSource?.id || ug.id,
          preferred_platform: ug.preferred_platform,
          platform: ug.preferred_platform as any,
          platforms: ug.platforms,
          nativeId: preferredSource?.native_id || '',
          title: ug.primary_title,
          installed: ug.installed,
          installDir: preferredSource?.install_dir || null,
          executable: preferredSource?.executable || null,
          sizeBytes: preferredSource?.size_bytes || null,
          playtimeMinutes: ug.total_playtime_minutes,
          lastPlayed: preferredSource?.last_played || null,
          art: {
            poster: ug.poster_url
              ? {
                  kind: (/^(https?:|data:)/i.test(ug.poster_url) ? 'remote' : 'local') as 'remote' | 'local',
                  value: ug.poster_url,
                }
              : null,
            hero: null,
            logo: null,
            icon: null,
          },
          tags: [],
          sources: ug.sources,
        };
      });
    }
    return games;
  });

  const installedCount = $derived(displayGames.filter((g) => g.installed).length);
  const uninstalledCount = $derived(displayGames.filter((g) => !g.installed).length);
  const unplayedCount = $derived(
    displayGames.filter((g) => g.installed && (!g.playtimeMinutes || g.playtimeMinutes < 30)).length
  );
  const favoritesCount = $derived(
    displayGames.filter((g) => favoriteIds.includes(g.id) || (g.tags && g.tags.includes('favorite'))).length
  );
  const steamCount = $derived(
    displayGames.filter((g) => g.platform === 'steam' || (g.platforms && g.platforms.includes('steam'))).length
  );
  const epicCount = $derived(
    displayGames.filter((g) => g.platform === 'epic' || (g.platforms && g.platforms.includes('epic'))).length
  );
  const robloxCount = $derived(
    displayGames.filter((g) => g.platform === 'roblox' || (g.platforms && g.platforms.includes('roblox'))).length
  );
  const emulatorCount = $derived(
    displayGames.filter((g) => g.platform === 'emulator' || (g.platforms && g.platforms.includes('emulator'))).length
  );

  const filteredGames = $derived.by(() => {
    const q = searchQuery.trim().toLowerCase();
    const list = displayGames.filter((g) => {
      // Smart Dynamic Filter logic
      let smartMatch = true;
      if (smartFilter === 'installed') {
        smartMatch = g.installed;
      } else if (smartFilter === 'unplayed') {
        smartMatch = g.installed && (!g.playtimeMinutes || g.playtimeMinutes < 30);
      } else if (smartFilter === 'favorites') {
        smartMatch = favoriteIds.includes(g.id) || (g.tags && g.tags.includes('favorite'));
      } else if (smartFilter === 'steam') {
        smartMatch = g.platform === 'steam' || (g.platforms && g.platforms.includes('steam'));
      } else if (smartFilter === 'epic') {
        smartMatch = g.platform === 'epic' || (g.platforms && g.platforms.includes('epic'));
      } else if (smartFilter === 'roblox') {
        smartMatch = g.platform === 'roblox' || (g.platforms && g.platforms.includes('roblox'));
      } else if (smartFilter === 'emulator') {
        smartMatch = g.platform === 'emulator' || (g.platforms && g.platforms.includes('emulator'));
      }

      const platformMatch =
        selectedPlatform === 'all' || g.platform === selectedPlatform;
      const installMatch =
        installedFilter === 'all' ||
        (installedFilter === 'installed' ? g.installed : !g.installed);
      const queryMatch = !q || g.title.toLowerCase().includes(q);
      return smartMatch && platformMatch && installMatch && queryMatch;
    });

    return [...list].sort((a, b) => {
      if (sortMode === 'playtime') {
        return (Number(b.playtimeMinutes) || 0) - (Number(a.playtimeMinutes) || 0);
      }
      if (sortMode === 'recent') {
        const ta = a.lastPlayed ? Date.parse(a.lastPlayed) : 0;
        const tb = b.lastPlayed ? Date.parse(b.lastPlayed) : 0;
        return tb - ta;
      }
      return a.title.localeCompare(b.title);
    });
  });

  const onlineFriendsCount = $derived(
    friends.filter((f) => f.state === 'ingame' || f.state === 'online').length
  );

  // ---- Lifecycle ------------------------------------------------------------
  onMount(() => {
    themeStore.initialize();

    try {
      const storedFavs = localStorage.getItem('omnilauncher.favorites');
      if (storedFavs) favoriteIds = JSON.parse(storedFavs);
    } catch {
      // Ignore
    }

    let unlisten: (() => void) | undefined;

    (async () => {
      try {
        const [g, m, f, u, r] = await Promise.all([
          fetchLibrary(),
          fetchMediaGallery(),
          fetchUnifiedFriends(),
          fetchUnifiedLibrary().catch(() => []),
          fetchRobloxRecentPlaces().catch(() => []),
        ]);
        games = g;
        mediaItems = m;
        friends = f;
        unifiedGames = u;
        robloxRecentPlaces = r;
      } catch (err) {
        console.warn('Initial load:', err);
      } finally {
        loading = false;
      }

      try {
        unlisten = await onLibraryUpdated(async (report) => {
          games = report.games;
          unifiedGames = await fetchUnifiedLibrary().catch(() => []);
        });
      } catch {
        // Fallback
      }
    })();

    const unsubStore = inAppStore.subscribe((s) => {
      storeModal = s;
    });

    return () => {
      unsubStore();
      unlisten?.();
    };
  });

  // Custom / Third-Party Games
  let customGameModalOpen = $state(false);
  let customGameTitle = $state('');
  let customGamePath = $state('');
  let customGameCover = $state('');
  let addingCustomGame = $state(false);

  async function handleAddCustomGame() {
    if (!customGamePath.trim()) return;
    addingCustomGame = true;
    try {
      await invoke('add_custom_game', {
        name: customGameTitle.trim() || '',
        executablePath: customGamePath.trim(),
        launchArguments: null,
        coverUrl: customGameCover.trim() || null,
      });
      customGameModalOpen = false;
      customGameTitle = '';
      customGamePath = '';
      customGameCover = '';
      await handleRefresh();
    } catch (err: any) {
      alert(`Ошибка добавления стороннего приложения: ${err?.message || err}`);
    } finally {
      addingCustomGame = false;
    }
  }

  // ---- Actions --------------------------------------------------------------
  async function handleRefresh() {
    loading = true;
    try {
      const [report, u, r] = await Promise.all([
        rescanLibrary(),
        fetchUnifiedLibrary().catch(() => []),
        fetchRobloxRecentPlaces().catch(() => []),
      ]);
      games = report.games;
      unifiedGames = u;
      robloxRecentPlaces = r;
    } finally {
      loading = false;
    }
  }

  async function refreshMedia() {
    try {
      mediaItems = await fetchMediaGallery();
    } catch {
      // Ignore
    }
  }

  async function cycleVibrancy(next: WindowEffectKind) {
    vibrancy = next;
    await themeStore.setVibrancy(next);
  }
</script>

<div class="flex h-full min-h-0 w-full overflow-hidden text-[var(--text-primary)]">
  <!-- Primary Sidebar Navigation -->
  <Navigation
    active={activeSection}
    totalGames={games.length}
    onlineFriends={onlineFriendsCount}
    mediaCount={mediaItems.length}
    onSelect={(section) => (activeSection = section)}
    onOpenRoulette={() => (rouletteOpen = true)}
    onOpenSaveVault={() => {
      saveVaultGame = filteredGames[0] || null;
      saveVaultOpen = true;
    }}
    onOpenStorageManager={() => (storageManagerOpen = true)}
  />

  <!-- Main Content Viewport -->
  <div data-scroll-viewport class="flex h-full min-h-0 min-w-0 flex-1 flex-col overflow-y-auto overflow-x-hidden">
    <!-- Top Command Bar -->
    <header
      class="sticky top-0 z-20 flex flex-wrap items-center justify-between gap-4 border-b border-[var(--border-subtle)] bg-[#09090b]/80 px-6 py-3.5 backdrop-blur-xl"
    >
      <div class="flex items-center gap-3">
        <span
          class="rounded-lg border border-white/15 bg-white/5 px-2.5 py-1 font-mono text-[11px] uppercase tracking-widest text-zinc-200"
        >
          {#if activeSection === 'home' || activeSection === 'store'}
            STORE
          {:else if activeSection === 'library'}
            LIBRARY
          {:else if activeSection === 'emulators'}
            EMULATORS
          {:else if activeSection === 'settings'}
            SETTINGS
          {:else if activeSection === 'downloads'}
            DOWNLOADS
          {:else if activeSection === 'roblox'}
            ROBLOX HUB
          {:else if activeSection === 'itch'}
            ITCH.IO STORE
          {:else if activeSection === 'deals'}
            DEALS RADAR
          {:else if activeSection === 'friends'}
            COMMUNITY
          {:else}
            MEDIA
          {/if}
        </span>
        {#if activeSection === 'library'}
          <span class="font-mono text-xs text-zinc-400">
            {filteredGames.length} Игр ({installedCount} Установлено, {uninstalledCount} в облаке)
          </span>
        {/if}
      </div>

      <!-- Controls: Vibrancy Switcher + Save Vault + Storage + Rescan -->
      <div class="flex flex-wrap items-center gap-2.5">
        <button
          type="button"
          onclick={() => {
            saveVaultGame = filteredGames[0] || null;
            saveVaultOpen = true;
          }}
          class="flex items-center gap-1.5 rounded-lg border border-[var(--border-subtle)] bg-[#121215] px-3 py-1.5 text-xs font-medium text-zinc-200 transition-colors hover:border-[var(--border-active)] hover:bg-[#18181b]"
          title="Save-State Time Machine (Бэкапы и таймлайн сохранений)"
        >
          <span>💾</span>
          <span>Save Vault</span>
        </button>

        <button
          type="button"
          onclick={() => (storageManagerOpen = true)}
          class="flex items-center gap-1.5 rounded-lg border border-[var(--border-subtle)] bg-[#121215] px-3 py-1.5 text-xs font-medium text-zinc-200 transition-colors hover:border-[var(--border-active)] hover:bg-[#18181b]"
          title="Storage & Shader Cache Reclaimer (Очистка кэша шейдеров DirectX/Vulkan)"
        >
          <span>🧹</span>
          <span>Storage</span>
        </button>

        <select
          aria-label="Эффект окна Windows"
          value={vibrancy}
          onchange={(e) =>
            cycleVibrancy((e.currentTarget as HTMLSelectElement).value as WindowEffectKind)}
          class="rounded-lg border border-[var(--border-subtle)] bg-[#121215] px-2.5 py-1.5 font-mono text-[11px] text-zinc-300 outline-none"
        >
          <option value="mica">Mica (Windows 11)</option>
          <option value="tabbed">Mica Alt</option>
          <option value="acrylic">Acrylic</option>
          <option value="none">Тёмный стиль</option>
        </select>

        <button
          type="button"
          onclick={() => (customGameModalOpen = true)}
          class="flex items-center gap-1.5 rounded-lg border border-white/15 bg-white/10 px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:border-white/30 hover:bg-white/20"
          title="Добавить стороннюю игру или приложение в библиотеку"
        >
          <span>+</span>
          <span>Добавить игру</span>
        </button>

        <button
          type="button"
          onclick={handleRefresh}
          class="flex items-center gap-1.5 rounded-lg border border-[var(--border-subtle)] bg-[#121215] px-3 py-1.5 text-xs font-medium text-zinc-200 transition-colors hover:border-[var(--border-active)] hover:bg-[#18181b]"
        >
          <svg class="h-3.5 w-3.5 fill-current {loading ? 'animate-spin' : ''}" viewBox="0 0 24 24">
            <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
          </svg>
          <span>Пересканировать</span>
        </button>
      </div>
    </header>

    <main class="mx-auto w-full max-w-[1680px] p-6 space-y-6">
      {#if activeSection === 'home' || activeSection === 'store'}
        <!-- 1. HOME/STORE: INFINITE STEAM STORE & RECOMMENDATIONS -->
        <StoreView />
      {:else if activeSection === 'emulators'}
        <!-- 2. EMULATORS: GITHUB OPEN-SOURCE CATALOG & 1-CLICK DOWNLOADER -->
        <EmulatorsView />
      {:else if activeSection === 'downloads'}
        <DownloadsView games={displayGames} />
      {:else if activeSection === 'settings'}
        <SettingsView />
      {:else if activeSection === 'library'}
        <!-- 2. LIBRARY: UNIFIED CATALOG OF INSTALLED & UNINSTALLED GAMES -->
        <div class="space-y-6">
          <!-- Library Filter Bar: Smart Dynamic Filter Pills + Platforms + Search + Sort -->
          <div class="flex flex-col gap-4 rounded-2xl border border-white/10 bg-[#121215]/80 p-4 backdrop-blur-xl">
            <!-- Smart Dynamic Filter Pills (Installed, Unplayed Backlog, Favorites, Platforms) -->
            <div class="flex flex-wrap items-center gap-2 pb-3 border-b border-white/5">
              <span class="font-mono text-[10px] uppercase text-zinc-500 mr-1">Фильтр:</span>
              <button
                type="button"
                onclick={() => (smartFilter = 'all')}
                class="flex items-center gap-1.5 rounded-full px-3 py-1 font-mono text-xs font-semibold transition-all {smartFilter === 'all'
                  ? 'border border-white/60 bg-white text-black shadow-md'
                  : 'border border-white/10 bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
              >
                <span>Все</span>
                <span class="rounded-full bg-black/15 px-1.5 py-0.2 text-[10px]">{displayGames.length}</span>
              </button>

              <button
                type="button"
                onclick={() => (smartFilter = 'installed')}
                class="flex items-center gap-1.5 rounded-full px-3 py-1 font-mono text-xs font-semibold transition-all {smartFilter === 'installed'
                  ? 'border border-emerald-400/80 bg-emerald-500 text-black shadow-md'
                  : 'border border-white/10 bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
              >
                <span class="h-1.5 w-1.5 rounded-full bg-emerald-400"></span>
                <span>Установлено</span>
                <span class="rounded-full bg-black/15 px-1.5 py-0.2 text-[10px]">{installedCount}</span>
              </button>

              <button
                type="button"
                onclick={() => (smartFilter = 'unplayed')}
                class="flex items-center gap-1.5 rounded-full px-3 py-1 font-mono text-xs font-semibold transition-all {smartFilter === 'unplayed'
                  ? 'border border-purple-400/80 bg-purple-500 text-white shadow-md'
                  : 'border border-white/10 bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
                title="Установленные игры с временем игры менее 30 минут"
              >
                <span class="h-1.5 w-1.5 rounded-full bg-purple-400"></span>
                <span>Бэклог (&lt;30м)</span>
                <span class="rounded-full bg-white/20 px-1.5 py-0.2 text-[10px]">{unplayedCount}</span>
              </button>

              <button
                type="button"
                onclick={() => (smartFilter = 'favorites')}
                class="flex items-center gap-1.5 rounded-full px-3 py-1 font-mono text-xs font-semibold transition-all {smartFilter === 'favorites'
                  ? 'border border-amber-400/80 bg-amber-400 text-black shadow-md'
                  : 'border border-white/10 bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
              >
                <span>⭐</span>
                <span>Избранное</span>
                {#if favoritesCount > 0}
                  <span class="rounded-full bg-black/15 px-1.5 py-0.2 text-[10px]">{favoritesCount}</span>
                {/if}
              </button>

              <div class="h-4 w-px bg-white/10 mx-1"></div>

              <button
                type="button"
                onclick={() => (smartFilter = 'steam')}
                class="flex items-center gap-1.5 rounded-full px-3 py-1 font-mono text-xs font-semibold transition-all {smartFilter === 'steam'
                  ? 'border border-sky-400/80 bg-sky-500 text-white shadow-md'
                  : 'border border-white/10 bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
              >
                <span>Steam</span>
                <span class="rounded-full bg-black/15 px-1.5 py-0.2 text-[10px]">{steamCount}</span>
              </button>

              <button
                type="button"
                onclick={() => (smartFilter = 'epic')}
                class="flex items-center gap-1.5 rounded-full px-3 py-1 font-mono text-xs font-semibold transition-all {smartFilter === 'epic'
                  ? 'border border-white/60 bg-white text-black shadow-md'
                  : 'border border-white/10 bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
              >
                <span>Epic Games</span>
                <span class="rounded-full bg-black/15 px-1.5 py-0.2 text-[10px]">{epicCount}</span>
              </button>

              <button
                type="button"
                onclick={() => (smartFilter = 'roblox')}
                class="flex items-center gap-1.5 rounded-full px-3 py-1 font-mono text-xs font-semibold transition-all {smartFilter === 'roblox'
                  ? 'border border-rose-400/80 bg-rose-500 text-white shadow-md'
                  : 'border border-white/10 bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
              >
                <span>Roblox</span>
                <span class="rounded-full bg-black/15 px-1.5 py-0.2 text-[10px]">{robloxCount}</span>
              </button>

              <button
                type="button"
                onclick={() => (smartFilter = 'emulator')}
                class="flex items-center gap-1.5 rounded-full px-3 py-1 font-mono text-xs font-semibold transition-all {smartFilter === 'emulator'
                  ? 'border border-emerald-400/80 bg-emerald-600 text-white shadow-md'
                  : 'border border-white/10 bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
              >
                <span>Эмуляторы</span>
                <span class="rounded-full bg-black/15 px-1.5 py-0.2 text-[10px]">{emulatorCount}</span>
              </button>
            </div>

            <div class="flex flex-wrap items-center justify-between gap-3">
              <!-- Platform Selector Pills -->
              <div class="flex flex-wrap items-center gap-1.5">
                {#each [
                  { id: 'all', label: 'Все платформы' },
                  { id: 'steam', label: 'Steam' },
                  { id: 'epic', label: 'Epic Games' },
                  { id: 'roblox', label: 'Roblox' },
                  { id: 'itch', label: 'itch.io' },
                  { id: 'emulator', label: 'Эмуляторы' },
                ] as item}
                  <button
                    type="button"
                    onclick={() => (selectedPlatform = item.id as any)}
                    class="rounded-xl px-3 py-1.5 text-xs font-semibold transition-all {selectedPlatform ===
                    item.id
                      ? 'bg-white text-black shadow-md'
                      : 'bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
                  >
                    {item.label}
                  </button>
                {/each}
              </div>

              <!-- Search Bar -->
              <input
                type="search"
                bind:value={searchQuery}
                placeholder="Поиск по библиотеке…"
                class="w-64 rounded-xl border border-[var(--border-subtle)] bg-[#18181b] px-3.5 py-1.5 text-xs text-white placeholder-zinc-500 outline-none transition-colors focus:border-white/40"
              />
            </div>

            <div class="flex flex-wrap items-center justify-between gap-3 pt-3 border-t border-white/5">
              <!-- Installation Status Filter -->
              <div class="flex items-center gap-2">
                <span class="font-mono text-[10px] uppercase text-zinc-500">Статус:</span>
                <button
                  type="button"
                  onclick={() => (installedFilter = 'all')}
                  class="rounded-lg px-2.5 py-1 font-mono text-xs transition-colors {installedFilter ===
                  'all'
                    ? 'bg-white/20 text-white font-bold'
                    : 'text-zinc-400 hover:text-white'}"
                >
                  Все ({games.length})
                </button>
                <button
                  type="button"
                  onclick={() => (installedFilter = 'installed')}
                  class="flex items-center gap-1.5 rounded-lg px-2.5 py-1 font-mono text-xs transition-colors {installedFilter ===
                  'installed'
                    ? 'bg-emerald-500/20 text-emerald-300 font-bold border border-emerald-500/30'
                    : 'text-zinc-400 hover:text-white'}"
                >
                  <span class="h-1.5 w-1.5 rounded-full bg-emerald-400"></span>
                  Установленные ({installedCount})
                </button>
                <button
                  type="button"
                  onclick={() => (installedFilter = 'uninstalled')}
                  class="flex items-center gap-1.5 rounded-lg px-2.5 py-1 font-mono text-xs transition-colors {installedFilter ===
                  'uninstalled'
                    ? 'bg-sky-500/20 text-sky-300 font-bold border border-sky-500/30'
                    : 'text-zinc-400 hover:text-white'}"
                >
                  <span class="h-1.5 w-1.5 rounded-full bg-sky-400"></span>
                  В облаке / Не установлены ({uninstalledCount})
                </button>
              </div>

              <div class="flex flex-wrap items-center gap-3">
                <!-- Deduplication Toggle (Feature 11) -->
                <button
                  type="button"
                  onclick={() => (dedupEnabled = !dedupEnabled)}
                  class="flex items-center gap-1.5 rounded-lg border px-2.5 py-1 font-mono text-xs transition-colors {dedupEnabled
                    ? 'border-[var(--accent-chrome,#e4e4e7)] bg-white/10 text-white font-bold'
                    : 'border-white/10 text-zinc-400 hover:text-white'}"
                  title="Объединять дубликаты игр из разных магазинов в единую карточку"
                >
                  <span>Дедупликация:</span>
                  <span class={dedupEnabled ? 'text-emerald-400 font-bold' : 'text-zinc-500'}>
                    {dedupEnabled ? 'ВКЛ' : 'ВЫКЛ'}
                  </span>
                </button>

                <!-- Sort Dropdown -->
                <div class="flex items-center gap-2">
                  <span class="font-mono text-[10px] uppercase text-zinc-500">Сортировка:</span>
                  <select
                    aria-label="Сортировка игр"
                    bind:value={sortMode}
                    class="rounded-lg border border-[var(--border-subtle)] bg-[#18181b] px-3 py-1 font-mono text-xs text-zinc-200 outline-none"
                  >
                    <option value="title">По названию (А–Я)</option>
                    <option value="playtime">По времени в игре</option>
                    <option value="recent">По недавнему запуску</option>
                  </select>
                </div>
              </div>
            </div>
          </div>

          <!-- Non-stretched Auto-fill Grid for Game Cards -->
          {#if filteredGames.length === 0 && !loading}
            <div
              class="flex flex-col items-center justify-center rounded-2xl border border-white/5 bg-white/[0.02] py-20 text-center text-zinc-400"
            >
              <p class="text-sm font-semibold text-zinc-200">Игры не найдены</p>
              <p class="mt-1 text-xs text-zinc-500">
                Измените фильтр платформы или статуса установки, либо запустите сканирование.
              </p>
            </div>
          {:else}
            <!-- Fixed minmax card width prevents stretching across large monitors -->
            <div
              class="grid grid-cols-[repeat(auto-fill,minmax(190px,1fr))] gap-5 justify-items-center sm:justify-items-start"
            >
              {#each filteredGames as game (game.id)}
                <GameCard
                  {game}
                  onSelect={(g) => (selectedGame = g)}
                  onProperties={(g) => (propertiesGame = g)}
                />
              {/each}
            </div>
          {/if}
        </div>
      {:else if activeSection === 'roblox'}
        <!-- 3. ROBLOX RECENT EXPERIENCES + MODES & STUDIO -->
        <div class="space-y-6">
          {#if robloxRecentPlaces.length > 0}
            <div class="rounded-2xl border border-white/10 bg-[#121215]/80 p-5 backdrop-blur-xl">
              <div class="flex items-center justify-between mb-4">
                <div>
                  <h3 class="text-sm font-bold text-white tracking-tight">Недавно сыгранные плейсы (Roblox)</h3>
                  <p class="text-xs text-zinc-400">История сессий из клиентских логов с возможностью прямого запуска</p>
                </div>
                <span class="rounded-full bg-white/10 px-2.5 py-0.5 font-mono text-[10px] text-zinc-300">
                  {robloxRecentPlaces.length} плейсов
                </span>
              </div>

              <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-3.5">
                {#each robloxRecentPlaces as place}
                  <div class="group relative overflow-hidden rounded-xl border border-white/10 bg-black/40 p-2.5 transition-all hover:border-white/30 hover:-translate-y-1">
                    <div class="aspect-square w-full overflow-hidden rounded-lg bg-zinc-900 mb-2">
                      {#if place.thumbnail_url}
                        <img src={place.thumbnail_url} alt={place.name} class="h-full w-full object-cover transition-transform group-hover:scale-105" />
                      {:else}
                        <div class="flex h-full w-full items-center justify-center text-zinc-600 font-mono text-xs">RBLX</div>
                      {/if}
                    </div>
                    <h4 class="truncate text-xs font-semibold text-white group-hover:text-amber-300 transition-colors" title={place.name}>
                      {place.name}
                    </h4>
                    <p class="text-[10px] text-zinc-500 font-mono mt-0.5">ID: {place.place_id}</p>
                    {#if place.launch_url}
                      <a
                        href={place.launch_url}
                        class="mt-2 block w-full rounded-lg bg-white/10 py-1 text-center font-mono text-[10px] font-bold text-white hover:bg-white/20 transition-colors"
                      >
                        PLAY
                      </a>
                    {/if}
                  </div>
                {/each}
              </div>
            </div>
          {/if}

          <RobloxStudioPanel />
        </div>
      {:else if activeSection === 'itch'}
        <!-- 4. ITCH.IO INDIE STORE -->
        <ItchStoreShowcase />
      {:else if activeSection === 'deals'}
        <!-- 5. DEALS RADAR -->
        <DealsRadar />
      {:else if activeSection === 'friends'}
        <!-- 6. FRIENDS -->
        <FriendsList />
      {:else if activeSection === 'media'}
        <!-- 7. MEDIA -->
        <MediaGallery items={mediaItems} onRefresh={refreshMedia} />
      {/if}
    </main>
  </div>
</div>

<!-- Backlog Roulette Modal -->
<RouletteModal open={rouletteOpen} onClose={() => (rouletteOpen = false)} />

<!-- Save-State Time Machine Modal -->
<SaveVaultModal
  game={saveVaultGame || filteredGames[0] || null}
  open={saveVaultOpen}
  onClose={() => {
    saveVaultOpen = false;
    saveVaultGame = null;
  }}
/>

<!-- Storage & Shader Cache Reclaimer Modal -->
<StorageManagerModal
  open={storageManagerOpen}
  onClose={() => (storageManagerOpen = false)}
/>

<!-- Game Details Modal (3D Casual GUI Style) -->
<GameDetailsModal game={selectedGame} onClose={() => (selectedGame = null)} />

<!-- Steam-Grade 5-Tab Game Properties Modal -->
<GamePropertiesModal
  game={propertiesGame}
  open={!!propertiesGame}
  onClose={() => (propertiesGame = null)}
/>

<!-- In-App Store Browser Modal (Same Window) -->
<StoreBrowserModal
  open={storeModal.open}
  url={storeModal.url}
  title={storeModal.title}
  installUri={storeModal.installUri}
  onClose={() => inAppStore.close()}
/>

<!-- In-Launcher Drive & Directory Selector Modal -->
<DirectoryPickerModal />

<!-- Add Custom / Non-Steam Game Modal -->
{#if customGameModalOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 p-4 backdrop-blur-md"
    role="dialog"
    aria-modal="true"
  >
    <div class="chrome-card relative w-full max-w-lg rounded-2xl border border-white/15 bg-[#121215] p-6 shadow-2xl">
      <div class="flex items-center justify-between pb-4 border-b border-white/10">
        <h3 class="text-base font-bold text-white flex items-center gap-2">
          <span>+</span>
          <span>Добавить стороннее приложение</span>
        </h3>
        <button
          type="button"
          onclick={() => (customGameModalOpen = false)}
          class="text-zinc-400 hover:text-white"
        >
          ✕
        </button>
      </div>

      <div class="space-y-4 py-4 text-xs">
        <div>
          <label class="block font-medium text-zinc-300 mb-1">
            Путь к исполняемому файлу (.exe / .lnk / .bat) <span class="text-red-400">*</span>
          </label>
          <input
            type="text"
            bind:value={customGamePath}
            placeholder="C:\Games\MyGame\game.exe"
            class="w-full rounded-xl border border-white/10 bg-black/60 px-3.5 py-2 font-mono text-xs text-white placeholder-zinc-500 outline-none focus:border-white/30"
          />
        </div>

        <div>
          <label class="block font-medium text-zinc-300 mb-1">
            Название игры (необязательно)
          </label>
          <input
            type="text"
            bind:value={customGameTitle}
            placeholder="Будет определено из файла если пусто"
            class="w-full rounded-xl border border-white/10 bg-black/60 px-3.5 py-2 text-xs text-white placeholder-zinc-500 outline-none focus:border-white/30"
          />
        </div>

        <div>
          <label class="block font-medium text-zinc-300 mb-1">
            URL обложки / постера (необязательно)
          </label>
          <input
            type="text"
            bind:value={customGameCover}
            placeholder="https://... или путь к картинке"
            class="w-full rounded-xl border border-white/10 bg-black/60 px-3.5 py-2 text-xs text-white placeholder-zinc-500 outline-none focus:border-white/30"
          />
        </div>
      </div>

      <div class="flex items-center justify-end gap-2 pt-4 border-t border-white/10">
        <button
          type="button"
          onclick={() => (customGameModalOpen = false)}
          class="btn-minimal px-4 py-2 text-xs"
        >
          Отмена
        </button>
        <button
          type="button"
          onclick={handleAddCustomGame}
          disabled={!customGamePath.trim() || addingCustomGame}
          class="btn-chrome px-5 py-2 text-xs font-bold bg-white text-black hover:bg-zinc-200 disabled:opacity-40"
        >
          {addingCustomGame ? 'Добавление...' : 'Добавить в библиотеку'}
        </button>
      </div>
    </div>
  </div>
{/if}
