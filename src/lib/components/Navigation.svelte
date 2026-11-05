<!--
  Navigation.svelte — Minimalist navigation rail for OmniLauncher (Svelte 5 Runes).
  TURA & Liquid Chrome aesthetic:
  - 5 Flagship Views: Library, Store, Emulators, Downloads, Settings
  - Ecosystem views: Roblox, itch.io, Deals Radar, Media, Friends
  - Sleek monochrome typography, zero redundant Russian helper labels
-->
<script lang="ts">
  export type NavSection =
    | 'library'
    | 'home'
    | 'store'
    | 'emulators'
    | 'downloads'
    | 'settings'
    | 'roblox'
    | 'itch'
    | 'deals'
    | 'media'
    | 'friends';

  interface Props {
    active: NavSection | string;
    totalGames?: number;
    onlineFriends?: number;
    mediaCount?: number;
    onSelect: (section: NavSection) => void;
    onOpenRoulette?: () => void;
    onOpenSaveVault?: () => void;
    onOpenStorageManager?: () => void;
  }

  let {
    active,
    totalGames = 0,
    onlineFriends = 0,
    mediaCount = 0,
    onSelect,
    onOpenRoulette,
    onOpenSaveVault,
    onOpenStorageManager,
  }: Props = $props();

  // Flagship Core Navigation Items
  const NAV_PRIMARY: {
    id: NavSection;
    label: string;
    badge?: () => string | number;
    icon: string;
  }[] = [
    {
      id: 'library',
      label: 'Library',
      badge: () => totalGames,
      icon: 'M4 6a2 2 0 0 1 2-2h12a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6zm2 0v12h12V6H6zm2 2h8v2H8V8zm0 4h8v2H8v-2z',
    },
    {
      id: 'home',
      label: 'Store',
      badge: () => 'HOT',
      icon: 'M3 3h2l.4 2M7 13h10l4-8H5.4M7 13L5.4 5M7 13l-2.293 2.293c-.63.63-.184 1.707.707 1.707H17m0 0a2 2 0 1 0 0 4 2 2 0 0 0 0-4zm-8 2a2 2 0 1 1-4 0 2 2 0 0 1 4 0z',
    },
    {
      id: 'emulators',
      label: 'Emulators',
      badge: () => 'RETRO',
      icon: 'M6 4h12a2 2 0 0 1 2 2v8a4 4 0 0 1-4 4H8a4 4 0 0 1-4-4V6a2 2 0 0 1 2-2zm2 7V9H6v2H4v2h2v2h2v-2h2v-2H8zm8-2a1 1 0 1 0 0 2 1 1 0 0 0 0-2zm2 3a1 1 0 1 0 0 2 1 1 0 0 0 0-2z',
    },
    {
      id: 'downloads',
      label: 'Downloads',
      icon: 'M12 3v10m0 0-4-4m4 4 4-4M4 17h16',
    },
    {
      id: 'settings',
      label: 'Settings',
      icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 0 0 2.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 0 0 1.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 0 0-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 0 0-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 0 0-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 0 0-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 0 0 1.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0z',
    },
  ];

  // Secondary Ecosystem Items
  const NAV_SECONDARY: {
    id: NavSection;
    label: string;
    badge?: () => string | number;
  }[] = [
    { id: 'roblox', label: 'Roblox', badge: () => 'MODES' },
    { id: 'itch', label: 'itch.io', badge: () => 'INDIE' },
    { id: 'deals', label: 'Deals Radar', badge: () => 'PROMO' },
    { id: 'media', label: 'Media Vault', badge: () => mediaCount },
    { id: 'friends', label: 'Community', badge: () => onlineFriends },
  ];
</script>

<aside
  class="sticky top-0 flex h-full w-56 shrink-0 flex-col justify-between border-r border-[var(--border-hairline)] bg-[var(--bg-canvas)]/90 p-3 backdrop-blur-2xl z-20 overflow-y-auto"
