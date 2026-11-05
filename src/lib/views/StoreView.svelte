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

  let scrollRoot: HTMLElement | null = null;
  let sentinelElement: HTMLDivElement | null = $state(null);
  let hoveredCardKey = $state<string | null>(null);
  let videoFailMap = $state<Record<string, boolean>>({});
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

  function handleOpenStore(item: StoreItem) {
    inAppStore.open(item.store_url, item.title, item.install_uri);
  }

  const featuredItem = $derived.by(() => {
    return storeFeed.items.length > 0 ? storeFeed.items[0] : null;
  });
</script>

<div class="space-y-6 pb-12">
  <!-- Top Command Bar: Title, Search, Currency Switcher -->
  <div class="flex flex-wrap items-center justify-between gap-4 rounded-2xl border border-white/10 bg-[#121215]/80 p-5 backdrop-blur-xl">
    <div class="flex items-center gap-4">
      <div>
        <h2 class="text-lg font-black tracking-tight text-white flex items-center gap-2">
          STORE
          <span class="rounded-full bg-white/10 px-2 py-0.5 font-mono text-[10px] uppercase tracking-widest text-zinc-300">
            Infinite Feed
          </span>
        </h2>
        <p class="text-xs text-zinc-400">Curated recommendations, deals, and video previews</p>
      </div>
    </div>

    <div class="flex flex-wrap items-center gap-3">
      <!-- Search Input -->
      <div class="relative w-64">
        <input
          type="search"
          value={searchInput}
          oninput={handleSearchInput}
          placeholder="Search Steam Store..."
          class="w-full rounded-xl border border-white/10 bg-black/50 px-3.5 py-1.5 pl-9 text-xs text-white placeholder-zinc-500 outline-none transition-colors focus:border-white/30"
        />
        <svg
          class="pointer-events-none absolute left-3 top-2.5 h-3.5 w-3.5 text-zinc-500"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
        </svg>
      </div>

      <!-- Currency Switcher Pill ($ USD / ₸ KZT) -->
      <div class="flex items-center rounded-xl border border-white/15 bg-black/60 p-1 shadow-inner">
        <button
          type="button"
          onclick={() => handleCurrencyToggle('USD')}
          class="flex items-center gap-1 rounded-lg px-2.5 py-1 font-mono text-xs font-bold transition-all {currencyStore.code === 'USD'
            ? 'bg-white text-black shadow-md'
            : 'text-zinc-400 hover:text-white'}"
        >
          $ USD
        </button>
        <button
          type="button"
          onclick={() => handleCurrencyToggle('KZT')}
          class="flex items-center gap-1 rounded-lg px-2.5 py-1 font-mono text-xs font-bold transition-all {currencyStore.code === 'KZT'
            ? 'bg-white text-black shadow-md'
            : 'text-zinc-400 hover:text-white'}"
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
          ? 'bg-white text-black shadow-md'
          : 'bg-white/5 text-zinc-400 hover:bg-white/10 hover:text-white'}"
      >
        {tag}
      </button>
    {/each}
  </div>

  <!-- Featured Spotlight Hero Banner (when available) -->
  {#if featuredItem && !storeFeed.searchQuery && storeFeed.selectedTag === 'All'}
    <div
      class="group relative overflow-hidden rounded-2xl border border-white/15 bg-gradient-to-br from-[#18181b] via-[#121215] to-[#09090b] shadow-2xl transition-all"
    >
      <div class="flex flex-col lg:flex-row">
        <!-- Media Preview Area (Video or High-Res Header) -->
        <div class="relative aspect-video w-full lg:w-3/5 overflow-hidden bg-black">
          {#key featuredItem.app_id}
            {#if featuredItem.media?.mp4Video && !videoFailMap[`spotlight_${featuredItem.app_id}`]}
              <video
                src={featuredItem.media.mp4Video}
                poster={featuredItem.header_image}
                autoplay
                loop
                muted
                playsinline
                onerror={() => (videoFailMap[`spotlight_${featuredItem.app_id}`] = true)}
                class="h-full w-full object-cover transition-transform duration-700 group-hover:scale-105"
              ></video>
            {:else if featuredItem.header_image}
              <img
                src={featuredItem.header_image}
                alt={featuredItem.title}
                class="h-full w-full object-cover transition-transform duration-700 group-hover:scale-105"
              />
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

            <h3 class="text-2xl font-black tracking-tight text-white">
              {featuredItem.title}
            </h3>

            <p class="text-xs text-zinc-300 line-clamp-3 leading-relaxed">
              {featuredItem.short_description || 'Experience this critically acclaimed masterpiece on Steam.'}
            </p>

            <div class="flex flex-wrap gap-1.5 pt-1">
              {#each featuredItem.tags.slice(0, 4) as tag}
                <span class="rounded-md bg-white/10 px-2 py-0.5 text-[10px] font-medium text-zinc-300">
                  {tag}
                </span>
              {/each}
            </div>
          </div>

          <!-- Price & Install Bar -->
          <div class="mt-6 flex flex-wrap items-center justify-between gap-4 border-t border-white/10 pt-4">
            <div class="flex items-center gap-2.5">
              {#if featuredItem.discount_percent > 0}
                <span class="rounded-lg bg-emerald-500/20 border border-emerald-500/40 px-2.5 py-1 font-mono text-xs font-black text-emerald-300">
                  -{featuredItem.discount_percent}%
                </span>
                <span class="font-mono text-xs text-zinc-500 line-through">
                  {featuredItem.original_price_formatted}
                </span>
              {/if}
              <span class="font-mono text-lg font-black text-white">
                {featuredItem.price_formatted}
              </span>
            </div>

            <div class="flex items-center gap-2.5">
              <a
                href={featuredItem.install_uri}
                class="rounded-xl border border-white/20 bg-white/10 px-4 py-2 font-mono text-xs font-bold text-white hover:bg-white/20 transition-all shadow-sm"
              >
                INSTALL
              </a>
              <button
                type="button"
                onclick={() => handleOpenStore(featuredItem)}
                class="rounded-xl bg-white px-5 py-2 text-xs font-bold text-black shadow-lg hover:bg-zinc-200 transition-all"
              >
                STORE PAGE
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
      <div
        role="article"
        class="group relative flex flex-col justify-between overflow-hidden rounded-2xl border border-white/10 bg-[#121215]/90 transition-all duration-300 hover:border-white/30 hover:-translate-y-1 hover:shadow-2xl"
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
                poster={item.header_image}
                autoplay
                loop
                muted
                playsinline
                onerror={() => (videoFailMap[cardKey] = true)}
                class="h-full w-full object-cover"
              ></video>
            {/key}
          {:else if item.header_image}
            <img
              src={item.header_image}
              alt={item.title}
              loading="lazy"
              class="h-full w-full object-cover transition-transform duration-500 group-hover:scale-105"
            />
          {/if}

          <!-- Discount Badge Overlay -->
          {#if item.discount_percent === 100}
            <div class="absolute top-2 left-2 rounded-lg bg-emerald-500 px-2 py-0.5 font-mono text-[10px] font-black text-black shadow-md">
              FREE PROMOTION (-100%)
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
            <h4 class="font-bold text-sm text-white group-hover:text-amber-300 transition-colors line-clamp-1" title={item.title}>
              {item.title}
            </h4>

            {#if item.review_score}
              <p class="font-mono text-[11px] text-zinc-400">
                ★ {item.review_score}
              </p>
            {/if}

            <div class="flex flex-wrap gap-1 pt-1">
              {#each item.tags.slice(0, 3) as tag}
                <span class="rounded bg-white/5 px-1.5 py-0.5 font-mono text-[9px] text-zinc-400">
                  {tag}
                </span>
              {/each}
            </div>
          </div>

          <!-- Bottom Price & Actions Bar -->
          <div class="flex items-center justify-between border-t border-white/5 pt-3">
            <div class="flex flex-col">
              {#if item.discount_percent > 0 && item.original_price_formatted}
                <span class="font-mono text-[10px] text-zinc-500 line-through">
                  {item.original_price_formatted}
                </span>
              {/if}
              <span class="font-mono text-xs font-black {item.price_raw === 0 ? 'text-emerald-400' : 'text-white'}">
                {item.price_formatted}
              </span>
            </div>

            <div class="flex items-center gap-1.5">
              <a
                href={item.install_uri}
                class="rounded-lg border border-white/10 bg-white/5 px-2 py-1 font-mono text-[10px] font-bold text-zinc-200 hover:bg-white/20 transition-colors"
                title="Direct Steam Install"
              >
                INSTALL
              </a>
              <button
                type="button"
                onclick={() => handleOpenStore(item)}
                class="rounded-lg bg-white px-2.5 py-1 text-[10px] font-bold text-black hover:bg-zinc-200 transition-colors"
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
      <div class="h-7 w-7 animate-spin rounded-full border-2 border-white/20 border-t-white"></div>
    </div>
  {/if}

  <!-- Catalog Exhaustion Banner -->
  {#if !storeFeed.hasMore && storeFeed.endMessage}
    <div class="flex flex-col items-center justify-center rounded-2xl border border-white/10 bg-white/[0.02] p-8 text-center">
      <div class="flex h-10 w-10 items-center justify-center rounded-full bg-white/10 text-zinc-400 mb-2">
        ✓
      </div>
      <p class="font-semibold text-sm text-white">
        {storeFeed.endMessage}
      </p>
      <p class="text-xs text-zinc-500 mt-1">Check back tomorrow for refreshed daily picks and specials</p>
    </div>
  {/if}

  <!-- Infinite Scroll Sentinel -->
  <div bind:this={sentinelElement} class="h-4 w-full"></div>
</div>
