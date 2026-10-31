import type {ComponentProps, ReactElement} from 'react';

import {CheckCircledIcon, CrossCircledIcon} from '@radix-ui/react-icons';
import {Flex, IconButton, Popover, Text, TextField} from '@radix-ui/themes';

import {ArkError} from 'arktype';

import {useFieldContext} from '../FormContext';

export type InputTextProps = Omit<ComponentProps<typeof TextField.Root>, 'name' | 'onChange' | 'type' | 'value'> & {
  label: string;
};

type ErrorPopoverProps = {
  errors: unknown[];
};

function ErrorPopover({errors}: ErrorPopoverProps): ReactElement {
  return (
    <Popover.Root>
      <Popover.Trigger>
        <IconButton aria-label='入力エラーを表示' color='red' size='1' type='button' variant='ghost'>
          <CrossCircledIcon />
        </IconButton>
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
}

export function InputText({label, ...props}: InputTextProps): ReactElement {
  const field = useFieldContext<string>();
  const errors = field.state.meta.errors;
  return (
    <label htmlFor={field.name}>
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
}
