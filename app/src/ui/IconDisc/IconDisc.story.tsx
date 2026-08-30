import {CheckCircledIcon, ExclamationTriangleIcon, InfoCircledIcon} from '@radix-ui/react-icons';
import {Flex} from '@radix-ui/themes';

import type {Meta, StoryObj} from '@storybook/react-vite';

import {IconDisc} from './IconDisc';

const meta: Meta<typeof IconDisc> = {
  component: IconDisc,
  // A primitive doesn't need the production shell; `fill: false` collapses the surface to content height.
  parameters: {surface: {value: 'app-dark', fill: false}, docs: {canvas: {withToolbar: true}}},
  argTypes: {
    color: {control: 'inline-radio', options: ['red', 'amber', 'green', 'blue', 'gray']},
  },
  args: {color: 'red', children: <ExclamationTriangleIcon height={20} width={20} />},
};
export default meta;

type Story = StoryObj<typeof IconDisc>;

export const Default: Story = {};

// The tint, glow and glyph all follow `color`.
export const Colors: Story = {
  render: () => (
    <Flex gap='4'>
      <IconDisc color='red'>
        <ExclamationTriangleIcon height={20} width={20} />
      </IconDisc>
      <IconDisc color='amber'>
        <ExclamationTriangleIcon height={20} width={20} />
      </IconDisc>
      <IconDisc color='green'>
        <CheckCircledIcon height={20} width={20} />
      </IconDisc>
      <IconDisc color='blue'>
        <InfoCircledIcon height={20} width={20} />
      </IconDisc>
      <IconDisc color='gray'>
        <InfoCircledIcon height={20} width={20} />
      </IconDisc>
    </Flex>
  ),
};
