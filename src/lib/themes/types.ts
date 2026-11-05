/**
 * Modular Theme System — Type Definitions
 * Compliant with OmniLauncher Flagship UI Spec (TURA & Liquid Chrome)
 */

export type WindowEffectKind = 'mica' | 'tabbed' | 'acrylic' | 'none';

export type AccentStyle = 'matte' | 'chrome' | 'iridescent' | 'neon' | string;

export interface ThemeTokens {
  // Surface & Canvas
  '--bg-canvas': string;
  '--bg-surface': string;
  '--bg-surface-base': string;
  '--bg-surface-elevated': string;
  '--bg-surface-floating': string;
  '--bg-surface-glass': string;


  // Liquid Chrome Accents
  '--chrome-gradient': string;
  '--chrome-border': string;
  '--chrome-specular': string;
  '--chrome-ribbon': string;

  // Borders & Hairlines
  '--border-hairline': string;
  '--border-subtle': string;
  '--border-active': string;
  '--border-specular': string;

  // Typography
  '--text-primary': string;
  '--text-secondary': string;
  '--text-muted': string;
  '--text-accent': string;

  // Primary & Metallic Accents
  '--accent-primary': string;
  '--accent-primary-contrast': string;
  '--accent-metallic': string;
  '--accent-chrome': string;
  '--accent-glow': string;

  // Status Indicators
  '--status-success': string;
  '--status-warning': string;
  '--status-danger': string;
  '--status-info': string;

  // Shadows & Elevation
  '--shadow-card': string;
  '--shadow-card-hover': string;
  '--shadow-modal': string;
  '--shadow-specular': string;

  // Geometry & Radii
  '--radius-xs': string;
  '--radius-sm': string;
  '--radius-md': string;
  '--radius-lg': string;
  '--radius-xl': string;
  '--radius-full': string;

  // Typography Settings
  '--font-sans': string;
  '--font-mono': string;
  '--font-heading-tracking': string;
  '--font-heading-weight': string;

  // Artwork & Visual Filters
  '--poster-filter': string;
  '--poster-filter-hover': string;
  '--backdrop-blur': string;

  // Extensible Custom Properties
  [key: `--${string}`]: string;
}

export interface ThemeMedia {
  backgroundVideo?: string | null;
  ambientAudio?: string | null;
  audioVolume?: number;
}

export interface ThemeDefinition {
  $schema?: string;
  id: string;
  name: string;
  author: string;
  version: string;
  description?: string;
  accentStyle?: AccentStyle;
  vibrancy?: WindowEffectKind;
  tokens: ThemeTokens | Record<string, string>;
  media?: ThemeMedia;
  isCustom?: boolean;
  filePath?: string;
}

export interface ThemeSummary {
  id: string;
  name: string;
  author: string;
  version: string;
  isCustom: boolean;
  filePath?: string;
}

export type ThemeChangeListener = (theme: ThemeDefinition) => void;
