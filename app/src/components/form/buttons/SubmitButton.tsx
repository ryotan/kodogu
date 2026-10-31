import type {PropsWithChildren, ReactElement} from 'react';

import {Button} from '@radix-ui/themes';

import {useFormContext} from '../FormContext';

export function SubmitButton({children}: PropsWithChildren): ReactElement {
  const form = useFormContext();
  return (
    <form.Subscribe selector={state => [state.canSubmit, state.isSubmitting]}>
      {([canSubmit, isSubmitting]) => (
        <Button disabled={!canSubmit} loading={isSubmitting}>
          {children}
        </Button>
      )}
    </form.Subscribe>
  );
}
