<!--
  RouletteModal.svelte — "What to Play Today?" Backlog Roulette + HLTB Slot Machine.
  Features constraint filtering, animated slot reel strip, and synthesized Web Audio sound effects.
-->
<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { launchGameById, resolveArtUrl, type Game, type PlatformKind } from '../stores/library';

  interface HltbBenchmark {
    title: string;
    mainStoryHours: number;
    mainExtrasHours: number;
    completionistHours: number;
  }

  interface RouletteResult {
    winner: Game;
    hltb: HltbBenchmark;
    reelCandidates: Game[];
    reason: string;
  }

  interface Props {
    open: boolean;
    onClose: () => void;
  }

  let { open, onClose }: Props = $props();

  // Web Audio Synthesizer Engine
  class RouletteSoundEngine {
    private ctx: AudioContext | null = null;

    private init() {
      if (!this.ctx && typeof window !== 'undefined') {
        const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
        if (AudioCtx) this.ctx = new AudioCtx();
      }
      if (this.ctx && this.ctx.state === 'suspended') {
        void this.ctx.resume();
      }
    }

    playTick(frequency = 750, volume = 0.08) {
      try {
        this.init();
        if (!this.ctx) return;
        const osc = this.ctx.createOscillator();
        const gain = this.ctx.createGain();
        osc.type = 'triangle';
        osc.frequency.setValueAtTime(frequency, this.ctx.currentTime);
        osc.frequency.exponentialRampToValueAtTime(150, this.ctx.currentTime + 0.025);
        gain.gain.setValueAtTime(volume, this.ctx.currentTime);
        gain.gain.exponentialRampToValueAtTime(0.001, this.ctx.currentTime + 0.025);
        osc.connect(gain);
        gain.connect(this.ctx.destination);
        osc.start();
        osc.stop(this.ctx.currentTime + 0.03);
      } catch {
        // Audio unavailable
      }
    }

    playWinnerChime() {
      try {
        this.init();
        if (!this.ctx) return;
        const notes = [523.25, 659.25, 783.99]; // C5, E5, G5
        notes.forEach((freq, i) => {
          const osc = this.ctx!.createOscillator();
          const gain = this.ctx!.createGain();
          osc.type = 'sine';
          osc.frequency.setValueAtTime(freq, this.ctx!.currentTime + i * 0.07);
          gain.gain.setValueAtTime(0.12, this.ctx!.currentTime + i * 0.07);
          gain.gain.exponentialRampToValueAtTime(0.001, this.ctx!.currentTime + i * 0.07 + 0.4);
          osc.connect(gain);
          gain.connect(this.ctx!.destination);
          osc.start(this.ctx!.currentTime + i * 0.07);
          osc.stop(this.ctx!.currentTime + i * 0.07 + 0.45);
        });
      } catch {
        // Audio unavailable
      }
    }
  }

  const soundEngine = new RouletteSoundEngine();

  // Filter state
  let maxSessionMinutes = $state<number | null>(120);
  let completionStatus = $state<'all' | 'unplayed' | 'in_progress' | 'completed'>('all');
  let platformFilter = $state<PlatformKind | 'all'>('all');
  let controllerOnly = $state(false);
  let hltbTier = $state<'all' | 'short' | 'medium' | 'epic'>('all');

  // Animation state
  let spinning = $state(false);
  let reelOffset = $state(0);
  let reelDisplayCandidates = $state<Game[]>([]);
  let result = $state<RouletteResult | null>(null);
  let errorMsg = $state<string | null>(null);

  async function spin() {
    if (spinning) return;
    spinning = true;
    errorMsg = null;
    result = null;

    try {
      const res = await invoke<RouletteResult>('spin_backlog_roulette', {
        filter: {
          maxSessionMinutes,
          unplayedOnly: completionStatus === 'unplayed',
          platform: platformFilter === 'all' ? null : platformFilter,
        },
      });

      // Prepare reel cards (repeating candidates for slot effect)
      const pool = res.reelCandidates && res.reelCandidates.length > 0
        ? res.reelCandidates
        : [res.winner];
      
      const cards: Game[] = [];
      for (let i = 0; i < 15; i++) {
        cards.push(pool[i % pool.length]);
      }
      cards.push(res.winner);
      reelDisplayCandidates = cards;

      // Animate slot reel with decelerating mechanical clicks
      let step = 0;
      const totalSteps = 24;
      let delay = 35;

      const runReelTick = () => {
        step++;
        reelOffset = (step * 85) % (cards.length * 85);
        soundEngine.playTick(600 + (step % 4) * 80, 0.07);

        if (step < totalSteps) {
          delay = Math.floor(delay * 1.09); // exponential deceleration
          setTimeout(runReelTick, delay);
        } else {
          // Locked on winner
          reelOffset = (cards.length - 1) * 85;
          spinning = false;
          result = res;
          soundEngine.playWinnerChime();
        }
      };

      runReelTick();

    } catch (err: any) {
      spinning = false;
      errorMsg = err?.message ?? 'No matching games found in library';
    }
  }
