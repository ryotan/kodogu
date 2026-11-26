import {sb} from 'storybook/test';

import {preview} from '#storybook/preview';

sb.mock(import('../src/reportError.ts'), {spy: true});
sb.mock(import('../src/commands/popover.ts'), {spy: true});

export default {...preview};
