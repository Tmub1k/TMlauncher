<!--
  EmulatorsView.svelte — GitHub Open-Source Emulators Catalog & 1-Click Downloader/Extractor.
  TURA & Liquid Chrome aesthetic:
  - 8 Premier Emulators (RetroArch, Dolphin, PCSX2, RPCS3, PPSSPP, DuckStation, Ryujinx, Lime3DS)
  - System filters (All, Nintendo, Sony, Multi-Core Retro)
  - Real-time download progress bar (percentage + speed MB/s)
  - In-launcher directory selection
  - 1-Click Install & direct Launch actions
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { emulatorStore, type EmulatorCatalogEntry } from '$lib/stores/emulators';
  import { directoryPicker } from '$lib/stores/filesystem';

  let installingId = $state<string | null>(null);
  let statusMessage = $state<{ text: string; type: 'success' | 'error' | 'info' } | null>(null);

  onMount(() => {
    emulatorStore.init();
  });

  async function handleInstall(emu: EmulatorCatalogEntry, useCustomPath = false) {
    let customDir: string | undefined;

    if (useCustomPath) {
      const selected = await directoryPicker.open(
        `Select Install Folder for ${emu.name}`,
        undefined,
        500 * 1024 * 1024
      );
      if (!selected) return; // cancelled
      customDir = selected;
    }

    installingId = emu.id;
    statusMessage = { text: `Starting download for ${emu.name}...`, type: 'info' };

    try {
      const res = await emulatorStore.install(emu.id, customDir);
      if (res && res.installed) {
        statusMessage = {
          text: `Successfully installed ${emu.name}! Added to Library Hub.`,
          type: 'success',
        };
      }
    } catch (err: any) {
      statusMessage = {
        text: `Installation failed: ${err?.message || String(err)}`,
        type: 'error',
      };
    } finally {
      installingId = null;
    }
  }

  async function handleLaunch(emu: EmulatorCatalogEntry) {
    const success = await emulatorStore.launch(emu.id);
    if (!success) {
      statusMessage = {
        text: `Could not launch ${emu.name}. Executable missing or blocked.`,
        type: 'error',
      };
    }
  }
</script>

