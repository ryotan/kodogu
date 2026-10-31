// =============================================================================
// Value types — mirrors newtype definitions in app/src-tauri/src/config/domain.rs.
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

export function Milliseconds(n: number): Milliseconds {
  return n as Milliseconds;
}
export function LogicalPixel(n: number): LogicalPixel {
  return n as LogicalPixel;
}
export function CssColor(s: string): CssColor {
  return s as CssColor;
}
export function CssFilter(s: string): CssFilter {
  return s as CssFilter;
}
export function CssFontFamily(s: string): CssFontFamily {
  return s as CssFontFamily;
}
export function CssLength(s: string): CssLength {
  return s as CssLength;
}
export function CssPadding(s: string): CssPadding {
  return s as CssPadding;
}

// =============================================================================
// Config types — mirrors struct definitions in app/src-tauri/src/config/domain.rs.
// Fields are camelCase (Rust side uses #[serde(rename_all = "camelCase")]).
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
