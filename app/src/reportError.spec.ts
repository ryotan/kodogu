// Load Playwright's CDP types without importing its Node implementation into the browser.
import type {} from '@vitest/browser-playwright';
import {describe, expect, it} from 'vitest';
import type {CDPSession} from 'vitest/browser';
import {cdp} from 'vitest/browser';

import {reportError} from '#reportError';

type ConsoleArgument = {type: string; subtype?: string; value?: unknown; objectId?: string};
type ConsoleEvent = {type: string; args: readonly ConsoleArgument[]};

async function readArgument(session: CDPSession, argument: ConsoleArgument): Promise<unknown> {
  if (!argument.objectId) {
    return {type: argument.type, value: argument.value};
  }
  const {result, exceptionDetails} = await session.send('Runtime.callFunctionOn', {
    objectId: argument.objectId,
    functionDeclaration:
      'function() { return this instanceof Error ? {name: this.name, message: this.message, stack: this.stack} : this; }',
    returnByValue: true,
  });
  if (exceptionDetails) {
    throw new Error('Failed to read a console argument');
  }
  const value: unknown = result.value;
  return {type: argument.subtype === 'error' ? 'error' : argument.type, value};
}

const error = new Error('IO failed');
const cases = [
  {name: 'message without details', details: []},
  {name: 'string detail', details: ['detail']},
  {name: 'undefined rejection', details: [undefined]},
  {name: 'object detail', details: [{code: 'E_IO'}]},
  {name: 'error with its stack', details: [error]},
  {name: 'multiple details in order', details: ['detail', {code: 'E_IO'}, undefined]},
];

async function captureErrorOutput(message: string, details: readonly unknown[]): Promise<readonly ConsoleArgument[]> {
  const session = cdp();
  const events: (readonly ConsoleArgument[])[] = [];
  const collect = (event: ConsoleEvent) => {
    if (event.type === 'error') {
      events.push(event.args);
    }
  };
  await session.send('Runtime.enable');
  session.on('Runtime.consoleAPICalled', collect);
  try {
    // Initialize the session before subscribing, and wait for registration before emitting the log.
    await session.send('Runtime.evaluate', {expression: 'undefined'});
    events.length = 0;
    reportError(message, ...details);
    await session.send('Runtime.evaluate', {expression: 'undefined'});
    await expect.poll(() => events.length).toBe(1);
    return events[0] ?? [];
  } finally {
    session.off('Runtime.consoleAPICalled', collect);
    await session.send('Runtime.evaluate', {expression: 'undefined'});
  }
}

describe('reportError console adapter', () => {
  it.each(cases)('writes $name once to the browser error console', async ({name, details}) => {
    const message = `reportError ${name}`;
    const arguments_ = await captureErrorOutput(message, details);
    const actual = await Promise.all(arguments_.map(argument => readArgument(cdp(), argument)));
    expect(actual).toEqual([
      {type: 'string', value: message},
      ...details.map(detail =>
        detail instanceof Error
          ? {type: 'error', value: {name: detail.name, message: detail.message, stack: detail.stack}}
          : {type: typeof detail, value: detail},
      ),
    ]);
  });
});
