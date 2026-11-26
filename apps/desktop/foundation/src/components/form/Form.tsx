import {createFormHook} from '@tanstack/react-form';

import {SubmitButton} from './buttons/SubmitButton';
import {fieldContext, formContext} from './FormContext';
import {InputText} from './fields/InputText';

export const {useAppForm, withForm} = createFormHook({
  fieldComponents: {
    InputText,
  },
  formComponents: {
    SubmitButton,
  },
  fieldContext,
  formContext,
});
