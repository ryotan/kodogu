import {useState} from 'react';

import {Link2Icon} from '@radix-ui/react-icons';
import {Badge, Box, Card, Flex, Inset, Text} from '@radix-ui/themes';

import type {Meta, StoryObj} from '@storybook/react-vite';
import {expect, userEvent} from 'storybook/test';

import {Disclosure} from './Disclosure';

const meta: Meta<typeof Disclosure.Section> = {
  component: Disclosure.Section,
  // A primitive doesn't need a full-height production shell; `fill: false` collapses the surface's
  // 100dvh so docs/story canvases sit at content height.
  // `docs.canvas.withToolbar` shows the zoom / open-in-new toolbar on each docs Canvas block.
  parameters: {surface: {value: 'app-dark', fill: false}, docs: {canvas: {withToolbar: true}}},
  // Section is the flat shorthand, so its props map cleanly onto controls. The compound stories
  // below render their own composition and don't use args.
  argTypes: {
    label: {control: 'text'},
    size: {control: 'inline-radio', options: ['1', '2']},
    defaultOpen: {control: 'boolean'},
    disabled: {control: 'boolean'},
  },
};
export default meta;

type Story = StoryObj<typeof Disclosure.Section>;

// =============================================================================
// Section shorthand — the common case: <Disclosure.Section label>. Owns the standard trigger row
// (label left, chevron right) and content padding. Constrained width so it reads like production.
// =============================================================================

export const Section: Story = {
  args: {label: 'Details', size: '2', defaultOpen: false},
  render: args => (
    <Box width='380px'>
      <Disclosure.Section {...args}>
        <Text color='gray' size='2'>
          Section content.
        </Text>
      </Disclosure.Section>
    </Box>
  ),
  play: async ({canvas}) => {
    const trigger = canvas.getByRole('button', {name: /details/i});
    await expect(trigger).toHaveAttribute('aria-expanded', 'false');
    await userEvent.click(trigger);
    await expect(trigger).toHaveAttribute('aria-expanded', 'true');
    await expect(canvas.getByText('Section content.')).toBeVisible();
  },
};

export const SectionOpen: Story = {
  render: () => (
    <Box width='380px'>
      <Disclosure.Section defaultOpen label='Details'>
        <Text color='gray' size='2'>
          Section content.
        </Text>
      </Disclosure.Section>
    </Box>
  ),
};

// Sections nest; the inner ones use the compact `size='1'`.
export const SectionNested: Story = {
  render: () => (
    <Box width='380px'>
      <Disclosure.Section defaultOpen label='Debug info'>
        <Disclosure.Section label='Stack trace' size='1'>
          <Text color='gray' size='2'>
            Frames…
          </Text>
        </Disclosure.Section>
        <Disclosure.Section label='Component stack' size='1'>
          <Text color='gray' size='2'>
            Components…
          </Text>
        </Disclosure.Section>
      </Disclosure.Section>
    </Box>
  ),
};

export const SectionDisabled: Story = {
  render: () => (
    <Box width='380px'>
      <Disclosure.Section disabled label='Details'>
        <Text color='gray' size='2'>
          Section content.
        </Text>
      </Disclosure.Section>
    </Box>
  ),
  play: async ({canvas}) => {
    await expect(canvas.getByRole('button', {name: /details/i})).toBeDisabled();
  },
};

// =============================================================================
// Compound primitives — bare composition for the cases Section can't express.
// =============================================================================

// Basic compound composition. `defaultOpen` / `disabled` are Root-level props, so they map to
// controls even though the trigger layout itself is hand-composed (and can't be).
export const Compound: Story = {
  args: {defaultOpen: false, disabled: false},
  argTypes: {
    defaultOpen: {control: 'boolean'},
    disabled: {control: 'boolean'},
    label: {table: {disable: true}},
    size: {table: {disable: true}},
  },
  render: ({defaultOpen, disabled}) => (
    <Box width='380px'>
      <Disclosure.Root defaultOpen={defaultOpen} disabled={disabled}>
        <Disclosure.Trigger>
          <Flex align='center' gap='3' px='4' py='3'>
            <Box flexGrow='1'>
              <Text size='2' weight='medium'>
                Details
              </Text>
            </Box>
            <Disclosure.Chevron />
          </Flex>
        </Disclosure.Trigger>
        <Disclosure.Content>
          <Box pb='3' px='4'>
            <Text color='gray' size='2'>
              Section content.
            </Text>
          </Box>
        </Disclosure.Content>
      </Disclosure.Root>
    </Box>
  ),
};

