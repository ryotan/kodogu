import type {ReactNode} from 'react';

import {Flex} from '@radix-ui/themes';

import styles from './IconDisc.module.css';

// Status accent colors a disc realistically uses; extend as new consumers need more (mirrors the
// restricted-palette pattern in FrameList rather than leaking the whole Radix accent scale).
type DiscColor = 'red' | 'amber' | 'green' | 'blue' | 'gray';

type IconDiscProps = {
  /** The glyph shown on the disc (typically an icon); it inherits the accent color via currentColor. */
  children: ReactNode;
  /** Accent color of the tint, glow and glyph. */
  color?: DiscColor;
};

// A tinted circular plate that gives a status glyph (error, warning, …) visual weight. Framing-only:
// the consumer supplies the icon as children. `data-accent-color` drives the tint (see the CSS).
export function IconDisc({children, color = 'red'}: IconDiscProps) {
  return (
    <Flex align='center' className={styles.disc} data-accent-color={color} justify='center'>
      {children}
    </Flex>
  );
}
