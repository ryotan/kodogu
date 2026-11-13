const messages = {
  configuration_rejected: 'Configuration was not saved. Your input has been kept.',
  configuration_unavailable: 'Configuration is unavailable. Restart KoDoGu before trying again.',
  configuration_unrepresentable:
    'Saved configuration cannot be displayed safely by this interface. Your data has been kept.',
  result_unavailable: 'Could not confirm the settings result. Read the current settings again.',
} as const;

type ConfigurationErrorCode = keyof typeof messages;

export class ConfigurationCommandError extends Error {
  constructor(readonly code: ConfigurationErrorCode) {
    super(messages[code]);
    this.name = 'ConfigurationCommandError';
  }

  get unavailable(): boolean {
    return this.code !== 'configuration_rejected';
  }

  static from(error: unknown): ConfigurationCommandError {
    if (error instanceof ConfigurationCommandError) {
      return error;
    }
    if (
      typeof error === 'object' &&
      error !== null &&
      'code' in error &&
      'message' in error &&
      typeof error.message === 'string'
    ) {
      switch (error.code) {
        case 'configuration_rejected':
        case 'configuration_unavailable':
        case 'configuration_unrepresentable':
          return new ConfigurationCommandError(error.code);
      }
    }
    return new ConfigurationCommandError('result_unavailable');
  }
}
