import {createFormHook} from '@tanstack/react-form';

import {fieldContext, formContext} from './FormContext';
import {SubmitButton} from './buttons/SubmitButton';
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
