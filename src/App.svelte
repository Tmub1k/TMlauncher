<script lang="ts">
  import { onMount } from 'svelte';
  import TitleBar from './lib/components/TitleBar.svelte';
  import BentoHub from './lib/components/BentoHub.svelte';
  import { themeEngine } from './lib/themes/theme-engine';
  import './lib/styles/app.css';

  onMount(() => {
    void themeEngine.init();
  });
</script>

<div class="flex h-screen w-screen flex-col overflow-hidden bg-[var(--bg-canvas)] text-[var(--text-primary)]">
  <!-- Frameless TitleBar Header (44px) -->
  <TitleBar />

  <!-- Main View Area -->
  <main class="relative flex-1 min-h-0 w-full overflow-hidden">
    <BentoHub />
  </main>
</div>
