import {useMemo, useState} from 'react';

import {useQueryErrorResetBoundary} from '@tanstack/react-query';
import type {ErrorComponentProps} from '@tanstack/react-router';

import {Button, Theme} from '@radix-ui/themes';

import styles from './AppErrorComponent.module.css';
import type {StackFrame} from './stackTrace';
import {formatOriginLocation, parseFrame} from './stackTrace';

const StackFrameRow = ({frame}: {frame: StackFrame}) => {
  const className = frame.isExternal ? `${styles.frameRow} ${styles.frameExternal}` : styles.frameRow;
  return (
    <div className={className}>
      <span className={styles.frameName}>{frame.name}</span>
      <span className={styles.frameAt}> @ </span>
      <span className={styles.frameLocation}>{frame.location}</span>
    </div>
  );
};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const AppErrorComponent = ({error, reset}: ErrorComponentProps) => {
  const {reset: resetQueries} = useQueryErrorResetBoundary();
  const [stackExpanded, setStackExpanded] = useState(false);

  const {frames, originFrame} = useMemo(() => {
    // Strip the first line (error name + message) to avoid duplication
    const parsed = (error.stack?.split('\n').slice(1) ?? []).map(parseFrame).filter((f): f is StackFrame => f !== null);
    return {frames: parsed, originFrame: parsed.find(f => !f.isExternal)};
  }, [error]);

  return (
    <Theme appearance='dark' hasBackground={false}>
      <div className={styles.root}>
        <div className={styles.accentBar} />

        <div className={styles.header}>
          <span className={styles.badge}>Runtime Error</span>
          <span className={styles.errorName}>{error.name}</span>
        </div>

        <div className={styles.body}>
          <p className={styles.message}>{error.message}</p>
        </div>

        {originFrame && (
          <div className={styles.origin}>
            <span className={styles.originIcon}>at</span>
            <span>{formatOriginLocation(originFrame.location)}</span>
          </div>
        )}

        {frames.length > 0 && (
          <div className={styles.stackSection}>
            <button className={styles.expandButton} onClick={() => setStackExpanded(v => !v)} type='button'>
              <span className={styles.chevron}>{stackExpanded ? '▾' : '▸'}</span>
              Stack trace
            </button>
            <div className={styles.stackBody} data-expanded={stackExpanded || undefined}>
              <div className={styles.stackCollapse}>
                <div className={styles.stackScroll}>
                  {frames.map(frame => (
                    <StackFrameRow frame={frame} key={`${frame.name}@${frame.location}`} />
                  ))}
                </div>
              </div>
            </div>
          </div>
        )}

        <div className={styles.footer}>
          <Button
            onClick={() => {
              resetQueries();
              reset();
            }}
            variant='surface'
          >
            Try Again
          </Button>
        </div>
      </div>
    </Theme>
  );
};
