import {ThemePanel} from '@radix-ui/themes';

export const RadixThemePanel = () => {
  if (!import.meta.env.DEV) {
    return <></>;
  }

  return <ThemePanel defaultOpen={false} />;
};
