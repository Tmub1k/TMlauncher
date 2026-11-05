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
    openInAppStore,
    type SteamStoreItem,
    type SteamStoreShowcase as ShowcaseData,
  } from '../stores/library';

  type StoreTab = 'specials' | 'topSellers' | 'newReleases' | 'comingSoon';

  let showcase = $state<ShowcaseData | null>(null);
  let activeTab = $state<StoreTab>('specials');
  let searchQuery = $state('');
  let searchResults = $state<SteamStoreItem[]>([]);
  let searching = $state(false);
  let loading = $state(true);
  let actionBusy = $state<number | null>(null);
  let notification = $state<string | null>(null);

  let searchTimer: ReturnType<typeof setTimeout> | null = null;

  async function loadShowcase() {
    loading = true;
    try {
      showcase = await fetchSteamStoreShowcase();
    } catch (err) {
      console.error('Failed to load Steam store showcase:', err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadShowcase();
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
        searchResults = await invoke<SteamStoreItem[]>('search_steam_store', { query: q });
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

  function handleOpenStore(item: SteamStoreItem) {
    openInAppStore(item.storeUrl, `${item.name} • Steam Store`, item.installUri);
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
        <div
          class="group relative flex flex-col justify-between overflow-hidden rounded-2xl border border-white/10 bg-[#121215] transition-all duration-300 hover:-translate-y-1 hover:border-white/30 hover:shadow-2xl"
        >
          <!-- Game Banner -->
          <div class="relative aspect-[16/9] w-full overflow-hidden bg-black/40">
            {#if item.largeCapsuleImage || item.headerImage}
              <img
                src={item.largeCapsuleImage ?? item.headerImage}
                alt={item.name}
                loading="lazy"
                decoding="async"
                class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
              />
            {:else}
              <div class="flex h-full w-full items-center justify-center font-mono text-xs text-zinc-500">
                Steam Game
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
              <h3 class="line-clamp-1 text-sm font-bold tracking-tight text-white group-hover:text-sky-300 transition-colors">
                {item.name}
              </h3>

              <div class="mt-2 flex items-baseline gap-2 font-mono text-xs">
                {#if item.formattedFinalPrice}
                  <span class="text-sm font-bold text-white">{item.formattedFinalPrice}</span>
                {/if}
                {#if item.formattedOriginalPrice && item.discounted}
                  <span class="text-zinc-500 line-through text-[11px]">{item.formattedOriginalPrice}</span>
                {/if}
              </div>
            </div>

            <!-- Action Buttons: Open in Steam Store or Direct Download -->
            <div class="mt-4 grid grid-cols-2 gap-2 pt-2 border-t border-white/5">
              <button
                type="button"
                onclick={() => handleOpenStore(item)}
                class="btn-game btn-game-dark py-2 text-xs"
              >
                В магазин ↗
              </button>

              <button
                type="button"
                onclick={() => handleInstall(item)}
                disabled={actionBusy === item.id}
                class="btn-game btn-game-blue py-2 text-xs"
              >
                <svg class="h-3.5 w-3.5 fill-current" viewBox="0 0 24 24">
                  <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
                </svg>
                <span>{actionBusy === item.id ? 'Запуск…' : 'Скачать'}</span>
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
