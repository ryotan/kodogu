import type {FC, PropsWithChildren} from 'react';

import {Button} from '@radix-ui/themes';

import {useFormContext} from '../FormContext';

export const SubmitButton: FC<PropsWithChildren> = ({children}) => {
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
};
