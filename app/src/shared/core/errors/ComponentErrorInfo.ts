/**
 * The slice of React's ErrorBoundary `errorInfo` that the error views render — the component stack
 * (where in the React tree the error surfaced). Shared by ErrorPage, PopoverAppErrorPage and the
 * dev-only ErrorDebugSection so the prop shape stays defined in one place.
 */
export type ComponentErrorInfo = {
  componentStack: string;
};
