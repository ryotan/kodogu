import {ExclamationTriangleIcon} from '@radix-ui/react-icons';
import {Box, Flex, Heading, Text} from '@radix-ui/themes';

import {IconDisc} from '#ui/IconDisc/IconDisc';

import './errors.css';

import {getUserMessage} from './getUserMessage';

type ErrorDisplayProps = {
  error: Error;
};

/**
 * User-facing presentation of a single caught error: the alert icon, the
 * user-facing message (via {@link getUserMessage}, Pattern A from ADR-0007),
 * and — when the error represents *several* failures at once (an
 * `AggregateError`, e.g. multiple form-validation errors) — each child's
 * message as a flat list.
 *
 * Deliberately flat: one error, optionally its direct aggregate children. The
 * recursive cause *tree* (cause chains, stack traces, component stack) is a
 * debug concern and lives in
 * {@link import('#(dev)/ErrorDebugSection').ErrorDebugSection}; retry lives in
 * {@link import('./ErrorPage').ErrorPage}.
 */
export function ErrorDisplay({error}: ErrorDisplayProps) {
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
}

function AggregateChildren({errors}: {errors: Error[]}) {
  // Natural width, centred by the parent Flex's `align='center'`. The red
  // left rule sits at the block's left edge, items remain left-aligned
  // inside, but the block as a whole shares the centred rhythm.
  return (
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
}

/**
 * The aggregate's children when this error represents several failures at once,
 * otherwise null. An aggregate reaches the user in two shapes, both handled here:
 *   - thrown directly — the caught `error` *is* the `AggregateError`;
 *   - wrapped — a {@link import('./RootError').RootError} carries the user
 *     message and holds the aggregate as its `cause`, so {@link getUserMessage}
 *     has a Pattern-A message to show above the list.
 * Children are filtered to `Error` instances so non-Error throws in the
 * aggregate never leak arbitrary strings to the user.
 */
function aggregateChildren(error: Error): Error[] | null {
  const aggregate = resolveAggregate(error);
  if (!aggregate) {
    return null;
  }
  return aggregate.errors.filter((e): e is Error => e instanceof Error);
}

// An aggregate is detected on the error itself first (a directly-thrown aggregate),
// then on its cause (a RootError wrapping an aggregate to carry the user message).
function resolveAggregate(error: Error): AggregateError | null {
  if (error instanceof AggregateError) {
    return error;
  }
  if (error.cause instanceof AggregateError) {
    return error.cause;
  }
  return null;
}
