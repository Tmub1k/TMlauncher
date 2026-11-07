<!--
  StoreView.svelte — Flagship Steam Store Infinite Recommendations & USD/KZT Currency Switcher.
  TURA & Liquid Chrome aesthetic:
  - Minimalist matte graphite canvas (#09090b)
  - Real-time USD ($) / KZT (₸) currency toggle with zero reload
  - Genre filter chips & debounced search
  - Featured Spotlight Hero with video trailer preview
  - Infinite scroll feed with IntersectionObserver & debounce lock
  - Catalog exhaustion banner ("You have caught up with all current recommendations")
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { currencyStore, storeFeed, type StoreItem } from '$lib/stores/store';
  import { inAppStore } from '$lib/stores/library';
  import { storeLinkManager, type StoreOpenMode } from '$lib/utils/storeLink';

  let scrollRoot: HTMLElement | null = null;
  let sentinelElement: HTMLDivElement | null = $state(null);
  let hoveredCardKey = $state<string | null>(null);
  let videoFailMap = $state<Record<string, boolean>>({});
  let imageFallbackAttempts = $state<Record<number, number>>({});
  let searchInput = $state('');
  let searchTimeout: any = null;

  const GENRE_TAGS = [
    'All',
    'FPS',
    'Action',
    'RPG',
    'Roguelike',
    'Strategy',
    'Indie',
    'Simulation',
    'Sci-Fi',
    'Open World',
    'Horror',
  ];

  function getSteamImage(appId: number, initialUrl?: string): string | null {
    const attempt = imageFallbackAttempts[appId] || 0;
    if (attempt === 0 && initialUrl) return initialUrl;
    if (attempt <= 1) {
      return `https://shared.cloudflare.steamstatic.com/store_item_assets/steam/apps/${appId}/header.jpg`;
    }
    if (attempt <= 2) {
      return `https://cdn.akamai.steamstatic.com/steam/apps/${appId}/capsule_616x353.jpg`;
    }
    return null;
  }

  onMount(() => {
    // Initial feed fetch
    if (storeFeed.items.length === 0) {
      storeFeed.loadPage(0, false);
    }

    scrollRoot = sentinelElement?.closest('[data-scroll-viewport]') as HTMLElement | null;

    // Set up IntersectionObserver for infinite scrolling
    const observer = new IntersectionObserver(
      (entries) => {
        const first = entries[0];
        if (first && first.isIntersecting) {
          if (!storeFeed.inFlight && storeFeed.hasMore && !storeFeed.loading) {
            storeFeed.loadNext();
          }
        }
      },
      { root: scrollRoot, rootMargin: '300px' }
    );

    if (sentinelElement) {
      observer.observe(sentinelElement);
    }

    return () => {
      observer.disconnect();
      if (searchTimeout) clearTimeout(searchTimeout);
    };
  });

  function handleSearchInput(e: Event) {
    const target = e.target as HTMLInputElement;
    searchInput = target.value;
    if (searchTimeout) clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      storeFeed.setSearch(searchInput);
    }, 300);
  }

  function handleCurrencyToggle(code: 'USD' | 'KZT') {
    currencyStore.setCurrency(code);
    // Reload feed with new currency prices
    storeFeed.loadPage(0, false);
  }

  function handleOpenStore(item: StoreItem, modeOverride?: StoreOpenMode) {
    storeLinkManager.open({
      url: item.store_url,
      title: item.title,
      appId: item.app_id,
      platform: 'steam',
      installUri: item.install_uri,
      mode: modeOverride,
    });
  }

  const featuredItem = $derived.by(() => {
    return storeFeed.items.length > 0 ? storeFeed.items[0] : null;
  });
</script>

