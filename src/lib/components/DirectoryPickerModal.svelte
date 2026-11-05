<!--
  DirectoryPickerModal.svelte — In-launcher folder browser & drive visualizer.
  TURA & Liquid Chrome aesthetic:
  - Eliminates OS/Steam dialog popups
  - Drive space utilization bars & logical drive switcher
  - Interactive breadcrumbs and folder navigation
  - Inline folder creation with Windows reserved names validation
-->
<script lang="ts">
  import {
    directoryPicker,
    calculateUtilization,
    isReservedWindowsName,
  } from '$lib/stores/filesystem';

  let newFolderName = $state('');
  let isCreatingFolder = $state(false);
  let createError = $state<string | null>(null);

  function splitBreadcrumbs(pathStr: string) {
    if (!pathStr) return [];
    const normalized = pathStr.replace(/\\+$/, '');
    const parts = normalized.split('\\');
    const crumbs: { label: string; fullPath: string }[] = [];

    let acc = '';
    for (let i = 0; i < parts.length; i++) {
      if (i === 0) {
        acc = `${parts[0]}\\`;
        crumbs.push({ label: parts[0] || 'Drive', fullPath: acc });
      } else {
        acc = `${acc}${parts[i]}\\`;
        crumbs.push({ label: parts[i], fullPath: acc });
      }
    }
    return crumbs;
  }

  const breadcrumbs = $derived.by(() => splitBreadcrumbs(directoryPicker.currentPath));

  async function handleCreateFolder() {
    if (!newFolderName.trim()) return;
    createError = null;

    if (isReservedWindowsName(newFolderName)) {
      createError = `'${newFolderName}' is a reserved Windows device name (CON, PRN, AUX, NUL...)`;
      return;
    }

    try {
      await directoryPicker.createFolder(newFolderName.trim());
      newFolderName = '';
      isCreatingFolder = false;
    } catch (err: any) {
      createError = err?.message || String(err);
    }
  }

  function handleSelectCurrent() {
    directoryPicker.select(directoryPicker.currentPath);
  }

  function handleCancel() {
    directoryPicker.cancel();
  }
</script>