<div class="space-y-6 pb-12">
  <!-- Top Command Bar: Title, System Filters, Search -->
  <div class="flex flex-wrap items-center justify-between gap-4 rounded-2xl border border-white/10 bg-[#121215]/80 p-5 backdrop-blur-xl">
    <div>
      <h2 class="text-lg font-black tracking-tight text-white flex items-center gap-2">
        EMULATORS
        <span class="rounded-full bg-white/10 px-2 py-0.5 font-mono text-[10px] uppercase tracking-widest text-zinc-300">
          GitHub Catalog
        </span>
      </h2>
      <p class="text-xs text-zinc-400">Curated open-source emulators with 1-click download & library sync</p>
    </div>

    <div class="flex flex-wrap items-center gap-3">
      <!-- Search Bar -->
      <div class="relative w-64">
        <input
          type="search"
          bind:value={emulatorStore.searchQuery}
          placeholder="Filter emulators..."
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

      <!-- System Filters -->
      <div class="flex items-center rounded-xl border border-white/15 bg-black/60 p-1">
        <button
          type="button"
          onclick={() => (emulatorStore.activeFilter = 'all')}
          class="rounded-lg px-2.5 py-1 font-mono text-xs font-semibold transition-all {emulatorStore.activeFilter === 'all'
            ? 'bg-white text-black shadow-md'
            : 'text-zinc-400 hover:text-white'}"
        >
          All
        </button>
        <button
          type="button"
          onclick={() => (emulatorStore.activeFilter = 'nintendo')}
          class="rounded-lg px-2.5 py-1 font-mono text-xs font-semibold transition-all {emulatorStore.activeFilter === 'nintendo'
            ? 'bg-white text-black shadow-md'
            : 'text-zinc-400 hover:text-white'}"
        >
          Nintendo
        </button>
        <button
          type="button"
          onclick={() => (emulatorStore.activeFilter = 'sony')}
          class="rounded-lg px-2.5 py-1 font-mono text-xs font-semibold transition-all {emulatorStore.activeFilter === 'sony'
            ? 'bg-white text-black shadow-md'
            : 'text-zinc-400 hover:text-white'}"
        >
          Sony
        </button>
        <button
          type="button"
          onclick={() => (emulatorStore.activeFilter = 'retro')}
          class="rounded-lg px-2.5 py-1 font-mono text-xs font-semibold transition-all {emulatorStore.activeFilter === 'retro'
            ? 'bg-white text-black shadow-md'
            : 'text-zinc-400 hover:text-white'}"
        >
          Retro
        </button>
      </div>
    </div>
  </div>

  <!-- Status Notification -->
  {#if statusMessage}
    <div
      class="flex items-center justify-between rounded-xl border p-3 text-xs animate-in fade-in {statusMessage.type === 'success'
        ? 'border-emerald-500/30 bg-emerald-500/10 text-emerald-300'
        : statusMessage.type === 'error'
        ? 'border-rose-500/30 bg-rose-500/10 text-rose-300'
        : 'border-white/20 bg-white/5 text-zinc-200'}"
    >
      <span>{statusMessage.text}</span>
      <button
        type="button"
        onclick={() => (statusMessage = null)}
        class="text-zinc-400 hover:text-white"
      >
        ✕
      </button>
    </div>
  {/if}

  <!-- Catalog Grid -->
  <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-2 gap-4">
    {#each emulatorStore.filteredCatalog as emu}
      {@const progress = emulatorStore.progressMap[emu.id]}
      {@const isBusy = installingId === emu.id || (progress && progress.phase !== 'completed' && progress.phase !== 'error')}

      <div
        class="flex flex-col justify-between rounded-2xl border border-white/10 bg-[#121215]/90 p-5 shadow-lg transition-all hover:border-white/25"
      >
        <!-- Top Info Header -->
        <div class="space-y-3">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2.5">
              <span class="rounded-lg bg-white/10 px-2 py-0.5 font-mono text-[10px] font-bold text-zinc-300">
                {emu.short_system}
              </span>
              <h3 class="text-base font-bold text-white tracking-tight">{emu.name}</h3>
              <span class="font-mono text-xs text-zinc-500">v{emu.version}</span>
            </div>

            {#if emu.installed}
              <span class="flex items-center gap-1.5 rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2.5 py-0.5 font-mono text-[10px] font-bold text-emerald-400">
                ● INSTALLED
              </span>
            {/if}
          </div>

          <p class="text-xs text-zinc-300 leading-relaxed">
            {emu.description}
          </p>

          <div class="flex flex-wrap items-center gap-2 pt-1 font-mono text-[10px] text-zinc-400">
            <span>By: <span class="text-zinc-300">{emu.author}</span></span>
            <span>•</span>
            <span>Repo: <a href="https://github.com/{emu.repo}" target="_blank" class="text-zinc-300 hover:underline">{emu.repo}</a></span>
          </div>

          <!-- Formats Chips -->
          <div class="flex flex-wrap gap-1 pt-1">
            {#each emu.supported_extensions.slice(0, 5) as ext}
              <span class="rounded bg-white/5 px-1.5 py-0.5 font-mono text-[9px] text-zinc-400">
                {ext}
              </span>
            {/each}
            {#if emu.supported_extensions.length > 5}
              <span class="rounded bg-white/5 px-1.5 py-0.5 font-mono text-[9px] text-zinc-500">
                +{emu.supported_extensions.length - 5}
              </span>
            {/if}
          </div>
        </div>

        <!-- Progress Bar (when downloading/extracting) -->
        {#if isBusy && progress}
          <div class="my-4 space-y-1.5 rounded-xl border border-white/10 bg-black/40 p-3">
            <div class="flex items-center justify-between font-mono text-[10px]">
              <span class="uppercase tracking-wider text-amber-300">
                {progress.phase}...
              </span>
              <span class="text-zinc-300">
                {Math.round(progress.percentage)}% • {progress.speedMbS.toFixed(1)} MB/s
              </span>
            </div>
            <div class="h-1.5 w-full overflow-hidden rounded-full bg-white/10">
              <div
                class="h-full bg-amber-400 rounded-full transition-all duration-200"
                style="width: {progress.percentage}%"
              ></div>
            </div>
          </div>
        {/if}

        <!-- Actions Bar -->
        <div class="mt-5 flex flex-wrap items-center justify-between gap-3 border-t border-white/10 pt-4">
          <div class="flex items-center gap-2">
            {#if emu.website}
              <a
                href={emu.website}
                target="_blank"
                class="text-xs text-zinc-400 hover:text-white transition-colors"
              >
                Website ↗
              </a>
            {/if}
          </div>

          <div class="flex items-center gap-2">
            {#if emu.installed}
              <button
                type="button"
                onclick={() => handleLaunch(emu)}
                class="rounded-xl bg-white px-4 py-1.5 font-mono text-xs font-bold text-black shadow-md hover:bg-zinc-200 transition-colors"
              >
                LAUNCH
              </button>
            {:else}
              <button
                type="button"
                disabled={isBusy}
                onclick={() => handleInstall(emu, false)}
                class="rounded-xl bg-white px-4 py-1.5 font-mono text-xs font-bold text-black shadow-md hover:bg-zinc-200 transition-colors disabled:opacity-50"
              >
                {isBusy ? 'INSTALLING...' : '1-CLICK INSTALL'}
              </button>
              <button
                type="button"
                disabled={isBusy}
                onclick={() => handleInstall(emu, true)}
                class="rounded-xl border border-white/15 bg-white/5 px-2.5 py-1.5 text-xs text-zinc-300 hover:bg-white/10 hover:text-white transition-colors disabled:opacity-50"
                title="Select Custom Install Directory"
              >
                Custom Folder...
              </button>
            {/if}
          </div>
        </div>
      </div>
    {/each}
  </div>
</div>
