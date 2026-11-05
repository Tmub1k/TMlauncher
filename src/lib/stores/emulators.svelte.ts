import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface EmulatorCatalogEntry {
  id: string;
  name: string;
  repo: string;
  version: string;
  installed: boolean;
  executable_name: string;
  arch: string;
  system: string;
  short_system: string;
  author: string;
  description: string;
  supported_extensions: string[];
  install_path?: string;
  executable_path?: string;
  is_rate_limited: boolean;
  website?: string;
}

export interface EmulatorProgressEvent {
  emulatorId: string;
  phase: 'downloading' | 'extracting' | 'registering' | 'completed' | 'error';
  downloadedBytes: number;
  totalBytes: number;
  percentage: number;
  speedMbS: number;
  error?: string;
}

export interface EmulatorInstallResult {
  success: boolean;
  installed: boolean;
  emulator_id: string;
  installed_location: string;
  dialog_type: string;
  registered_game_id: string;
}

class EmulatorStore {
  catalog = $state<EmulatorCatalogEntry[]>([]);
  progressMap = $state<Record<string, EmulatorProgressEvent>>({});
  loading = $state<boolean>(false);
  activeFilter = $state<'all' | 'nintendo' | 'sony' | 'retro'>('all');
  searchQuery = $state<string>('');
  private unlisten: UnlistenFn | null = null;

  filteredCatalog = $derived.by(() => {
    let list = this.catalog;

    if (this.activeFilter === 'nintendo') {
      list = list.filter((e) =>
        e.system.toLowerCase().includes('nintendo') ||
        e.short_system.includes('WII') ||
        e.short_system.includes('SWITCH') ||
        e.short_system.includes('3DS')
      );
    } else if (this.activeFilter === 'sony') {
      list = list.filter((e) =>
        e.system.toLowerCase().includes('playstation') ||
        e.short_system.includes('PS')
      );
    } else if (this.activeFilter === 'retro') {
      list = list.filter((e) =>
        e.system.toLowerCase().includes('retro') ||
        e.short_system.includes('MULTI')
      );
    }

    if (this.searchQuery.trim()) {
      const q = this.searchQuery.toLowerCase().trim();
      list = list.filter(
        (e) =>
          e.name.toLowerCase().includes(q) ||
          e.system.toLowerCase().includes(q) ||
          e.description.toLowerCase().includes(q)
      );
    }

    return list;
  });

  async init() {
    await this.loadCatalog();
    this.startProgressListener();
  }

  async loadCatalog() {
    this.loading = true;
    try {
      const res = await invoke<EmulatorCatalogEntry[]>('get_emulator_catalog');
      this.catalog = res;
    } catch (err) {
      console.error('Failed to load emulator catalog:', err);
    } finally {
      this.loading = false;
    }
  }

  async startProgressListener() {
    if (this.unlisten) return;
    try {
      this.unlisten = await listen<EmulatorProgressEvent>(
        'emulator://progress',
        (event) => {
          const payload = event.payload;
          if (payload && payload.emulatorId) {
            this.progressMap[payload.emulatorId] = payload;
            if (payload.phase === 'completed') {
              this.loadCatalog();
            }
          }
        }
      );
    } catch (err) {
      console.warn('Could not register emulator progress listener:', err);
    }
  }

  async install(emulatorId: string, targetDir?: string): Promise<EmulatorInstallResult | null> {
    try {
      const res = await invoke<EmulatorInstallResult>('install_emulator', {
        emulatorId,
        targetDir: targetDir || null,
      });
      await this.loadCatalog();
      return res;
    } catch (err) {
      console.error(`Failed to install emulator ${emulatorId}:`, err);
      throw err;
    }
  }

  async launch(emulatorId: string): Promise<boolean> {
    try {
      await invoke('launch_emulator', { emulatorId });
      return true;
    } catch (err) {
      console.error(`Failed to launch emulator ${emulatorId}:`, err);
      return false;
    }
  }
}

export const emulatorStore = new EmulatorStore();