</script>

{#if open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/85 p-4 backdrop-blur-md"
    role="dialog"
    aria-modal="true"
  >
    <div
      class="w-full max-w-xl rounded-2xl border border-white/15 bg-[#121215] p-6 text-white shadow-2xl relative overflow-hidden"
    >
      <!-- Specular Top Sheen -->
      <div class="pointer-events-none absolute -top-24 left-1/2 h-48 w-96 -translate-x-1/2 rounded-full bg-gradient-to-b from-white/10 to-transparent blur-2xl"></div>

      <!-- Header -->
      <div class="flex items-center justify-between border-b border-white/10 pb-4 relative z-10">
        <div class="flex items-center gap-3">
          <div class="flex h-9 w-9 items-center justify-center rounded-xl border border-white/20 bg-white/10 text-base shadow-inner">
            🎲
          </div>
          <div>
            <h2 class="text-base font-bold tracking-tight">Intelligent Backlog Roulette</h2>
            <p class="font-mono text-[11px] text-zinc-400">
              Curation engine with slot-machine reel & HowLongToBeat forecasting
            </p>
          </div>
        </div>
        <button
          type="button"
          onclick={onClose}
          class="rounded-lg border border-white/10 px-2.5 py-1 font-mono text-xs text-zinc-400 hover:border-white/30 hover:text-white"
        >
          ESC
        </button>
      </div>

      <!-- Filter Matrix -->
      <div class="mt-4 space-y-3.5 relative z-10">
        <!-- Available Session Window -->
        <div>
          <span class="block font-mono text-[10px] uppercase tracking-widest text-zinc-400">
            Available Session Window
          </span>
          <div class="mt-1.5 grid grid-cols-3 gap-2">
            {#each [
              { val: 30, label: '30 Mins (Burst)' },
              { val: 120, label: '2 Hours (Standard)' },
              { val: null, label: 'Unlimited (Marathon)' },
            ] as opt}
              <button
                type="button"
                onclick={() => (maxSessionMinutes = opt.val)}
                class="rounded-xl border px-3 py-2 font-mono text-xs transition-all {maxSessionMinutes === opt.val
                  ? 'border-white bg-white font-semibold text-black shadow-md'
                  : 'border-white/10 bg-white/5 text-zinc-300 hover:border-white/25'}"
              >
                {opt.label}
              </button>
            {/each}
          </div>
        </div>

        <!-- Completion Status & Platform Filters -->
        <div class="grid grid-cols-2 gap-3">
          <div>
            <span class="block font-mono text-[10px] uppercase tracking-widest text-zinc-400">
              Completion Status
            </span>
            <div class="mt-1.5 grid grid-cols-2 gap-1.5 font-mono text-xs">
              <button
                type="button"
                onclick={() => (completionStatus = 'all')}
                class="rounded-lg border px-2 py-1.5 text-center transition-all {completionStatus === 'all'
                  ? 'border-white bg-white/20 text-white font-bold'
                  : 'border-white/10 bg-white/5 text-zinc-400 hover:text-white'}"
              >
                All Statuses
              </button>
              <button
                type="button"
                onclick={() => (completionStatus = 'unplayed')}
                class="rounded-lg border px-2 py-1.5 text-center transition-all {completionStatus === 'unplayed'
                  ? 'border-purple-400/80 bg-purple-500/20 text-purple-200 font-bold'
                  : 'border-white/10 bg-white/5 text-zinc-400 hover:text-white'}"
              >
                Unplayed (&lt;30m)
              </button>
            </div>
          </div>

          <div>
            <span class="block font-mono text-[10px] uppercase tracking-widest text-zinc-400">
              Platform & Hardware
            </span>
            <div class="mt-1.5 flex gap-2">
              <select
                bind:value={platformFilter}
                class="flex-1 rounded-lg border border-white/10 bg-[#18181b] px-2.5 py-1.5 font-mono text-xs text-zinc-200 outline-none hover:border-white/25"
              >
                <option value="all">All Stores</option>
                <option value="steam">Steam</option>
                <option value="epic">Epic Games</option>
                <option value="roblox">Roblox</option>
                <option value="itch">itch.io</option>
                <option value="emulator">Emulators</option>
              </select>

              <button
                type="button"
                onclick={() => (controllerOnly = !controllerOnly)}
                class="rounded-lg border px-2.5 py-1.5 font-mono text-xs transition-all {controllerOnly
                  ? 'border-emerald-400/80 bg-emerald-500/20 text-emerald-200 font-bold'
                  : 'border-white/10 bg-white/5 text-zinc-400 hover:text-white'}"
                title="Filter for Full Controller Support"
              >
                🎮 Pad
              </button>
            </div>
          </div>
        </div>

        <!-- Slot Reel Strip Visualizer -->
        {#if spinning && reelDisplayCandidates.length > 0}
          <div class="relative h-24 overflow-hidden rounded-xl border border-white/20 bg-black/70 shadow-inner">
            <div class="pointer-events-none absolute inset-0 z-20 bg-gradient-to-b from-black/80 via-transparent to-black/80"></div>
            <div class="pointer-events-none absolute left-0 right-0 top-1/2 h-14 -translate-y-1/2 border-y border-white/30 bg-white/5 z-10"></div>
            
            <div
              class="flex flex-col items-center transition-transform duration-75 ease-out"
              style="transform: translateY(-{reelOffset}px);"
            >
              {#each reelDisplayCandidates as cand}
                <div class="flex h-[85px] w-full items-center justify-center gap-3 px-4">
                  {#if resolveArtUrl(cand.art?.poster)}
                    <img
                      src={resolveArtUrl(cand.art?.poster)}
                      alt={cand.title}
                      class="h-14 w-10 rounded object-cover shadow"
                    />
                  {/if}
                  <div class="text-left">
                    <span class="font-mono text-[9px] uppercase tracking-wider text-zinc-400">{cand.platform}</span>
                    <h4 class="font-bold text-sm text-white truncate max-w-[280px]">{cand.title}</h4>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Winner Card -->
        {#if result && !spinning}
          <div class="rounded-xl border border-emerald-500/30 bg-emerald-950/20 p-4 shadow-xl">
            <div class="flex items-center gap-4">
              {#if resolveArtUrl(result.winner.art?.poster)}
                <img
                  src={resolveArtUrl(result.winner.art?.poster)}
                  alt={result.winner.title}
                  class="h-28 w-20 rounded-lg object-cover shadow-md border border-white/15"
                />
              {/if}
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span
                    class="rounded border border-emerald-400/40 bg-emerald-500/20 px-2 py-0.5 font-mono text-[10px] font-bold uppercase text-emerald-200"
                  >
                    ★ WINNER
                  </span>
                  <span class="font-mono text-[10px] text-zinc-400 uppercase">
                    {result.winner.platform}
                  </span>
                </div>
                <h3 class="mt-1 truncate text-lg font-bold text-white tracking-tight">{result.winner.title}</h3>
                <p class="font-mono text-[11px] text-zinc-300 mt-0.5">{result.reason}</p>

                <div class="mt-2.5 grid grid-cols-3 gap-2 font-mono text-[10px]">
                  <div class="rounded-lg bg-black/40 border border-white/10 px-2 py-1.5 text-center">
                    <span class="block text-[9px] text-zinc-400">Main Story</span>
                    <strong class="text-emerald-300 text-xs">{result.hltb.mainStoryHours}h</strong>
                  </div>
                  <div class="rounded-lg bg-black/40 border border-white/10 px-2 py-1.5 text-center">
                    <span class="block text-[9px] text-zinc-400">+Extras</span>
                    <strong class="text-zinc-200 text-xs">{result.hltb.mainExtrasHours}h</strong>
                  </div>
                  <div class="rounded-lg bg-black/40 border border-white/10 px-2 py-1.5 text-center">
                    <span class="block text-[9px] text-zinc-400">Completionist</span>
                    <strong class="text-zinc-200 text-xs">{result.hltb.completionistHours}h</strong>
                  </div>
                </div>
              </div>
            </div>
          </div>
        {:else if errorMsg}
          <p class="rounded-xl border border-rose-500/30 bg-rose-950/20 p-3 font-mono text-xs text-rose-300">
            {errorMsg}
          </p>
        {/if}

        <!-- Actions -->
        <div class="flex items-center justify-end gap-2.5 pt-2">
          <button
            type="button"
            onclick={spin}
            disabled={spinning}
            class="rounded-xl border border-white/20 bg-white/10 px-5 py-2.5 text-xs font-semibold text-white hover:bg-white hover:text-black transition-all"
          >
            {spinning ? 'Rolling Reel…' : result ? 'Spin Again 🎲' : 'Spin Slot Machine 🎲'}
          </button>
          {#if result && !spinning}
            <button
              type="button"
              onclick={() => {
                if (result) {
                  launchGameById(result.winner.id);
                  onClose();
                }
              }}
              class="rounded-xl bg-white px-5 py-2.5 text-xs font-bold text-black shadow-lg hover:bg-zinc-200 transition-all"
            >
              Launch Game 🚀
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
