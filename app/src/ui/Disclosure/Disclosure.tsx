import type {PropsWithChildren, ReactNode} from 'react';

import {ChevronDownIcon} from '@radix-ui/react-icons';
import {Box, Flex, Reset} from '@radix-ui/themes';
import {Collapsible} from 'radix-ui';

import styles from './Disclosure.module.css';

// Open-state props modelled as a mutually-exclusive union. The `?: never` markers make the
// exclusivity enforceable for variables, not just object literals — in particular `<Disclosure.Root
// open={x} />` without `onOpenChange` becomes a compile error (it can't fall through to the
// uncontrolled branch), which prevents a silently frozen disclosure.
type ControlledProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  defaultOpen?: never;
};
type UncontrolledProps = {
  open?: never;
  onOpenChange?: never;
  /** Initial open state; omit to start closed. */
  defaultOpen?: boolean;
};
type OpenProps = ControlledProps | UncontrolledProps;
type RootProps = PropsWithChildren<{disabled?: boolean}> & OpenProps;

// Owns the open/closed state via Radix Collapsible. Layout and padding live in the consumer's
// composition inside Trigger / Content — the primitive only provides the mechanism.
function Root({children, disabled, open, defaultOpen, onOpenChange}: RootProps) {
  return (
    <Collapsible.Root
      className={styles.root}
      defaultOpen={defaultOpen}
      disabled={disabled}
      onOpenChange={onOpenChange}
      open={open}
    >
      {children}
    </Collapsible.Root>
  );
}

// The trigger. Radix `Collapsible.Trigger` is already a real <button> (aria-expanded + keyboard);
// `<Reset>` strips the native button chrome. The consumer composes the inner row — label, chevron,
// padding — as children; the primitive only forces the full-width row so triggers stay uniform.
function Trigger({children}: PropsWithChildren) {
  return (
    <Reset>
      <Collapsible.Trigger className={styles.trigger}>{children}</Collapsible.Trigger>
    </Reset>
  );
}

// Chevron that rotates when its enclosing Trigger opens. Must be rendered inside a `Trigger` — the
// rotation is driven by the Trigger's `data-state` via a descendant selector.
function Chevron() {
  return <ChevronDownIcon className={styles.chevron} />;
}

// The collapsible region (with open/close height animation). The consumer pads its children.
function Content({children}: PropsWithChildren) {
  return <Collapsible.Content className={styles.content}>{children}</Collapsible.Content>;
}

// Padding scale for the Section shorthand. Selecting from a fixed scale (rather than taking raw
// padding values) keeps spacing consistent; `1` is the compact step for nested sections.
const SECTION_PADDING = {
  '1': {triggerX: '3', triggerY: '2', contentX: '3', contentBottom: '2'},
  '2': {triggerX: '4', triggerY: '3', contentX: '4', contentBottom: '3'},
} as const;

type SectionProps = PropsWithChildren<{
  /** Trigger label, left-aligned with the chevron pinned to the right. */
  label: ReactNode;
  /** Padding scale of the trigger row and content. */
  size?: '1' | '2';
  disabled?: boolean;
}> &
  OpenProps;

// Shorthand for the common case: a labelled trigger (label left, chevron right) over padded
// content. For anything non-standard (e.g. a second trigger at the bottom of the content), compose
// Root / Trigger / Chevron / Content directly instead.
function Section({label, size = '2', children, ...rootProps}: SectionProps) {
  const pad = SECTION_PADDING[size];
  return (
    <Root {...rootProps}>
      <Trigger>
        <Flex align='center' gap='3' px={pad.triggerX} py={pad.triggerY}>
          <Box flexGrow='1'>{label}</Box>
          <Chevron />
        </Flex>
      </Trigger>
      <Content>
        <Box pb={pad.contentBottom} px={pad.contentX}>
          {children}
        </Box>
      </Content>
    </Root>
  );
}

Root.displayName = 'Disclosure.Root';
Trigger.displayName = 'Disclosure.Trigger';
Chevron.displayName = 'Disclosure.Chevron';
Content.displayName = 'Disclosure.Content';
Section.displayName = 'Disclosure.Section';

export const Disclosure = {Root, Trigger, Chevron, Content, Section};
