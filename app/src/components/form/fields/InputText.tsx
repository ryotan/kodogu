import type {ComponentProps, FC} from 'react';

import {CheckCircledIcon, CrossCircledIcon} from '@radix-ui/react-icons';
import {Flex, Popover, Text, TextField} from '@radix-ui/themes';

import {ArkError} from 'arktype';

import {useFieldContext} from '../FormContext';

export type InputTextProps = Omit<ComponentProps<typeof TextField.Root>, 'name' | 'onChange' | 'type' | 'value'> & {
  label: string;
};

type ErrorPopoverProps = {
  errors: unknown[];
};

const ErrorPopover: FC<ErrorPopoverProps> = ({errors}) => {
  return (
    <Popover.Root>
      <Popover.Trigger style={{cursor: 'pointer'}}>
        <CrossCircledIcon color='red' />
      </Popover.Trigger>
      <Popover.Content>
        {errors.map((e, idx) => {
          if (e instanceof ArkError) {
            return (
              <Text color='red' key={`${e.path.join('.')}-${idx}`} size='1'>
                {e.message}
              </Text>
            );
          }
          return (
            <Text color='red' key={String(e)} size='1'>
              {String(e)}
            </Text>
          );
        })}
      </Popover.Content>
    </Popover.Root>
  );
};

export const InputText: FC<InputTextProps> = ({label, ...props}) => {
  const field = useFieldContext<string>();
  const errors = field.state.meta.errors;
  return (
    <label htmlFor='name'>
      <Flex align='center' direction='row' gap='1' justify='between'>
        <Text size='2' weight='bold'>
          {label}
        </Text>
        <TextField.Root
          id={field.name}
          {...props}
          name={field.name}
          onChange={e => field.handleChange(e.target.value)}
          type='text'
          value={field.state.value}
        >
          <TextField.Slot side='right'>
            {errors.length ? <ErrorPopover errors={errors} /> : <CheckCircledIcon color='green' />}
          </TextField.Slot>
        </TextField.Root>
      </Flex>
    </label>
  );
};
