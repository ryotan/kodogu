import {describe, expect, it} from 'vitest';

import {ApplicationError} from './ApplicationError';
import {getUserMessage} from './getUserMessage';
import {PlatformError} from './PlatformError';
import {RootError} from './RootError';

describe('getUserMessage', () => {
  it('returns the message verbatim for RootError instances', () => {
    expect(getUserMessage(new RootError('保存に失敗しました'))).toBe('保存に失敗しました');
  });

  it('returns the message verbatim for ApplicationError instances', () => {
    expect(getUserMessage(new ApplicationError('入力に不備があります'))).toBe('入力に不備があります');
  });

  it('returns the message verbatim for PlatformError instances', () => {
    expect(getUserMessage(new PlatformError('接続情報の読み込みに失敗しました'))).toBe(
      '接続情報の読み込みに失敗しました',
    );
  });

  it('falls back for plain Error instances (not in the RootError hierarchy)', () => {
    expect(getUserMessage(new Error('something broke'))).toBe('予期しないエラーが発生しました');
  });

  it('falls back for non-Error values', () => {
    expect(getUserMessage('a string was thrown')).toBe('予期しないエラーが発生しました');
    expect(getUserMessage(undefined)).toBe('予期しないエラーが発生しました');
    expect(getUserMessage(null)).toBe('予期しないエラーが発生しました');
    expect(getUserMessage({message: 'duck-typed'})).toBe('予期しないエラーが発生しました');
  });
});
