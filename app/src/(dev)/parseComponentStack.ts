export type ComponentStackFrame = {
  component: string;
  location?: string;
};

// React's componentStack lines look like:
//   "    at App"
//   "    at App (file.tsx:42:18)"
//   "    at withForwardRef(MyComp) (file.tsx:42)"   // HOC nest + column-less
// Component name can include parens (HOC nesting), so the inner group excludes parens to
// keep the location's outer parens unambiguous. The column suffix is optional to tolerate
// React variants that omit it.
const FRAME_RE = /^\s*at\s+(.+?)(?:\s+\(([^()]+:\d+(?::\d+)?)\))?$/;

export function parseComponentStack(stack: string): ComponentStackFrame[] {
  return stack
    .split('\n')
    .map(line => FRAME_RE.exec(line))
    .filter((m): m is RegExpExecArray => m !== null)
    .map(([, component, location]) => ({component, ...(location ? {location} : {})}));
}
