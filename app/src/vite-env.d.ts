/// <reference types="vite/client" />

// Build-time flag controlling whether dev-only tools (devtools panels, stack-trace UI, etc.) are
// rendered. Replaced literally by Vite's `define` so tree-shaking eliminates the disabled branch.
declare const __DEV_TOOLS__: boolean;
