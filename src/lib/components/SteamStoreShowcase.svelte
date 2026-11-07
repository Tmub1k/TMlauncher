<!--
  SteamStoreShowcase.svelte — Live Steam Store synchronization on the Home page.
  Fetches live Steam specials, top sellers, new releases, and coming soon titles.
  Allows immediate installation/downloading via `steam://install/<id>` or opening in store.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import {
    fetchSteamStoreShowcase,
    installGameById,
    type SteamStoreItem,
    type SteamStoreShowcase as ShowcaseData,
  } from '../stores/library';
  import { currencyStore } from '$lib/stores/store';
  import { storeLinkManager, type StoreOpenMode } from '$lib/utils/storeLink';

  type StoreTab = 'specials' | 'topSellers' | 'newReleases' | 'comingSoon';

  let showcase = $state<ShowcaseData | null>(null);
  let activeTab = $state<StoreTab>('specials');
  let searchQuery = $state('');
  let searchResults = $state<SteamStoreItem[]>([]);
  let searching = $state(false);
  let loading = $state(true);
  let actionBusy = $state<number | null>(null);
  let notification = $state<string | null>(null);
  let imageFailAttempts = $state<Record<number, number>>({});
  let lastCurrency = $state<'USD' | 'KZT'>(currencyStore.code);

  let searchTimer: ReturnType<typeof setTimeout> | null = null;

  function getShowcaseImage(item: SteamStoreItem): string | null {
    const attempts = imageFailAttempts[item.id] || 0;
    const initial = item.largeCapsuleImage ?? item.headerImage;
    if (attempts === 0 && initial) return initial;
    if (attempts <= 1) {
      return `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${item.id}/header.jpg`;
    }
    if (attempts <= 2) {
      return `https://cdn.akamai.steamstatic.com/steam/apps/${item.id}/capsule_616x353.jpg`;
    }
    return null;
  }

  async function loadShowcase(curr = currencyStore.code) {
    loading = true;
    try {
      showcase = await fetchSteamStoreShowcase(curr);
    } catch (err) {
      console.error('Failed to load Steam store showcase:', err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadShowcase();
  });

  // Re-fetch when currency changes
  $effect(() => {
    const currentCode = currencyStore.code;
    if (currentCode !== lastCurrency) {
      lastCurrency = currentCode;
      loadShowcase(currentCode);
    }
  });

  $effect(() => {
    const q = searchQuery.trim();
    if (searchTimer) clearTimeout(searchTimer);
    if (!q) {
      searchResults = [];
      searching = false;
      return;
    }
    searching = true;
    searchTimer = setTimeout(async () => {
      try {
        searchResults = await invoke<SteamStoreItem[]>('search_steam_store', {
          query: q,
          currency: currencyStore.code,
        });
      } catch (err) {
        console.error('Failed to search entire Steam store:', err);
      } finally {
        searching = false;
      }
    }, 350);
  });

  const currentItems = $derived.by<SteamStoreItem[]>(() => {
    if (searchQuery.trim().length > 0) {
      return searchResults;
    }
    if (!showcase) return [];
    if (activeTab === 'specials') return showcase.specials;
    if (activeTab === 'topSellers') return showcase.topSellers;
    if (activeTab === 'newReleases') return showcase.newReleases;
    if (activeTab === 'comingSoon') return showcase.comingSoon;
    return [];
  });

  async function handleInstall(item: SteamStoreItem) {
    if (actionBusy === item.id) return;
    actionBusy = item.id;
    notification = `Начинается скачивание ${item.name} в Steam…`;
    try {
      await installGameById(`steam:${item.id}`);
    } catch (err) {
      console.error('Install failed:', err);
      window.open(item.installUri, '_blank');
    } finally {
      setTimeout(() => {
        actionBusy = null;
      }, 1500);
      setTimeout(() => {
        notification = null;
      }, 3500);
    }
  }

  function handleOpenStore(item: SteamStoreItem, modeOverride?: StoreOpenMode) {
    storeLinkManager.open({
      url: item.storeUrl,
      title: `${item.name} • Steam Store`,
      appId: item.id,
      platform: 'steam',
      installUri: item.installUri,
      mode: modeOverride,
    });
  }
</script>

<div class="space-y-6">
  <!-- Store Tabs & Search Across ENTIRE Steam Store -->
  <div class="flex flex-wrap items-center justify-between gap-3 border-b border-[var(--border-subtle)] pb-3">
    <div class="flex flex-wrap items-center gap-2">
      <button
        type="button"
        onclick={() => {
          activeTab = 'specials';
          searchQuery = '';
        }}
        class="flex items-center gap-1.5 rounded-xl px-3.5 py-2 text-xs font-semibold transition-all {activeTab ===
        'specials' && !searchQuery
          ? 'bg-white text-black shadow-md'
          : 'bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
      >
        <span>🔥 Скидки и акции</span>
        {#if showcase?.specials}
          <span class="rounded px-1.5 font-mono text-[10px] {activeTab === 'specials' && !searchQuery ? 'bg-black/15 text-black' : 'bg-white/10 text-zinc-300'}">
            {showcase.specials.length}
          </span>
        {/if}
      </button>

      <button
        type="button"
        onclick={() => {
          activeTab = 'topSellers';
          searchQuery = '';
        }}
        class="flex items-center gap-1.5 rounded-xl px-3.5 py-2 text-xs font-semibold transition-all {activeTab ===
        'topSellers' && !searchQuery
          ? 'bg-white text-black shadow-md'
          : 'bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
      >
        <span>🏆 Лидеры продаж</span>
        {#if showcase?.topSellers}
          <span class="rounded px-1.5 font-mono text-[10px] {activeTab === 'topSellers' && !searchQuery ? 'bg-black/15 text-black' : 'bg-white/10 text-zinc-300'}">
            {showcase.topSellers.length}
          </span>
        {/if}
      </button>

      <button
        type="button"
        onclick={() => {
          activeTab = 'newReleases';
          searchQuery = '';
        }}
        class="flex items-center gap-1.5 rounded-xl px-3.5 py-2 text-xs font-semibold transition-all {activeTab ===
        'newReleases' && !searchQuery
          ? 'bg-white text-black shadow-md'
          : 'bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
      >
        <span>✨ Новинки</span>
      </button>

      <button
        type="button"
        onclick={() => {
          activeTab = 'comingSoon';
          searchQuery = '';
        }}
        class="flex items-center gap-1.5 rounded-xl px-3.5 py-2 text-xs font-semibold transition-all {activeTab ===
        'comingSoon' && !searchQuery
          ? 'bg-white text-black shadow-md'
          : 'bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
      >
        <span>⏳ Скоро выйдут</span>
      </button>
    </div>

    <!-- Live Search across ALL of Steam Store -->
    <div class="flex items-center gap-2.5">
      <div class="relative">
        <input
          type="search"
          bind:value={searchQuery}
          placeholder="Поиск по всему магазину Steam…"
          class="w-72 rounded-xl border border-[var(--border-subtle)] bg-[#121215] px-3.5 py-1.5 text-xs text-white placeholder-zinc-500 outline-none transition-colors focus:border-white/40"
        />
        {#if searching}
          <div class="absolute right-2.5 top-2 h-3.5 w-3.5 animate-spin rounded-full border border-sky-400 border-t-transparent"></div>
        {/if}
      </div>

      <button
        type="button"
        onclick={loadShowcase}
        disabled={loading}
        class="flex items-center gap-1.5 rounded-xl border border-white/10 bg-white/5 px-3 py-1.5 text-xs font-medium text-zinc-300 hover:bg-white hover:text-black transition-colors"
        title="Обновить витрину"
      >
        <svg class="h-3.5 w-3.5 fill-current {loading ? 'animate-spin' : ''}" viewBox="0 0 24 24">
          <path d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"/>
        </svg>
      </button>
    </div>
  </div>

  <!-- Showcase Cards Grid -->
  {#if loading && !showcase}
    <div class="flex flex-col items-center justify-center py-24 text-zinc-400">
      <div class="h-8 w-8 animate-spin rounded-full border-2 border-white/20 border-t-white"></div>
      <p class="mt-4 text-xs font-medium">Синхронизация с официальными серверами Steam…</p>
    </div>
  {:else if currentItems.length === 0}
    <div class="flex flex-col items-center justify-center rounded-2xl border border-white/5 bg-white/[0.02] py-20 text-zinc-400">
      <p class="text-sm font-semibold text-zinc-300">Ничего не найдено</p>
      <p class="mt-1 text-xs text-zinc-500">Попробуйте изменить поисковый запрос или обновить категорию.</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
      {#each currentItems as item (item.id)}
        {@const cardImg = getShowcaseImage(item)}
        <div
          class="group relative flex flex-col justify-between overflow-hidden rounded-2xl border border-[var(--border-subtle)] bg-[var(--bg-surface-base)] transition-all duration-300 hover:-translate-y-1 hover:border-[var(--border-active)] hover:shadow-2xl"
        >
          <!-- Game Banner -->
          <div class="relative aspect-[16/9] w-full overflow-hidden bg-black/40">
            {#if cardImg}
              <img
                src={cardImg}
                alt={item.name}
                loading="lazy"
                decoding="async"
                onerror={() => {
                  imageFailAttempts[item.id] = (imageFailAttempts[item.id] || 0) + 1;
                }}
                class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
              />
            {:else}
              <div class="flex h-full w-full flex-col items-center justify-center bg-gradient-to-br from-[var(--bg-surface-elevated)] to-[var(--bg-canvas)] p-3 font-mono text-xs text-[var(--accent-primary)]">
                <span>STEAM #{item.id}</span>
                <span class="text-[10px] text-[var(--text-primary)] text-center line-clamp-1 mt-1">{item.name}</span>
              </div>
            {/if}

            <!-- Discount Pill -->
            {#if item.discounted && item.discountPercent > 0}
              <div
                class="absolute right-2.5 top-2.5 rounded-lg bg-emerald-500 px-2 py-0.5 font-mono text-xs font-black text-black shadow-md"
              >
                -{item.discountPercent}%
              </div>
            {/if}

            <span
              class="absolute left-2.5 top-2.5 rounded-md border border-black/40 bg-black/60 px-2 py-0.5 font-mono text-[9px] uppercase tracking-wider text-zinc-200 backdrop-blur-md"
            >
              APP #{item.id}
            </span>
          </div>

          <!-- Metadata & Details -->
          <div class="flex flex-1 flex-col justify-between p-4">
            <div>
              <h3 class="line-clamp-1 text-sm font-bold tracking-tight text-[var(--text-primary)] group-hover:text-[var(--accent-primary)] transition-colors">
                {item.name}
              </h3>

              <div class="mt-2 flex items-baseline gap-2 font-mono text-xs">
                {#if item.formattedFinalPrice}
                  <span class="text-sm font-bold text-[var(--text-primary)]">{item.formattedFinalPrice}</span>
                {/if}
                {#if item.formattedOriginalPrice && item.discounted}
                  <span class="text-[var(--text-muted)] line-through text-[11px]">{item.formattedOriginalPrice}</span>
                {/if}
              </div>
            </div>

            <!-- Action Buttons: Open in Steam Client / Launcher or Direct Download -->
            <div class="mt-4 flex items-center gap-1.5 pt-2 border-t border-[var(--border-hairline)]">
              <button
                type="button"
                onclick={() => handleOpenStore(item, 'client')}
                class="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] p-2 text-[var(--text-secondary)] hover:text-[var(--accent-primary)] hover:border-[var(--accent-primary)] transition-colors"
                title="Открыть в приложении Steam"
              >
                <span class="font-bold text-xs">🎮</span>
              </button>

              <button
                type="button"
                onclick={() => handleOpenStore(item)}
                class="btn-game btn-game-dark flex-1 py-1.5 text-xs"
              >
                В магазин ↗
              </button>

              <button
                type="button"
                onclick={() => handleInstall(item)}
                disabled={actionBusy === item.id}
                class="btn-game btn-game-blue px-3 py-1.5 text-xs"
                title="Скачать игру"
              >
                <svg class="h-3.5 w-3.5 fill-current" viewBox="0 0 24 24">
                  <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
