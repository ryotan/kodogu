import {ExclamationTriangleIcon} from '@radix-ui/react-icons';
import {Box, Flex, Heading, Text} from '@radix-ui/themes';

import {IconDisc} from '#ui/IconDisc/IconDisc';

import './errors.css';

import {getUserMessage} from './getUserMessage';

type ErrorDisplayProps = {
  error: Error;
};

/**
 * User-facing presentation of an error. Renders the alert icon, the message
 * (via {@link getUserMessage}, Pattern A from ADR-0007), and — when the
 * top-level cause is an `AggregateError` — the children as a list.
 *
 * No retry, no debug section, no stack traces. Those live in
 * {@link import('./ErrorPage').ErrorPage} and
 * {@link import('#(dev)/ErrorDebugSection').ErrorDebugSection} respectively.
 */
export const ErrorDisplay = ({error}: ErrorDisplayProps) => {
  const children = aggregateChildren(error);
  return (
    <Flex align='center' direction='column' gap='3'>
      <IconDisc>
        <ExclamationTriangleIcon height={20} width={20} />
      </IconDisc>
      <Heading align='center' size='4' weight='medium'>
        {getUserMessage(error)}
      </Heading>
      {!children && (
        <Text align='center' color='gray' size='2'>
          時間をおいて、もう一度お試しください。
        </Text>
      )}
      {children && <AggregateChildren errors={children} />}
    </Flex>
  );
};

const AggregateChildren = ({errors}: {errors: Error[]}) => (
  // Natural width, centred by the parent Flex's `align='center'`. The red
  // left rule sits at the block's left edge, items remain left-aligned
  // inside, but the block as a whole shares the centred rhythm.
  <Box className='error-aggregate-children' pl='3' py='1'>
    <Flex direction='column' gap='2'>
      {errors.map((child, i) => (
        <Text key={i} size='2'>
          {getUserMessage(child)}
        </Text>
      ))}
    </Flex>
  </Box>
);

/**
 * Returns the children list when the top-level cause is an `AggregateError`,
 * otherwise null. Filtered to `Error` instances — non-Error throws in the
 * aggregate are skipped to avoid leaking arbitrary strings to the user.
 */
function aggregateChildren(error: Error): Error[] | null {
  if (!(error.cause instanceof AggregateError)) {
    return null;
  }
  return error.cause.errors.filter((e): e is Error => e instanceof Error);
}