// A second trigger at the bottom of the content (collapse-from-below), only possible with a Trigger
// inside Content — the case the Section shorthand can't express.
export const CompoundWithFooterToggle: Story = {
  render: () => {
    const [open, setOpen] = useState(false);
    return (
      <Box width='380px'>
        <Disclosure.Root onOpenChange={setOpen} open={open}>
          <Disclosure.Trigger>
            <Flex align='center' gap='3' px='4' py='3'>
              <Box flexGrow='1'>
                <Text size='2' weight='medium'>
                  Stack trace
                </Text>
              </Box>
              <Disclosure.Chevron />
            </Flex>
          </Disclosure.Trigger>
          <Disclosure.Content>
            <Box pb='3' px='4'>
              <Flex direction='column' gap='2'>
                <Text color='gray' size='2'>
                  frame one
                </Text>
                <Text color='gray' size='2'>
                  frame two
                </Text>
                <Disclosure.Trigger>
                  <Flex justify='center' py='2'>
                    <Text color='gray' size='1'>
                      Collapse
                    </Text>
                  </Flex>
                </Disclosure.Trigger>
              </Flex>
            </Box>
          </Disclosure.Content>
        </Disclosure.Root>
      </Box>
    );
  },
  play: async ({canvas}) => {
    await userEvent.click(canvas.getByRole('button', {name: /stack trace/i}));
    await expect(canvas.getByRole('button', {name: /collapse/i})).toBeVisible();
  },
};

// `Chevron` alone (no label) — the consumer centres it.
export const ChevronOnly: Story = {
  render: () => (
    <Box width='380px'>
      <Disclosure.Root>
        <Disclosure.Trigger>
          <Flex justify='center' px='4' py='3'>
            <Disclosure.Chevron />
          </Flex>
        </Disclosure.Trigger>
        <Disclosure.Content>
          <Box pb='3' px='4'>
            <Text color='gray' size='2'>
              Section content.
            </Text>
          </Box>
        </Disclosure.Content>
      </Disclosure.Root>
    </Box>
  ),
};

// =============================================================================
// Realistic usage — the only dressed story. A connection list: each row is a Card hosting a
// Disclosure; the consumer owns the trigger layout/padding and content padding, so everything
// lines up.
// =============================================================================

type Connection = {
  name: string;
  status: {label: string; color: 'green' | 'gray'};
  rows: [string, string][];
};

const connections: Connection[] = [
  {
    name: 'cloud-sql-proxy',
    status: {label: 'running', color: 'green'},
    rows: [
      ['Host', 'db.internal:5432'],
      ['User', 'app_ro'],
      ['SSL', 'required'],
    ],
  },
  {
    name: 'ssh-bastion',
    status: {label: 'running', color: 'green'},
    rows: [
      ['Host', '10.0.1.4:22'],
      ['User', 'deploy'],
      ['Forward', 'localhost:6432'],
    ],
  },
  {
    name: 'aws-ssm',
    status: {label: 'stopped', color: 'gray'},
    rows: [
      ['Instance', 'i-0abc123def'],
      ['Region', 'ap-northeast-1'],
    ],
  },
];

export const RealisticUsage: Story = {
  render: () => (
    <Box width='380px'>
      <Flex direction='column'>
        {connections.map(conn => (
          <Card key={conn.name} size='2'>
            <Inset>
              <Disclosure.Root defaultOpen={conn.name === 'cloud-sql-proxy'}>
                <Disclosure.Trigger>
                  <Flex align='center' gap='3' px='4' py='3'>
                    <Flex align='center' flexGrow='1' gap='2'>
                      <Link2Icon />
                      <Text size='2' weight='medium'>
                        {conn.name}
                      </Text>
                      <Badge color={conn.status.color} size='1'>
                        {conn.status.label}
                      </Badge>
                    </Flex>
                    <Disclosure.Chevron />
                  </Flex>
                </Disclosure.Trigger>
                <Disclosure.Content>
                  <Box pb='3' px='4'>
                    <Flex direction='column' gap='2'>
                      {conn.rows.map(([key, value]) => (
                        <Flex align='center' justify='between' key={key}>
                          <Text color='gray' size='2'>
                            {key}
                          </Text>
                          <Text size='2'>{value}</Text>
                        </Flex>
                      ))}
                    </Flex>
                  </Box>
                </Disclosure.Content>
              </Disclosure.Root>
            </Inset>
          </Card>
        ))}
      </Flex>
    </Box>
  ),
};
