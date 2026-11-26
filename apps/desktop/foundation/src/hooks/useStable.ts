import {useState} from 'react';

/**
 * Returns the value `initializer` creates on mount, keeping the same instance for as long as the component stays
 * mounted. `useMemo` cannot do this because React may discard memoized values and recompute them.
 */
export function useStable<T>(initializer: () => T): T {
  // The value is never replaced, so there is no setter to take.
  // eslint-disable-next-line react/hook-use-state
  const [value] = useState(initializer);
  return value;
}
