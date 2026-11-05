import { invoke } from '@tauri-apps/api/core';

export interface StoreMedia {
  screenshots: string[];
  webmVideo?: string;
  mp4Video?: string;
}

export interface StoreItem {
  app_id: number;
  title: string;
  price_raw: number;
  price_formatted: string;
  original_price_raw?: number;
  original_price_formatted?: string;
  discount_percent: number;
  thumbnail_url: string;
  header_image?: string;
  banner_url?: string;
  tags: string[];
  review_score?: string;
  media?: StoreMedia;
  short_description?: string;
  release_date?: string;
  store_url: string;
  install_uri: string;
}

export interface StorePage {
  page: number;
  currency: 'USD' | 'KZT';
  items: StoreItem[];
  has_more: boolean;
  total_pages: number;
  end_message?: string;
}

export const FIXED_EXCHANGE_RATE = 495.0;

class CurrencyStore {
  code = $state<'USD' | 'KZT'>('USD');
  rate = $state<number>(FIXED_EXCHANGE_RATE);
  lastUpdated = $state<number>(Date.now());

  isStale = $derived.by(() => {
    const maxAgeMs = 7 * 24 * 3600 * 1000; // 7 days
    return Date.now() - this.lastUpdated > maxAgeMs;
  });

  constructor() {
    // Attempt to load preferred currency from localStorage or backend
    if (typeof localStorage !== 'undefined') {
      const saved = localStorage.getItem('omni_preferred_currency');
      if (saved === 'USD' || saved === 'KZT') {
        this.code = saved;
      }
    }
  }

  async setCurrency(newCode: 'USD' | 'KZT') {
    if (newCode !== 'USD' && newCode !== 'KZT') return;
    this.code = newCode;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('omni_preferred_currency', newCode);
    }
    try {
      await invoke('set_preferred_currency', { currency: newCode });
    } catch (e) {
      console.warn('Could not persist currency to backend:', e);
    }
  }

  convertFromUsd(usd: number): number {
    if (usd <= 0) return 0;
    if (this.code === 'USD') return usd;
    return Math.max(1, Math.round(usd * this.rate));
  }

  format(usd: number): string {
    if (usd === 0) return 'Free to Play';
    if (this.code === 'USD') {
      return `$${usd.toFixed(2)}`;
    }
    const kzt = Math.max(1, Math.round(usd * this.rate));
    return `${kzt.toLocaleString('ru-RU')} ₸`;
  }

  formatSavings(regularUsd: number, saleUsd: number): string {
    const savedUsd = regularUsd - saleUsd;
    if (this.code === 'KZT') {
      const savedKzt = Math.round(savedUsd * this.rate);
      return `Вы экономите ${savedKzt.toLocaleString('ru-RU')} ₸`;
    }
    return `You save $${savedUsd.toFixed(2)}`;
  }
}

class StoreFeedStore {
  items = $state<StoreItem[]>([]);
  page = $state<number>(0);
  hasMore = $state<boolean>(true);
  loading = $state<boolean>(false);
  inFlight = $state<boolean>(false);
  selectedTag = $state<string>('All');
  searchQuery = $state<string>('');
  endMessage = $state<string | null>(null);

  async loadPage(pageToLoad: number, append = false): Promise<boolean> {
    if (this.inFlight) return false;
    this.inFlight = true;
    this.loading = true;

    try {
      const activeCurrency = currencyStore.code;
      const res = await invoke<StorePage>('get_store_recommendations', {
        page: pageToLoad,
        currency: activeCurrency,
        tag: this.selectedTag === 'All' ? null : this.selectedTag,
        query: this.searchQuery.trim() ? this.searchQuery : null,
      });

      if (append) {
        this.items = [...this.items, ...res.items];
      } else {
        this.items = res.items;
      }

      this.page = res.page;
      this.hasMore = res.has_more;
      this.endMessage = res.end_message || null;
      return true;
    } catch (err) {
      console.error('Failed to load store recommendations:', err);
      return false;
    } finally {
      this.loading = false;
      this.inFlight = false;
    }
  }

  async loadNext(): Promise<boolean> {
    if (!this.hasMore || this.loading || this.inFlight) {
      return false;
    }
    return await this.loadPage(this.page + 1, true);
  }

  setTag(tag: string) {
    if (this.selectedTag === tag) return;
    this.selectedTag = tag;
    this.reset();
    this.loadPage(0, false);
  }

  setSearch(query: string) {
    this.searchQuery = query;
    this.reset();
    this.loadPage(0, false);
  }

  reset() {
    this.items = [];
    this.page = 0;
    this.hasMore = true;
    this.endMessage = null;
  }
}

export const currencyStore = new CurrencyStore();
export const storeFeed = new StoreFeedStore();
