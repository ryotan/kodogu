import {ThemePanel} from '@radix-ui/themes';

export const RadixThemePanel = () => {
  if (!import.meta.env.DEV) {
    return null;
  }

  return <ThemePanel defaultOpen={false} />;
};
