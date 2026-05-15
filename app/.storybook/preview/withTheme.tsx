import {Theme} from '@radix-ui/themes';

import type {Decorator} from '@storybook/react-vite';

declare module '@storybook/react-vite' {
  // biome-ignore lint/style/useConsistentTypeDefinitions: TypeScript module augmentation requires `interface`; `type` cannot extend an existing interface from another module.
  interface Parameters {
    /**
     * Story configuration consumed by the {@link withTheme} decorator.
     * Has no effect when the decorator is not applied to the story.
     */
    theme?: {
      /** Render the story without wrapping it in the Radix Theme. */
      skip?: boolean;
    };
  }
}

export const withTheme: Decorator = (Story, {parameters}) => {
  if (parameters.theme?.skip === true) {
    return <Story />;
  }
  return (
    <Theme panelBackground='solid'>
      <Story />
    </Theme>
  );
};
