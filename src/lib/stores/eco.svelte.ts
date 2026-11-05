/**
 * eco.svelte.ts — Handheld & Laptop Eco / Battery Saver Mode store.
 * Throttles rendering frame rate, disables expensive CSS blurs, and applies
 * true OLED pitch black theme to maximize device battery life.
 */

class EcoStore {
  enabled = $state(false);
  oledBlack = $state(true);
  reduceMotion = $state(true);
  batteryLevel = $state<number | null>(null);
  isCharging = $state(true);

  constructor() {
    this.init();
  }

  private init() {
    if (typeof window === 'undefined') return;

    // Load saved eco preference from localStorage
    try {
      const saved = localStorage.getItem('omnilauncher.eco_mode');
      if (saved === 'true') {
        this.enabled = true;
        document.documentElement.dataset.eco = 'true';
      }
    } catch {
      // Storage unavailable
    }

    // Connect to Battery Status API if supported
    if ('getBattery' in navigator) {
      (navigator as any).getBattery()
        .then((battery: any) => {
          this.batteryLevel = Math.round(battery.level * 100);
          this.isCharging = battery.charging;

          battery.addEventListener('chargingchange', () => {
            this.isCharging = battery.charging;
            if (!battery.charging && this.batteryLevel !== null && this.batteryLevel < 25) {
              this.enable();
            }
          });

          battery.addEventListener('levelchange', () => {
            this.batteryLevel = Math.round(battery.level * 100);
            if (!this.isCharging && this.batteryLevel < 20) {
              this.enable();
            }
          });
        })
        .catch(() => {
          // Battery API denied or desktop environment
        });
    }
  }

  enable() {
    this.enabled = true;
    if (typeof document !== 'undefined') {
      document.documentElement.dataset.eco = 'true';
    }
    try {
      localStorage.setItem('omnilauncher.eco_mode', 'true');
    } catch {}
  }

  disable() {
    this.enabled = false;
    if (typeof document !== 'undefined') {
      document.documentElement.dataset.eco = 'false';
    }
    try {
      localStorage.setItem('omnilauncher.eco_mode', 'false');
    } catch {}
  }

  toggle() {
    if (this.enabled) {
      this.disable();
    } else {
      this.enable();
    }
  }
}

export const ecoStore = new EcoStore();
