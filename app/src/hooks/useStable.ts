import {useState} from 'react';

/**
 * Returns a value whose identity stays stable across re-renders for the component's mount
 * lifecycle. The factory runs once on mount; subsequent renders receive the same reference even
 * if the factory closes over values that have changed.
 *
 * The value itself may be mutable — only the reference identity is preserved. Use for objects
 * whose internal state evolves while their identity must remain stable (e.g., a `QueryClient`
 * that owns a cache, an `EventEmitter` with subscribers).
 *
 * Why not `useMemo`? React explicitly reserves the right to discard memoized values to free
 * memory ("React may choose to forget some previously memoized values and recalculate them on
 * next render"). For values whose identity must not change, that's a correctness bug — a new
 * `QueryClient` instance would silently throw away all cached state.
 *
 * Why not `useRef`? `useRef` works but requires a null-check + manual assignment dance on the
 * first render. `useState`'s lazy initializer is the React-blessed idiom for this pattern.
 *
 * To force re-initialization when an input changes, use the React `key` prop on the surrounding
 * component rather than threading dependencies through this hook — `useStable` intentionally
 * does not accept a deps array.
 *
 * @param initializer A factory called exactly once on the first render.
 * @returns The initializer's return value, stable for the component's lifetime.
 */
export function useStable<T>(initializer: () => T): T {
  // The setter is intentionally discarded: this hook is a stability primitive, not a piece of
  // React state. Callers who need to update the value should reach for `useState` directly.
  // eslint-disable-next-line react/hook-use-state
  const [value] = useState(initializer);
  return value;
}