>
  <div class="space-y-4 pt-1">
    <!-- Flagship Primary Tabs -->
    <div class="space-y-1">
      {#each NAV_PRIMARY as item}
        <button
          type="button"
          onclick={() => onSelect(item.id)}
          class="group flex w-full items-center justify-between rounded-lg px-2.5 py-2 text-left text-xs font-medium transition-all {active ===
          item.id
            ? 'bg-white text-black font-semibold shadow-sm'
            : 'text-zinc-400 hover:bg-white/[0.06] hover:text-white'}"
        >
          <div class="flex items-center gap-2.5">
            <svg
              class="h-4 w-4 shrink-0 transition-colors {active === item.id
                ? 'fill-current stroke-current'
                : 'fill-none stroke-current opacity-70 group-hover:opacity-100'}"
              viewBox="0 0 24 24"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d={item.icon} />
            </svg>
            <span class="tracking-wide">{item.label}</span>
          </div>

          {#if item.badge}
            <span
              class="rounded-md px-1.5 py-0.5 font-mono text-[9px] font-bold tracking-wider {active ===
              item.id
                ? 'bg-black/15 text-black'
                : 'bg-white/10 text-zinc-300'}"
            >
              {item.badge()}
            </span>
          {/if}
        </button>
      {/each}
    </div>

    <!-- Secondary Ecosystem Rail -->
    <div class="space-y-1 border-t border-[var(--border-hairline)] pt-3">
      {#each NAV_SECONDARY as item}
        <button
          type="button"
          onclick={() => onSelect(item.id)}
          class="flex w-full items-center justify-between rounded-lg px-2.5 py-1.5 text-left text-[11px] font-medium transition-all {active ===
          item.id
            ? 'bg-white text-black font-semibold shadow-sm'
            : 'text-zinc-500 hover:bg-white/[0.04] hover:text-zinc-200'}"
        >
          <span>{item.label}</span>
          {#if item.badge && String(item.badge()) !== '0'}
            <span
              class="rounded px-1.5 py-0.5 font-mono text-[8px] {active === item.id
                ? 'bg-black/15 text-black font-semibold'
                : 'bg-white/5 text-zinc-400'}"
            >
              {item.badge()}
            </span>
          {/if}
        </button>
      {/each}
    </div>
  </div>

  <!-- Bottom Quick Triggers (Roulette, Save Vault, Shader Reclaimer) -->
  <div class="border-t border-[var(--border-hairline)] pt-3 space-y-2">
    <button
      type="button"
      onclick={onOpenRoulette}
      class="group flex w-full items-center justify-between rounded-lg border border-white/10 bg-white/[0.03] px-3 py-2 text-xs font-semibold text-zinc-300 transition-all hover:border-white/30 hover:bg-white hover:text-black"
    >
      <span class="tracking-wide">Backlog</span>
      <span
        class="flex items-center gap-1 font-mono text-[10px] uppercase text-zinc-400 group-hover:text-black"
      >
        <span>Spin</span>
        <span>🎲</span>
      </span>
    </button>

    <div class="grid grid-cols-2 gap-1.5">
      <button
        type="button"
        onclick={onOpenSaveVault}
        class="flex items-center justify-center gap-1 rounded-lg border border-white/10 bg-white/[0.02] px-2 py-1.5 font-mono text-[10px] text-zinc-400 transition-all hover:border-white/20 hover:bg-white/5 hover:text-white"
        title="Save-State Time Machine (Бэкапы сохранений)"
      >
        <span>💾</span>
        <span>Vault</span>
      </button>

      <button
        type="button"
        onclick={onOpenStorageManager}
        class="flex items-center justify-center gap-1 rounded-lg border border-white/10 bg-white/[0.02] px-2 py-1.5 font-mono text-[10px] text-zinc-400 transition-all hover:border-white/20 hover:bg-white/5 hover:text-white"
        title="Shader & Storage Reclaimer (Очистка кэша)"
      >
        <span>🧹</span>
        <span>Storage</span>
      </button>
    </div>
  </div>
</aside>
