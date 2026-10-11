// =============================================================================
// Value types — mirrors newtype definitions in app/src-tauri/src/config.rs.
// Branded at compile time only; runtime values are plain primitives.
// No runtime validation — Rust is the single source of truth for valid values.
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
//
// No runtime validation on invoke<AppConfig> — the type parameter is
// compile-time only. Rust's serde serialization is the source of truth.
// =============================================================================

export type PopoverBehaviorConfig = {
  readonly toggleDebounceMs: Milliseconds;
};

export type PopoverLayoutConfig = {
  readonly initialWidth: LogicalPixel;
  readonly bottomMargin: LogicalPixel;
  readonly trayGap: LogicalPixel;
  readonly fallbackMaxHeight: LogicalPixel;
  readonly minMaxHeight: LogicalPixel;
  readonly headerHeight: CssLength;
  readonly cardListPadding: CssPadding;
};

export type PopoverAppearanceConfig = {
  readonly surfaceColor: CssColor;
  readonly borderColor: CssColor;
  readonly backdropFilter: CssFilter;
  readonly fontFamily: CssFontFamily;
};

export type PopoverConfig = {
  readonly behavior: PopoverBehaviorConfig;
  readonly layout: PopoverLayoutConfig;
  readonly appearance: PopoverAppearanceConfig;
};

export type AppConfig = {
  readonly popover: PopoverConfig;
};
