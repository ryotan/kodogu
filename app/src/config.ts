// =============================================================================
// Value types — mirrors newtype definitions in app/src-tauri/src/config.rs.
// Branded at compile time only; runtime values are plain primitives.
// =============================================================================

declare const _brand: unique symbol;
type Brand<T, B> = T & {readonly [_brand]: B};

export type Milliseconds = Brand<number, 'Milliseconds'>;
export type LogicalPixel = Brand<number, 'LogicalPixel'>;
export type CssColor = Brand<string, 'CssColor'>;
export type CssFilter = Brand<string, 'CssFilter'>;
export type CssFontFamily = Brand<string, 'CssFontFamily'>;
export type CssLength = Brand<string, 'CssLength'>;
export type CssPadding = Brand<string, 'CssPadding'>;

export const Milliseconds = (n: number): Milliseconds => n as Milliseconds;
export const LogicalPixel = (n: number): LogicalPixel => n as LogicalPixel;
export const CssColor = (s: string): CssColor => s as CssColor;
export const CssFilter = (s: string): CssFilter => s as CssFilter;
export const CssFontFamily = (s: string): CssFontFamily => s as CssFontFamily;
export const CssLength = (s: string): CssLength => s as CssLength;
export const CssPadding = (s: string): CssPadding => s as CssPadding;

// =============================================================================
// Config types — mirrors struct definitions in app/src-tauri/src/config.rs.
// Fields are camelCase (Rust side uses #[serde(rename_all = "camelCase")]).
// =============================================================================

export type PopoverBehaviorConfig = {
  toggleDebounceMs: Milliseconds;
};

export type PopoverLayoutConfig = {
  initialWidth: LogicalPixel;
  bottomMargin: LogicalPixel;
  trayGap: LogicalPixel;
  fallbackMaxHeight: LogicalPixel;
  minMaxHeight: LogicalPixel;
  headerHeight: CssLength;
  cardListPadding: CssPadding;
};

export type PopoverAppearanceConfig = {
  surfaceColor: CssColor;
  borderColor: CssColor;
  backdropFilter: CssFilter;
  fontFamily: CssFontFamily;
};

export type PopoverConfig = {
  behavior: PopoverBehaviorConfig;
  layout: PopoverLayoutConfig;
  appearance: PopoverAppearanceConfig;
};

export type AppConfig = {
  popover: PopoverConfig;
};
