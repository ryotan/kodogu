import {Badge, Card, Code, Flex, Text} from '@radix-ui/themes';

// Restricted to the palette currently in use by dev-only stack views. Extend as new consumers
// need additional colors rather than letting the entire Radix palette leak into the type.
type BadgeColor = 'gray' | 'blue' | 'green' | 'red';

export type FrameView = {
  badge?: {text: string; color: BadgeColor};
  name: string;
  location?: string;
};

export const FrameList = ({frames, emptyMessage}: {frames: FrameView[]; emptyMessage: string}) => (
  <Card size='1'>
    <Flex direction='column' gap='1'>
      {frames.length === 0 ? (
        <Text color='gray' size='1'>
          {emptyMessage}
        </Text>
      ) : (
        frames.map((frame, i) => <FrameRow frame={frame} key={i} />)
      )}
    </Flex>
  </Card>
);

const FrameRow = ({frame}: {frame: FrameView}) => (
  <Flex align='baseline' gap='2'>
    {frame.badge && (
      <Badge color={frame.badge.color} size='1' variant='soft'>
        {frame.badge.text}
      </Badge>
    )}
    <Flex direction='column' minWidth='0'>
      <Code size='1' truncate>
        {frame.name}
      </Code>
      {frame.location && (
        <Text color='gray' size='1' truncate>
          {frame.location}
        </Text>
      )}
    </Flex>
  </Flex>
);