{#if directoryPicker.isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-md p-4">
    <div
      class="relative flex h-[620px] w-full max-w-2xl flex-col rounded-2xl border border-white/15 bg-[#0e0e12] shadow-2xl text-white overflow-hidden animate-in fade-in zoom-in-95 duration-200"
    >
      <!-- Modal Header -->
      <div class="flex items-center justify-between border-b border-white/10 px-6 py-4 bg-white/[0.02]">
        <div class="flex items-center gap-2.5">
          <div class="flex h-7 w-7 items-center justify-center rounded-lg bg-white/10 text-zinc-300">
            <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
              />
            </svg>
          </div>
          <div>
            <h3 class="text-sm font-bold tracking-tight text-white">{directoryPicker.title}</h3>
            <p class="text-[11px] text-zinc-400">In-launcher directory browser • No OS dialogs</p>
          </div>
        </div>

        <button
          type="button"
          onclick={handleCancel}
          aria-label="Close dialog"
          class="rounded-lg p-1.5 text-zinc-400 hover:bg-white/10 hover:text-white transition-colors"
        >
          <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Drive Selector Row -->
      <div class="border-b border-white/10 bg-black/40 px-6 py-3">
        <div class="flex items-center gap-2.5 overflow-x-auto pb-1 scrollbar-thin">
          {#each directoryPicker.drives as drive}
            {@const util = calculateUtilization(drive.totalBytes, drive.freeBytes)}
            {@const isActive = directoryPicker.currentPath.toUpperCase().startsWith(drive.drive.toUpperCase())}
            <button
              type="button"
              onclick={() => directoryPicker.navigateTo(drive.mountPoint)}
              class="group flex min-w-[140px] flex-col rounded-xl border p-2 text-left transition-all {isActive
                ? 'border-white/40 bg-white/10 shadow-sm'
                : 'border-white/10 bg-white/[0.03] hover:border-white/25 hover:bg-white/[0.06]'}"
            >
              <div class="flex items-center justify-between text-xs font-bold">
                <span class={isActive ? 'text-white' : 'text-zinc-300'}>{drive.label}</span>
                <span class="font-mono text-[10px] text-zinc-400">{drive.freeGb.toFixed(1)} GB free</span>
              </div>
              <div class="mt-1.5 h-1.5 w-full overflow-hidden rounded-full bg-white/10">
                <div
                  class="h-full rounded-full transition-all {util > 90
                    ? 'bg-rose-500'
                    : util > 80
                    ? 'bg-amber-400'
                    : 'bg-emerald-400'}"
                  style="width: {util}%"
                ></div>
              </div>
            </button>
          {/each}
        </div>
      </div>

      <!-- Breadcrumbs Bar & Actions -->
      <div class="flex flex-wrap items-center justify-between gap-2 border-b border-white/10 bg-white/[0.01] px-6 py-2.5">
        <div class="flex flex-wrap items-center gap-1 font-mono text-xs">
          {#if directoryPicker.listing?.parentPath}
            <button
              type="button"
              onclick={() => directoryPicker.navigateTo(directoryPicker.listing?.parentPath || 'C:\\')}
              class="rounded px-2 py-1 text-zinc-400 hover:bg-white/10 hover:text-white transition-colors"
              title="Go to parent directory"
            >
              ↑ Parent
            </button>
            <span class="text-zinc-600">/</span>
          {/if}

          {#each breadcrumbs as crumb, i}
            <button
              type="button"
              onclick={() => directoryPicker.navigateTo(crumb.fullPath)}
              class="rounded px-1.5 py-0.5 transition-colors {i === breadcrumbs.length - 1
                ? 'bg-white/15 text-white font-bold'
                : 'text-zinc-400 hover:text-white hover:bg-white/10'}"
            >
              {crumb.label}
            </button>
            {#if i < breadcrumbs.length - 1}
              <span class="text-zinc-600">\</span>
            {/if}
          {/each}
        </div>

        <button
          type="button"
          onclick={() => {
            isCreatingFolder = !isCreatingFolder;
            createError = null;
          }}
          class="flex items-center gap-1.5 rounded-lg border border-white/15 bg-white/5 px-2.5 py-1 text-xs font-semibold text-zinc-200 hover:bg-white/10 transition-colors"
        >
          <svg class="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          New Folder
        </button>
      </div>

      <!-- Inline Folder Creation Form -->
      {#if isCreatingFolder}
        <div class="border-b border-white/10 bg-white/5 px-6 py-2.5 animate-in fade-in">
          <form
            onsubmit={(e) => {
              e.preventDefault();
              handleCreateFolder();
            }}
            class="flex items-center gap-2"
          >
            <input
              type="text"
              bind:value={newFolderName}
              placeholder="Enter new folder name..."
              class="flex-1 rounded-lg border border-white/20 bg-black/60 px-3 py-1.5 text-xs text-white placeholder-zinc-500 outline-none focus:border-white/50"
            />
            <button
              type="submit"
              class="rounded-lg bg-white px-3 py-1.5 text-xs font-bold text-black hover:bg-zinc-200 transition-colors"
            >
              Create
            </button>
            <button
              type="button"
              onclick={() => {
                isCreatingFolder = false;
                createError = null;
              }}
              class="rounded-lg border border-white/15 bg-transparent px-3 py-1.5 text-xs text-zinc-300 hover:bg-white/10 transition-colors"
            >
              Cancel
            </button>
          </form>
          {#if createError}
            <p class="mt-1.5 text-[11px] text-rose-400 font-mono">{createError}</p>
          {/if}
        </div>
      {/if}

      <!-- Folder Directory Listing -->
      <div class="flex-1 overflow-y-auto p-4 space-y-1">
        {#if directoryPicker.loading}
          <div class="flex h-48 items-center justify-center">
            <div class="h-6 w-6 animate-spin rounded-full border-2 border-white/20 border-t-white"></div>
          </div>
        {:else if !directoryPicker.listing || directoryPicker.listing.entries.length === 0}
          <div class="flex h-48 flex-col items-center justify-center text-center text-zinc-500">
            <svg class="h-10 w-10 stroke-zinc-600 mb-2" fill="none" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="1.5"
                d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z"
              />
            </svg>
            <p class="text-xs">No subdirectories in this location</p>
          </div>
        {:else}
          {#each directoryPicker.listing.entries as entry}
            <button
              type="button"
              onclick={() => directoryPicker.navigateTo(entry.path)}
              class="group flex w-full items-center justify-between rounded-xl px-3 py-2 text-left transition-all hover:bg-white/[0.06]"
            >
              <div class="flex items-center gap-3 min-w-0">
                <svg class="h-5 w-5 text-amber-400/90 shrink-0" fill="currentColor" viewBox="0 0 20 20">
                  <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
                </svg>
                <span class="truncate text-xs font-semibold text-zinc-200 group-hover:text-white">
                  {entry.name}
                </span>
              </div>
              <span class="text-[11px] text-zinc-500 font-mono opacity-0 group-hover:opacity-100 transition-opacity">
                Open →
              </span>
            </button>
          {/each}
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="flex flex-wrap items-center justify-between gap-3 border-t border-white/10 bg-white/[0.02] px-6 py-4">
        <div class="flex flex-col min-w-0">
          <span class="text-[11px] text-zinc-400">Current Destination:</span>
          <span class="truncate font-mono text-xs text-white" title={directoryPicker.currentPath}>
            {directoryPicker.currentPath}
          </span>
        </div>

        <div class="flex items-center gap-2.5">
          <button
            type="button"
            onclick={handleCancel}
            class="rounded-xl border border-white/15 bg-white/5 px-4 py-2 text-xs font-semibold text-zinc-300 hover:bg-white/10 hover:text-white transition-colors"
          >
            Cancel
          </button>
          <button
            type="button"
            onclick={handleSelectCurrent}
            class="rounded-xl bg-white px-5 py-2 text-xs font-bold text-black shadow-lg hover:bg-zinc-200 transition-colors"
          >
            Select This Folder
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
