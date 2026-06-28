import {Box, Code, Flex, Text} from '@radix-ui/themes';

import {Disclosure} from '#ui/Disclosure/Disclosure';

import type {ErrorChild, ErrorNode} from './buildErrorTree';

import './ErrorTreeNode.css';

import {StackTrace} from './StackTrace';

type ErrorTreeNodeProps = {
  node: ErrorNode;
  showExternal: boolean;
};

// Dev-only. Renders one error in the cause tree. The `name: message` header is itself the toggle —
// the chevron sits inline at its left and clicking the row reveals this error's own stack frames.
// Causes are always shown, nested below under a left rule. Recurses for child errors. Reached only
// through ErrorDebugSection, which is already gated behind __DEV_TOOLS__ + lazy().
export const ErrorTreeNode = ({node, showExternal}: ErrorTreeNodeProps) => (
  <Flex direction='column' gap='1'>
    <Disclosure.Root>
      <Disclosure.Trigger>
        <Flex align='center' gap='1'>
          <Disclosure.Chevron />
          <Box>
            <Code color='red' size='2' variant='ghost'>
              {node.error.name}
            </Code>
            <Text color='gray' size='2'>
              : {node.error.message}
            </Text>
          </Box>
        </Flex>
      </Disclosure.Trigger>
      <Disclosure.Content>
        <StackTrace error={node.error} showExternal={showExternal} />
      </Disclosure.Content>
    </Disclosure.Root>
    {node.children.length > 0 && (
      <Box className='error-tree-node__causes' pl='3'>
        <Flex direction='column' gap='1'>
          {node.children.map((child, i) => (
            <CauseView child={child} key={i} showExternal={showExternal} />
          ))}
        </Flex>
      </Box>
    )}
  </Flex>
);

// One cause of a node: a nested error recurses; a non-Error cause and a truncation point each
// render as a single muted line so nothing is silently dropped.
const CauseView = ({child, showExternal}: {child: ErrorChild; showExternal: boolean}) => {
  switch (child.kind) {
    case 'error':
      return <ErrorTreeNode node={child.node} showExternal={showExternal} />;
    case 'raw':
      return (
        <Text color='gray' size='2'>
          {child.value}
        </Text>
      );
    case 'cutoff':
      return (
        <Text color='gray' size='1'>
          … ({child.reason === 'cycle' ? 'cycle' : 'max depth'})
        </Text>
      );
  }
};
