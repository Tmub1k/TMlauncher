<!--
  ItchStoreShowcase.svelte — Live itch.io Store synchronization.
  Browses popular DRM-free indie games, on-sale titles, and free releases.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    fetchItchStoreShowcase,
    installGameById,
    openInAppStore,
    type ItchStoreGame,
  } from '../stores/library';

  type ItchCategory = 'top-sellers' | 'on-sale' | 'new-and-popular' | 'free';

  let games = $state<ItchStoreGame[]>([]);
  let activeCategory = $state<ItchCategory>('top-sellers');
  let searchQuery = $state('');
  let loading = $state(true);
  let actionBusy = $state<string | null>(null);

  async function loadFeed() {
    loading = true;
    try {
      games = await fetchItchStoreShowcase(activeCategory);
    } catch (err) {
      console.error('Failed to load itch store feed:', err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadFeed();
  });

  function changeCategory(cat: ItchCategory) {
    activeCategory = cat;
    loadFeed();
  }

  const filteredGames = $derived.by<ItchStoreGame[]>(() => {
    const q = searchQuery.trim().toLowerCase();
    if (!q) return games;
    return games.filter(
      (g) => g.title.toLowerCase().includes(q) || g.description.toLowerCase().includes(q)
    );
  });

  async function handleDownload(game: ItchStoreGame) {
    actionBusy = game.id;
    try {
      await installGameById(`itch:${game.id}`);
    } catch {
      window.open(game.url, '_blank');
    } finally {
      setTimeout(() => {
        actionBusy = null;
      }, 1000);
    }
  }
</script>

<div class="space-y-6">
  <!-- Status Banner -->
  <div
    class="flex flex-wrap items-center justify-between gap-4 rounded-2xl border border-white/10 bg-gradient-to-r from-rose-950/40 via-[#121215] to-[#18181b] p-5 backdrop-blur-xl shadow-xl"
  >
    <div class="flex items-center gap-4">
      <div
        class="flex h-12 w-12 items-center justify-center rounded-2xl border border-rose-400/30 bg-rose-500/10 text-rose-300 shadow-inner font-black text-sm"
      >
        itch
      </div>
      <div>
        <div class="flex items-center gap-2">
          <span class="h-2 w-2 rounded-full bg-rose-400 animate-pulse"></span>
          <h2 class="text-base font-bold tracking-tight text-white">Магазин itch.io • DRM-Free Инди-витрина</h2>
          <span class="rounded bg-rose-500/20 px-2 py-0.5 font-mono text-[10px] text-rose-300 border border-rose-500/30">
            SYNC
          </span>
        </div>
        <p class="text-xs text-zinc-400 mt-0.5">
          Прямая синхронизация с каталогом независимых разработчиков, геймджемов и эксклюзивных релизов.
        </p>
      </div>
    </div>

    <button
      type="button"
      onclick={loadFeed}
      disabled={loading}
      class="flex items-center gap-2 rounded-xl border border-white/15 bg-white/5 px-4 py-2 text-xs font-semibold text-white transition-all hover:bg-white hover:text-black disabled:opacity-40"
    >
      <svg
        class="h-3.5 w-3.5 fill-current {loading ? 'animate-spin' : ''}"
        viewBox="0 0 24 24"
      >
        <path
          d="M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z"
        />
      </svg>
      <span>{loading ? 'Загрузка…' : 'Обновить каталог'}</span>
    </button>
  </div>

  <!-- Category Pills & Search -->
  <div class="flex flex-wrap items-center justify-between gap-3 border-b border-[var(--border-subtle)] pb-3">
    <div class="flex flex-wrap items-center gap-2">
      <button
        type="button"
        onclick={() => changeCategory('top-sellers')}
        class="rounded-xl px-3.5 py-2 text-xs font-semibold transition-all {activeCategory ===
        'top-sellers'
          ? 'bg-white text-black shadow-md'
          : 'bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
      >
        🏆 Лидеры продаж
      </button>

      <button
        type="button"
        onclick={() => changeCategory('on-sale')}
        class="rounded-xl px-3.5 py-2 text-xs font-semibold transition-all {activeCategory ===
        'on-sale'
          ? 'bg-white text-black shadow-md'
          : 'bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
      >
        🏷️ Распродажи
      </button>

      <button
        type="button"
        onclick={() => changeCategory('new-and-popular')}
        class="rounded-xl px-3.5 py-2 text-xs font-semibold transition-all {activeCategory ===
        'new-and-popular'
          ? 'bg-white text-black shadow-md'
          : 'bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
      >
        ✨ Популярные новинки
      </button>

      <button
        type="button"
        onclick={() => changeCategory('free')}
        class="rounded-xl px-3.5 py-2 text-xs font-semibold transition-all {activeCategory ===
        'free'
          ? 'bg-white text-black shadow-md'
          : 'bg-white/5 text-zinc-400 hover:text-white hover:bg-white/10'}"
      >
        🎁 Бесплатные игры
      </button>
    </div>

    <input
      type="search"
      bind:value={searchQuery}
      placeholder="Поиск по itch.io…"
      class="w-56 rounded-xl border border-[var(--border-subtle)] bg-[#121215] px-3.5 py-1.5 text-xs text-white placeholder-zinc-500 outline-none transition-colors focus:border-white/40"
    />
  </div>

  <!-- Cards Grid -->
  {#if loading}
    <div class="flex flex-col items-center justify-center py-24 text-zinc-400">
      <div class="h-8 w-8 animate-spin rounded-full border-2 border-white/20 border-t-white"></div>
      <p class="mt-4 text-xs font-medium">Синхронизация с серверами itch.io…</p>
    </div>
  {:else if filteredGames.length === 0}
    <div class="flex flex-col items-center justify-center rounded-2xl border border-white/5 bg-white/[0.02] py-20 text-zinc-400">
      <p class="text-sm font-semibold text-zinc-300">Игры не найдены</p>
      <p class="mt-1 text-xs text-zinc-500">Попробуйте другую категорию или измените поисковый фильтр.</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4">
      {#each filteredGames as game (game.id)}
        <div
          class="group relative flex flex-col justify-between overflow-hidden rounded-2xl border border-white/10 bg-[#121215] transition-all duration-300 hover:-translate-y-1 hover:border-white/30 hover:shadow-2xl"
        >
          <div class="relative aspect-[16/10] w-full overflow-hidden bg-black/40">
            {#if game.coverUrl}
              <img
                src={game.coverUrl}
                alt={game.title}
                loading="lazy"
                decoding="async"
                class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
              />
            {:else}
              <div class="flex h-full w-full items-center justify-center font-mono text-xs text-zinc-500">
                itch.io Game
              </div>
            {/if}

            <div
              class="absolute right-2.5 top-2.5 rounded-lg bg-black/75 px-2 py-0.5 font-mono text-xs font-bold text-white border border-white/20 backdrop-blur-md"
            >
              {game.price}
            </div>
          </div>

          <div class="flex flex-1 flex-col justify-between p-4">
            <div>
              <h3 class="line-clamp-1 text-sm font-bold tracking-tight text-white group-hover:text-rose-300 transition-colors">
                {game.title}
              </h3>
              {#if game.description}
                <p class="mt-1 line-clamp-2 text-xs text-zinc-400">
                  {game.description}
                </p>
              {/if}

              <div class="mt-2.5 flex flex-wrap gap-1">
                {#each game.platforms as p}
                  <span class="rounded bg-white/5 border border-white/10 px-1.5 py-0.5 font-mono text-[9px] text-zinc-400">
                    {p}
                  </span>
                {/each}
              </div>
            </div>

            <div class="mt-4 grid grid-cols-2 gap-2 pt-2 border-t border-white/5">
              <button
                type="button"
                onclick={() => openInAppStore(game.url, `${game.title} • itch.io`)}
                class="btn-game btn-game-dark py-2 text-xs"
              >
                В магазин ↗
              </button>

              <button
                type="button"
                onclick={() => handleDownload(game)}
                disabled={actionBusy === game.id}
                class="btn-game btn-game-red py-2 text-xs"
              >
                <svg class="h-3.5 w-3.5 fill-current" viewBox="0 0 24 24">
                  <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
                </svg>
                <span>Скачать</span>
              </button>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
