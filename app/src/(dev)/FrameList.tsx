import {Badge, Box, Flex, Text} from '@radix-ui/themes';

import './FrameList.css';

// Restricted to the palette currently in use by dev-only stack views. Extend as new consumers
// need additional colors rather than letting the entire Radix palette leak into the type.
type BadgeColor = 'gray' | 'blue' | 'green' | 'red';

export type FrameView = {
  badge?: {text: string; color: BadgeColor};
  name: string;
  location?: string;
  /** When true, the frame is dimmed (used to mark `node_modules` etc.). */
  isExternal?: boolean;
};

/**
 * Compact stack-frame list shared by {@link import('./StackTrace').StackTrace}
 * and {@link import('./ComponentStack').ComponentStack}. One frame per line:
 * `at name · location`. Long paths don't wrap — the container scrolls
 * horizontally instead (scrollbar chrome hidden). External frames are dimmed
 * with lower-contrast gray.
 */
export const FrameList = ({frames, emptyMessage}: {frames: FrameView[]; emptyMessage: string}) =>
  frames.length === 0 ? (
    <Text color='gray' size='1'>
      {emptyMessage}
    </Text>
  ) : (
    <Box className='frame-list'>
      <Flex direction='column'>
        {frames.map((frame, i) => (
          <FrameRow frame={frame} key={i} />
        ))}
      </Flex>
    </Box>
  );

const FrameRow = ({frame}: {frame: FrameView}) => (
  <Flex align='baseline' className={`frame-list__row${frame.isExternal ? ' frame-list__row--external' : ''}`} gap='2'>
    {frame.badge && (
      <Badge color={frame.badge.color} size='1' variant='soft'>
        {frame.badge.text}
      </Badge>
    )}
    <Box>
      <Text color='gray'>at </Text>
      <Text className='frame-list__name'>{frame.name}</Text>
      {frame.location && <Text className='frame-list__location'> · {frame.location}</Text>}
    </Box>
  </Flex>
);
