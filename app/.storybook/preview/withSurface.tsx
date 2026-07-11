import type {ReactNode} from 'react';

import {Box, Flex, Theme} from '@radix-ui/themes';

import type {Decorator} from '@storybook/react-vite';

import type {Appearance, SurfaceLiteral} from './surfaces';
import {resolveSurface} from './surfaces';

declare module '@storybook/react-vite' {
  // biome-ignore lint/style/useConsistentTypeDefinitions: TypeScript module augmentation requires `interface`; `type` cannot extend an existing interface from another module.
  interface Parameters {
    /**
     * Story configuration consumed by the {@link withSurface} decorator.
     * Has no effect when the decorator is not applied to the story.
     */
    surface?: {
      /** Render the story without wrapping it in the Radix Theme. */
      skip?: boolean;
      /**
       * Which production surface the story should render against by default.
       * Overridable at runtime via the `Surface` toolbar control. Falls back
       * to `app-inherit` if neither this nor the toolbar global is set.
       *
       * Surface naming uses `${kind}-${appearance}` where:
       * - kind ∈ {app, popover} — which production layout the surface mimics
       * - appearance ∈ {inherit, light, dark} — Radix Theme appearance
       *
       * `inherit` follows the OS / browser `prefers-color-scheme` (the same value
       * Radix Theme uses to defer to system).
       */
      value?: SurfaceLiteral;
      /**
       * Whether the surface fills the canvas height (`min-height: 100dvh`).
       * Defaults to `true` — production surfaces (popover window, app panel)
       * occupy the full height. Set `false` for component/primitive stories so
       * the canvas collapses to content height instead of a tall empty area.
       *
       * Resolved in this decorator (not a surface prop), so it never leaks into
       * the surface's own API when the surface graduates to production.
       */
      fill?: boolean;
    };
  }
}

// =============================================================================
// Surface primitives — production-equivalent Theme + shell wrappers
//
// These will eventually move into a real `ThemeProvider` + surface primitive
// pair in `app/src/ui/`. The Storybook-side stop-gap lives here, so individual
// stories don't have to know about Theme settings, and so the production
// refactor only needs to touch one delegation site. See KDG-29 comment for
// the broader plan.
// =============================================================================

function PopoverSurface({appearance, children}: {appearance: Appearance; children: ReactNode}) {
  return (
    <Theme appearance={appearance} hasBackground={false}>
      {/* Desktop / canvas — area around the Tauri popover window. Horizontal
        center + top-start places the window emulator like a real macOS
        popover. This is sim cosmetic, not "layout centring".
        `100dvh` here is equivalent to the Storybook canvas size in the
        `layout: 'fullscreen'` mode this prototype uses (the iframe's own
        viewport). It's not coupling to the host browser viewport. */}
      <Flex align='start' className='surface-region' justify='center' p='6' style={{minHeight: '100dvh'}}>
        {/* Tauri window emulator. The native popover window can't content-fit
          itself (the OS won't size the window without programmatic input),
          so in production `useAutoResize` resizes the window to match content.
          Here we let the box be CSS content-fit, which has the same visual. */}
        <Box
          style={{
            width: 360,
            borderRadius: 14,
            overflow: 'hidden',
            boxShadow: '0 24px 64px -12px rgba(0, 0, 0, 0.7)',
          }}
        >
          {/* `.popover-shell` sim — mirrors app/src/routes/popover.css. The
            production CSS says `height: 100dvh`, but combined with the
            window being content-fit, the rendered height matches content
            (= no centring opportunity anyway). Defaults for --kodogu-surface
            come from popover.css; useDesignTokenSync is *meant* to override
            them per AppConfig, but the wire-up is broken (see KDG-29). */}
          <Box
            style={{
              ['--kodogu-surface' as never]: 'rgba(28, 28, 30, 0.92)',
              background: 'var(--kodogu-surface)',
              backdropFilter: 'blur(24px) saturate(180%)',
              WebkitBackdropFilter: 'blur(24px) saturate(180%)',
              display: 'flex',
              flexDirection: 'column',
            }}
          >
            {children}
          </Box>
        </Box>
      </Flex>
    </Theme>
  );
}

function AppSurface({appearance, children}: {appearance: Appearance; children: ReactNode}) {
  return (
    <Theme appearance={appearance} panelBackground='solid'>
      {/* Flex-column container sized to the Storybook canvas. `100dvh` here is
        equivalent to the iframe's own viewport (= canvas) under
        `layout: 'fullscreen'`, not the host browser viewport. Pages opting
        into self-centring (e.g., an error page that sets `flex: 1`) fill the
        canvas and do their own centring; pages that don't opt in flow from
        the top, indistinguishable from the block layout. */}
      <Flex className='surface-region' direction='column' style={{minHeight: '100dvh'}}>
        {children}
      </Flex>
    </Theme>
  );
}

export const withSurface: Decorator = (Story, {parameters, globals}) => {
  if (parameters.surface?.skip === true) {
    return <Story />;
  }
  const surface = resolveSurface(globals, parameters.surface?.value);
  const SurfaceComponent = surface.kind === 'popover' ? PopoverSurface : AppSurface;
  // `fill` defaults to true (production surfaces occupy full height). When false, the wrapper class
  // collapses the surface's `min-height: 100dvh` (see surface.css) so the canvas fits content.
  const fill = parameters.surface?.fill ?? true;
  const surfaceElement = (
    <SurfaceComponent appearance={surface.appearance}>
      <Story />
    </SurfaceComponent>
  );
  return fill ? surfaceElement : <div className='sb-no-fill'>{surfaceElement}</div>;
};