<div class="space-y-6 pb-12 text-[var(--text-primary)]">
  <!-- Top Command Bar: Title, Search, Store Open Mode, Currency Switcher -->
  <div class="flex flex-wrap items-center justify-between gap-4 rounded-2xl border border-[var(--border-subtle)] bg-[var(--bg-surface-base)]/80 p-5 backdrop-blur-xl">
    <div class="flex items-center gap-4">
      <div>
        <h2 class="text-lg font-black tracking-tight text-[var(--text-primary)] flex items-center gap-2">
          STORE
          <span class="rounded-full bg-[var(--accent-primary)]/20 border border-[var(--accent-primary)]/30 px-2.5 py-0.5 font-mono text-[10px] uppercase tracking-widest text-[var(--accent-primary)] font-bold">
            Infinite Feed
          </span>
        </h2>
        <p class="text-xs text-[var(--text-secondary)]">Рекомендации Steam, скидки и предпросмотр трейлеров</p>
      </div>
    </div>

    <div class="flex flex-wrap items-center gap-3">
      <!-- Search Input -->
      <div class="relative w-64">
        <input
          type="search"
          value={searchInput}
          oninput={handleSearchInput}
          placeholder="Поиск по магазину Steam..."
          class="w-full rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-canvas)]/60 px-3.5 py-1.5 pl-9 text-xs text-[var(--text-primary)] placeholder-zinc-500 outline-none transition-colors focus:border-[var(--accent-primary)]"
        />
        <svg
          class="pointer-events-none absolute left-3 top-2.5 h-3.5 w-3.5 text-zinc-400"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
      </div>

      <!-- Store Opening Mode Pill -->
      <div class="flex items-center rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-canvas)]/60 p-1 shadow-inner">
        <button
          type="button"
          onclick={() => storeLinkManager.setMode('client')}
          class="flex items-center gap-1 rounded-lg px-2.5 py-1 font-mono text-[11px] font-bold transition-all {storeLinkManager.mode === 'client'
            ? 'bg-[var(--accent-primary)] text-white shadow-md'
            : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
          title="Открывать в клиенте Steam (с вашей авторизацией)"
        >
          🎮 В клиенте
        </button>
        <button
          type="button"
          onclick={() => storeLinkManager.setMode('launcher')}
          class="flex items-center gap-1 rounded-lg px-2.5 py-1 font-mono text-[11px] font-bold transition-all {storeLinkManager.mode === 'launcher'
            ? 'bg-[var(--accent-primary)] text-white shadow-md'
            : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
          title="Открывать во встроенном окне лаунчера"
        >
          ❐ Окно
        </button>
        <button
          type="button"
          onclick={() => storeLinkManager.setMode('browser')}
          class="flex items-center gap-1 rounded-lg px-2.5 py-1 font-mono text-[11px] font-bold transition-all {storeLinkManager.mode === 'browser'
            ? 'bg-[var(--accent-primary)] text-white shadow-md'
            : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
          title="Открывать в стандартном браузере Windows"
        >
          🌐 Браузер
        </button>
      </div>

      <!-- Currency Switcher Pill ($ USD / ₸ KZT) -->
      <div class="flex items-center rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-canvas)]/60 p-1 shadow-inner">
        <button
          type="button"
          onclick={() => handleCurrencyToggle('USD')}
          class="flex items-center gap-1 rounded-lg px-2.5 py-1 font-mono text-xs font-bold transition-all {currencyStore.code === 'USD'
            ? 'bg-[var(--accent-primary)] text-white shadow-md'
            : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
        >
          $ USD
        </button>
        <button
          type="button"
          onclick={() => handleCurrencyToggle('KZT')}
          class="flex items-center gap-1 rounded-lg px-2.5 py-1 font-mono text-xs font-bold transition-all {currencyStore.code === 'KZT'
            ? 'bg-[var(--accent-primary)] text-white shadow-md'
            : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
        >
          ₸ KZT
        </button>
      </div>

      {#if currencyStore.isStale}
        <span
          class="flex items-center gap-1 rounded-md border border-amber-500/30 bg-amber-500/10 px-2 py-1 font-mono text-[10px] text-amber-300"
          title="Offline > 7 days: using cached rate (1 USD = 495 KZT)"
        >
          ⚠ Stale Rate
        </span>
      {/if}
    </div>
  </div>

  <!-- Genre Tag Filter Chips -->
  <div class="flex items-center gap-1.5 overflow-x-auto pb-1 scrollbar-thin">
    {#each GENRE_TAGS as tag}
      <button
        type="button"
        onclick={() => storeFeed.setTag(tag)}
        class="shrink-0 rounded-xl px-3 py-1.5 text-xs font-semibold transition-all {storeFeed.selectedTag === tag
          ? 'bg-[var(--accent-primary)] text-white shadow-md'
          : 'border border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] text-[var(--text-secondary)] hover:text-[var(--text-primary)]'}"
      >
        {tag}
      </button>
    {/each}
  </div>

  <!-- Featured Spotlight Hero Banner (when available) -->
  {#if featuredItem && !storeFeed.searchQuery && storeFeed.selectedTag === 'All'}
    <div
      class="group relative overflow-hidden rounded-2xl border border-[var(--border-subtle)] bg-gradient-to-br from-[var(--bg-surface-elevated)] via-[var(--bg-surface-base)] to-[var(--bg-canvas)] shadow-2xl transition-all"
    >
      <div class="flex flex-col lg:flex-row">
        <!-- Media Preview Area (Video or High-Res Header) -->
        <div class="relative aspect-video w-full lg:w-3/5 overflow-hidden bg-black">
          {#key featuredItem.app_id}
            {#if featuredItem.media?.mp4Video && !videoFailMap[`spotlight_${featuredItem.app_id}`]}
              <video
                src={featuredItem.media.mp4Video}
                poster={getSteamImage(featuredItem.app_id, featuredItem.header_image) || featuredItem.header_image}
                autoplay
                loop
                muted
                playsinline
                onerror={() => (videoFailMap[`spotlight_${featuredItem.app_id}`] = true)}
                class="h-full w-full object-cover transition-transform duration-700 group-hover:scale-105"
              ></video>
            {:else}
              {@const spotlightImg = getSteamImage(featuredItem.app_id, featuredItem.header_image)}
              {#if spotlightImg}
                <img
                  src={spotlightImg}
                  alt={featuredItem.title}
                  onerror={() => {
                    imageFallbackAttempts[featuredItem.app_id] = (imageFallbackAttempts[featuredItem.app_id] || 0) + 1;
                  }}
                  class="h-full w-full object-cover transition-transform duration-700 group-hover:scale-105"
                />
              {/if}
            {/if}
          {/key}
          <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent lg:hidden"></div>
        </div>

        <!-- Meta & Actions Panel -->
        <div class="flex flex-1 flex-col justify-between p-6 lg:p-8">
          <div class="space-y-3">
            <div class="flex items-center gap-2">
              <span class="rounded-full bg-amber-400/20 border border-amber-400/30 px-2.5 py-0.5 font-mono text-[10px] font-bold text-amber-300">
                SPOTLIGHT
              </span>
              {#if featuredItem.review_score}
                <span class="font-mono text-xs text-emerald-400">
                  ★ {featuredItem.review_score}
                </span>
              {/if}
            </div>

            <h3 class="text-2xl font-black tracking-tight text-[var(--text-primary)]">
              {featuredItem.title}
            </h3>

            <p class="text-xs text-[var(--text-secondary)] line-clamp-3 leading-relaxed">
              {featuredItem.short_description || 'Оцените этот шедевр в магазине Steam.'}
            </p>

            <div class="flex flex-wrap gap-1.5 pt-1">
              {#each featuredItem.tags.slice(0, 4) as tag}
                <span class="rounded-md border border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] px-2 py-0.5 text-[10px] font-medium text-[var(--text-secondary)]">
                  {tag}
                </span>
              {/each}
            </div>
          </div>

          <!-- Price & Install Bar -->
          <div class="mt-6 flex flex-wrap items-center justify-between gap-4 border-t border-[var(--border-subtle)] pt-4">
            <div class="flex items-center gap-2.5">
              {#if featuredItem.discount_percent > 0}
                <span class="rounded-lg bg-emerald-500/20 border border-emerald-500/40 px-2.5 py-1 font-mono text-xs font-black text-emerald-400">
                  -{featuredItem.discount_percent}%
                </span>
                <span class="font-mono text-xs text-[var(--text-muted)] line-through">
                  {featuredItem.original_price_formatted}
                </span>
              {/if}
              <span class="font-mono text-lg font-black text-[var(--text-primary)]">
                {featuredItem.price_formatted}
              </span>
            </div>

            <div class="flex items-center gap-2.5">
              <button
                type="button"
                onclick={() => handleOpenStore(featuredItem, 'client')}
                class="rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] px-3 py-2 font-mono text-xs font-bold text-[var(--text-primary)] hover:border-[var(--accent-primary)] transition-all flex items-center gap-1.5"
                title="Открыть в приложении Steam"
              >
                <span>🎮</span> В Steam
              </button>
              <a
                href={featuredItem.install_uri}
                class="rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] px-4 py-2 font-mono text-xs font-bold text-[var(--text-primary)] hover:border-[var(--accent-primary)] transition-all shadow-sm"
              >
                INSTALL
              </a>
              <button
                type="button"
                onclick={() => handleOpenStore(featuredItem)}
                class="btn-chrome px-5 py-2 text-xs font-bold"
              >
                DETAILS
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- Recommendations Infinite Grid -->
  <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-3 gap-4">
    {#each storeFeed.items as item, index (`${item.app_id}_${index}`)}
      {@const cardKey = `${item.app_id}_${index}`}
      {@const cardImg = getSteamImage(item.app_id, item.header_image)}
      <div
        role="article"
        class="group relative flex flex-col justify-between overflow-hidden rounded-2xl border border-[var(--border-subtle)] bg-[var(--bg-surface-base)] transition-all duration-300 hover:border-[var(--border-active)] hover:-translate-y-1 hover:shadow-2xl"
        onmouseenter={() => (hoveredCardKey = cardKey)}
        onmouseleave={() => {
          if (hoveredCardKey === cardKey) hoveredCardKey = null;
        }}
      >
        <!-- Card Artwork / Trailer -->
        <div class="relative aspect-video w-full overflow-hidden bg-black/60">
          {#if hoveredCardKey === cardKey && item.media?.mp4Video && !videoFailMap[cardKey]}
            {#key item.media.mp4Video}
              <video
                src={item.media.mp4Video}
                poster={cardImg || item.header_image}
                autoplay
                loop
                muted
                playsinline
                onerror={() => (videoFailMap[cardKey] = true)}
                class="h-full w-full object-cover"
              ></video>
            {/key}
          {:else if cardImg}
            <img
              src={cardImg}
              alt={item.title}
              loading="lazy"
              onerror={() => {
                imageFallbackAttempts[item.app_id] = (imageFallbackAttempts[item.app_id] || 0) + 1;
              }}
              class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
            />
          {:else}
            <!-- High quality fallback cover -->
            <div class="flex h-full w-full flex-col items-center justify-center bg-gradient-to-br from-[var(--bg-surface-elevated)] to-[var(--bg-canvas)] p-4 text-center">
              <span class="text-xs font-mono font-bold text-[var(--accent-primary)] mb-1">STEAM #{item.app_id}</span>
              <span class="text-xs font-bold text-[var(--text-primary)] line-clamp-2">{item.title}</span>
            </div>
          {/if}

          <!-- Discount Badge Overlay -->
          {#if item.discount_percent === 100}
            <div class="absolute top-2 left-2 rounded-lg bg-emerald-500 px-2 py-0.5 font-mono text-[10px] font-black text-black shadow-md">
              БЕСПЛАТНО (-100%)
            </div>
          {:else if item.discount_percent > 0}
            <div class="absolute top-2 left-2 rounded-lg bg-emerald-500/90 px-2 py-0.5 font-mono text-[10px] font-black text-white shadow-md">
              -{item.discount_percent}%
            </div>
          {/if}
        </div>

        <!-- Card Content Body -->
        <div class="flex flex-1 flex-col justify-between p-4 space-y-3">
          <div class="space-y-1.5">
            <h4 class="font-bold text-sm text-[var(--text-primary)] group-hover:text-[var(--accent-primary)] transition-colors line-clamp-1" title={item.title}>
              {item.title}
            </h4>

            {#if item.review_score}
              <p class="font-mono text-[11px] text-[var(--text-secondary)]">
                ★ {item.review_score}
              </p>
            {/if}

            <div class="flex flex-wrap gap-1 pt-1">
              {#each item.tags.slice(0, 3) as tag}
                <span class="rounded bg-white/5 border border-white/5 px-1.5 py-0.5 font-mono text-[9px] text-[var(--text-secondary)]">
                  {tag}
                </span>
              {/each}
            </div>
          </div>

          <!-- Bottom Price & Actions Bar -->
          <div class="flex items-center justify-between border-t border-[var(--border-subtle)] pt-3">
            <div class="flex flex-col">
              {#if item.discount_percent > 0 && item.original_price_formatted}
                <span class="font-mono text-[10px] text-[var(--text-muted)] line-through">
                  {item.original_price_formatted}
                </span>
              {/if}
              <span class="font-mono text-xs font-black {item.price_raw === 0 ? 'text-emerald-400' : 'text-[var(--text-primary)]'}">
                {item.price_formatted}
              </span>
            </div>

            <div class="flex items-center gap-1.5">
              <!-- Quick 1-click open in Steam client -->
              <button
                type="button"
                onclick={() => handleOpenStore(item, 'client')}
                class="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] p-1 text-[var(--text-secondary)] hover:text-[var(--accent-primary)] hover:border-[var(--accent-primary)] transition-colors"
                title="Открыть прямо в приложении Steam"
              >
                <svg class="h-3.5 w-3.5 fill-current" viewBox="0 0 24 24"><path d="M12 2C6.47 2 2 6.47 2 12c0 4.19 2.58 7.78 6.24 9.25l2.42-3.48A4.954 4.954 0 0 1 10 15c0-.62.11-1.21.32-1.76L8.43 11.8A7.03 7.03 0 0 1 12 5c3.87 0 7 3.13 7 7 0 3.2-2.15 5.9-5.11 6.74l1.17 1.68C18.9 19.16 22 15.93 22 12c0-5.53-4.47-10-10-10zm0 5a5 5 0 0 0-4.9 4.02l2.49 1.74A2.99 2.99 0 0 1 12 11c1.66 0 3 1.34 3 3 0 .76-.28 1.45-.75 1.98l2.4 1.68A5.002 5.002 0 0 0 12 7z"/></svg>
              </button>
              <a
                href={item.install_uri}
                class="rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-surface-elevated)] px-2 py-1 font-mono text-[10px] font-bold text-[var(--text-primary)] hover:border-[var(--accent-primary)] transition-colors"
                title="Direct Steam Install"
              >
                INSTALL
              </a>
              <button
                type="button"
                onclick={() => handleOpenStore(item)}
                class="btn-chrome px-2.5 py-1 text-[10px] font-bold"
              >
                DETAILS
              </button>
            </div>
          </div>
        </div>
      </div>
    {/each}
  </div>

  <!-- Loading Spinner -->
  {#if storeFeed.loading}
    <div class="flex justify-center py-8">
      <div class="h-7 w-7 animate-spin rounded-full border-2 border-[var(--border-subtle)] border-t-[var(--accent-primary)]"></div>
    </div>
  {/if}

  <!-- Catalog Exhaustion Banner -->
  {#if !storeFeed.hasMore && storeFeed.endMessage}
    <div class="flex flex-col items-center justify-center rounded-2xl border border-[var(--border-subtle)] bg-[var(--bg-surface-base)]/50 p-8 text-center">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-[var(--accent-primary)]/20 text-[var(--accent-primary)] mb-2">
        ✓
      </div>
      <p class="font-semibold text-sm text-[var(--text-primary)]">
        {storeFeed.endMessage}
      </p>
      <p class="text-xs text-[var(--text-secondary)] mt-1">Возвращайтесь завтра за обновленными рекомендациями</p>
    </div>
  {/if}

  <!-- Infinite Scroll Sentinel -->
  <div bind:this={sentinelElement} class="h-4 w-full"></div>
</div>
