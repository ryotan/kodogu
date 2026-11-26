import type {ReactElement} from 'react';

import type {Meta, StoryObj} from '@storybook/react-vite';
import {type} from 'arktype';
import {expect, fn, mocked, waitFor, within} from 'storybook/test';

import {useAppForm} from './Form';

type Values = {name: string};
type FormExampleProps = {
  errorKind: 'message' | 'arktype';
  onSubmit: (value: Values) => Promise<void>;
};

const nameSchema = type.string.atLeastLength(2);
const validationMessage = 'Enter at least two characters';

function FormExample({errorKind, onSubmit}: FormExampleProps): ReactElement {
  const form = useAppForm({
    defaultValues: {name: 'Valid'},
    onSubmit: ({value}) => onSubmit(value),
  });

  return (
    <form action={() => form.handleSubmit()}>
      <form.AppForm>
        <form.AppField
          name='name'
          validators={{
            onChange: ({value}) => {
              const result = nameSchema(value);
              return result instanceof type.errors
                ? errorKind === 'arktype'
                  ? result[0]
                  : validationMessage
                : undefined;
            },
          }}
        >
          {field => <field.InputText label='Name' />}
        </form.AppField>
        <form.SubmitButton>Save</form.SubmitButton>
      </form.AppForm>
    </form>
  );
}

const meta = {
  component: FormExample,
  args: {errorKind: 'message', onSubmit: fn<(value: Values) => Promise<void>>().mockResolvedValue(undefined)},
} satisfies Meta<typeof FormExample>;

export default meta;
type Story = StoryObj<typeof meta>;

function checkValidation(invalidValue: string): Story['play'] {
  return async ({args, canvas, canvasElement, userEvent}) => {
    const input = canvas.getByRole('textbox', {name: 'Name'});
    const submit = canvas.getByRole('button', {name: 'Save'});
    await expect(input).toHaveValue('Valid');
    await expect(submit).toBeEnabled();

    await userEvent.clear(input);
    if (invalidValue) {
      await userEvent.type(input, invalidValue);
    }
    await expect(input).toHaveValue(invalidValue);
    await waitFor(() => expect(submit).toBeDisabled());
    await expect(input).toHaveAccessibleName('Name');
    await userEvent.click(submit);
    await expect(args.onSubmit).not.toHaveBeenCalled();

    const trigger = canvas.getByRole('button', {name: '入力エラーを表示'});
    await userEvent.click(trigger);
    const dialog = await within(canvasElement.ownerDocument.body).findByRole('dialog');
    const result = nameSchema(invalidValue);
    if (!(result instanceof type.errors)) {
      throw new Error('The invalid input unexpectedly passed validation');
    }
    const message = args.errorKind === 'arktype' ? result[0].message : validationMessage;
    await waitFor(() => expect(within(dialog).getByText(message)).toBeVisible());
    await userEvent.keyboard('{Escape}');
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
    await waitFor(() => expect(trigger).toHaveFocus());
    await userEvent.keyboard('{Enter}');
    const reopened = await within(canvasElement.ownerDocument.body).findByRole('dialog');
    await waitFor(() => expect(within(reopened).getByText(message)).toBeVisible());
    await userEvent.keyboard('{Escape}');
    await waitFor(() => expect(reopened).not.toBeInTheDocument());

    await userEvent.clear(input);
    await userEvent.type(input, 'AB');
    await expect(input).toHaveValue('AB');
    await waitFor(() => expect(submit).toBeEnabled());
    await expect(canvas.queryByRole('button', {name: '入力エラーを表示'})).not.toBeInTheDocument();
    await userEvent.type(input, 'C');
    await expect(input).toHaveValue('ABC');
    await expect(submit).toBeEnabled();
    await userEvent.click(submit);
    await waitFor(() => expect(mocked(args.onSubmit).mock.calls).toEqual([[{name: 'ABC'}]]));
  };
}

export const EmptyValueWithMessage: Story = {args: {errorKind: 'message'}, play: checkValidation('')};
export const ShortValueWithMessage: Story = {args: {errorKind: 'message'}, play: checkValidation('A')};
export const EmptyValueWithArkError: Story = {args: {errorKind: 'arktype'}, play: checkValidation('')};
export const ShortValueWithArkError: Story = {args: {errorKind: 'arktype'}, play: checkValidation('A')};

export const PendingSubmission: Story = {
  play: async ({args, canvas, userEvent}) => {
    const completion: {resolve?: () => void} = {};
    const pending = new Promise<void>(resolve => {
      completion.resolve = resolve;
    });
    mocked(args.onSubmit).mockReturnValueOnce(pending);
    const input = canvas.getByRole('textbox', {name: 'Name'});
    const submit = canvas.getByRole('button', {name: 'Save'});
    await userEvent.clear(input);
    await userEvent.type(input, 'Alice');

    try {
      await userEvent.click(submit);
      await waitFor(() => expect(mocked(args.onSubmit).mock.calls).toEqual([[{name: 'Alice'}]]));
      await expect(submit).toBeDisabled();
      await expect(submit).toHaveAccessibleName('Save');
      await expect(submit.querySelector('.rt-Spinner')).toBeVisible();
      await userEvent.click(submit);
      await expect(args.onSubmit).toHaveBeenCalledTimes(1);
    } finally {
      completion.resolve?.();
    }
    await waitFor(() => expect(submit).toBeEnabled());
    await expect(submit.querySelector('.rt-Spinner')).not.toBeInTheDocument();
    await expect(input).toHaveValue('Alice');
    await expect(mocked(args.onSubmit).mock.calls).toEqual([[{name: 'Alice'}]]);
  },
};
