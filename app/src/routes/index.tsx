import './index.css';

import {useState} from 'react';

import {createFileRoute} from '@tanstack/react-router';

import {Flex} from '@radix-ui/themes';

import {invoke} from '@tauri-apps/api/core';
import {type} from 'arktype';

import reactLogo from '#assets/react.svg';
import {useAppForm} from '#components/form/Form';

const formInput = type({
  name: type.string.atLeastLength(2),
});
const initialValues = {name: ''} satisfies typeof formInput.infer;

const App = () => {
  const [greetMsg, setGreetMsg] = useState('');

  const form = useAppForm({
    defaultValues: initialValues,
    validators: {onChange: formInput},
    onSubmit: async f => {
      try {
        // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
        setGreetMsg(await invoke('greet', {name: f.value.name}));
      } catch (e) {
        setGreetMsg(`Error: ${String(e)}`);
      }
    },
  });

  return (
    <main className='container'>
      <h1>Welcome to Tauri + React</h1>

      <div className='row'>
        <a href='https://vitejs.dev' rel='noreferrer' target='_blank'>
          <img alt='Vite logo' className='logo vite' src='/vite.svg' />
        </a>
        <a href='https://tauri.app' rel='noreferrer' target='_blank'>
          <img alt='Tauri logo' className='logo tauri' src='/tauri.svg' />
        </a>
        <a href='https://reactjs.org' rel='noreferrer' target='_blank'>
          <img alt='React logo' className='logo react' src={reactLogo} />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form action={() => form.handleSubmit()} autoComplete='off' className='row'>
        <form.AppForm>
          <Flex direction='row' gap='2'>
            <form.AppField name='name'>{field => <field.InputText autoComplete='off' label='Name' />}</form.AppField>
            <form.SubmitButton>Greet</form.SubmitButton>
          </Flex>
        </form.AppForm>
      </form>
      <p>{greetMsg}</p>
    </main>
  );
};

export const Route = createFileRoute('/')({
  component: App,
});
