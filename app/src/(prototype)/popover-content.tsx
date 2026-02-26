// biome-ignore-all lint: this is a prototype
/* eslint-disable */
/**
 * M1 layout verification prototype. Will be fully replaced with real components in M2.
 */
import {useCallback, useState} from 'react';

// Manages card expand/collapse state
function useCardToggle(initialOpen: string[]) {
  const [openCards, setOpenCards] = useState<Set<string>>(() => new Set(initialOpen));
  const toggle = useCallback((id: string) => {
    setOpenCards(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);
  return {isOpen: (id: string) => openCards.has(id), toggle};
}

export const colors = {
  surface: 'rgba(28, 28, 30, 0.92)',
  surfaceCard: 'rgba(44, 44, 46, 0.8)',
  surfaceExpanded: 'rgba(36, 36, 38, 0.95)',
  border: 'rgba(255, 255, 255, 0.08)',
  textPrimary: 'rgba(255, 255, 255, 0.92)',
  textSecondary: 'rgba(255, 255, 255, 0.45)',
  textTertiary: 'rgba(255, 255, 255, 0.28)',
  statusPrimary: '#ff453a',
  statusReplica: '#30d158',
  statusWarning: '#ffd60a',
  statusDisconnected: 'rgba(255, 255, 255, 0.3)',
  tagBg: 'rgba(255, 255, 255, 0.08)',
  tagText: 'rgba(255, 255, 255, 0.55)',
} as const;

const fontMono = "'SF Mono', 'Fira Code', monospace";

const SectionLabel = ({children}: {children: string}) => (
  <div
    style={{
      fontSize: 10,
      color: colors.textTertiary,
      textTransform: 'uppercase',
      letterSpacing: '0.06em',
      padding: '6px 16px 2px',
    }}
  >
    {children}
  </div>
);

const Tag = ({children}: {children: string}) => (
  <span
    style={{
      fontSize: 10,
      padding: '1px 5px',
      borderRadius: 4,
      background: colors.tagBg,
      color: colors.tagText,
      flexShrink: 0,
    }}
  >
    {children}
  </span>
);

const RoleBadge = ({role}: {role: 'primary' | 'replica'}) => {
  const isPrimary = role === 'primary';
  return (
    <span
      style={{
        fontSize: 10,
        fontWeight: 600,
        padding: '1px 5px',
        borderRadius: 4,
        letterSpacing: '0.02em',
        textTransform: 'uppercase',
        flexShrink: 0,
        background: isPrimary ? 'rgba(255, 69, 58, 0.18)' : 'rgba(48, 209, 88, 0.15)',
        color: isPrimary ? colors.statusPrimary : colors.statusReplica,
      }}
    >
      {isPrimary ? 'Primary' : 'Replica'}
    </span>
  );
};

const DetailRow = ({label, value, mono}: {label: string; value: string; mono?: boolean}) => (
  <div style={{display: 'flex', justifyContent: 'space-between', gap: 8, padding: '3px 0'}}>
    <span style={{fontSize: 11, color: colors.textSecondary, whiteSpace: 'nowrap', flexShrink: 0}}>{label}</span>
    <span
      style={{
        fontSize: 11,
        color: colors.textPrimary,
        textAlign: 'right',
        wordBreak: 'break-all',
        fontFamily: mono ? fontMono : undefined,
      }}
    >
      {value}
    </span>
  </div>
);

const DetailSep = () => <div style={{height: 1, background: colors.border, margin: '4px 0'}} />;

const StopIcon = () => (
  <svg fill='none' height={14} viewBox='0 0 14 14' width={14}>
    <rect fill={colors.statusPrimary} height={9} rx={1.5} width={9} x={2.5} y={2.5} />
  </svg>
);

const PlayIcon = () => (
  <svg fill='none' height={14} viewBox='0 0 14 14' width={14}>
    <circle cx={7} cy={7} r={5.5} stroke={colors.statusDisconnected} strokeWidth={1.5} />
    <path d='M5.5 4.5l4 2.5-4 2.5V4.5z' fill={colors.statusDisconnected} />
  </svg>
);

const WarningIcon = () => (
  <svg fill='none' height={14} viewBox='0 0 14 14' width={14}>
    <path d='M7 1.5L12.5 11H1.5L7 1.5z' fill={colors.statusWarning} />
    <text fill='#1a1a1c' fontSize={5} fontWeight={700} textAnchor='middle' x={7} y={9.5}>
      !
    </text>
  </svg>
);

const Chevron = ({open}: {open?: boolean}) => (
  <svg
    fill='none'
    height={16}
    viewBox='0 0 16 16'
    width={16}
    style={{
      flexShrink: 0,
      color: colors.textTertiary,
      transform: open ? 'rotate(180deg)' : undefined,
      transition: 'transform 0.2s',
    }}
  >
    <path d='M4 6l4 4 4-4' stroke='currentColor' strokeLinecap='round' strokeLinejoin='round' strokeWidth={1.5} />
  </svg>
);

const StatusButton = ({children}: {children: React.ReactNode}) => (
  <button
    type='button'
    style={{
      width: 28,
      height: 28,
      borderRadius: '50%',
      border: 'none',
      background: 'transparent',
      cursor: 'pointer',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      flexShrink: 0,
    }}
  >
    {children}
  </button>
);

const CardCollapsed = ({
  icon,
  name,
  role,
  tag,
  open,
  onClick,
}: {
  icon: React.ReactNode;
  name: string;
  role: 'primary' | 'replica';
  tag: string;
  open?: boolean;
  onClick?: () => void;
}) => (
  <div
    onClick={onClick}
    style={{
      display: 'flex',
      alignItems: 'center',
      gap: 8,
      padding: '8px 10px',
      cursor: 'pointer',
      background: colors.surfaceCard,
      borderRadius: open ? '10px 10px 0 0' : 10,
      userSelect: 'none',
    }}
  >
    <StatusButton>{icon}</StatusButton>
    <div style={{flex: 1, minWidth: 0, display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap'}}>
      <span
        style={{
          fontSize: 13,
          fontWeight: 500,
          color: colors.textPrimary,
          whiteSpace: 'nowrap',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
        }}
      >
        {name}
      </span>
      <RoleBadge role={role} />
      <Tag>{tag}</Tag>
    </div>
    <Chevron open={open} />
  </div>
);

const CardBody = ({children}: {children: React.ReactNode}) => (
  <div style={{background: colors.surfaceExpanded, borderTop: `1px solid ${colors.border}`}}>
    <div style={{padding: '8px 12px 10px', display: 'flex', flexDirection: 'column', gap: 1}}>{children}</div>
  </div>
);

const ConnectionDetails = ({
  project,
  region,
  instance,
  database,
  ipType,
  auth,
  iam,
  address,
  port,
}: {
  project: string;
  region: string;
  instance: string;
  database: string;
  ipType: string;
  auth: string;
  iam: string;
  address: string;
  port: string;
}) => (
  <>
    <DetailRow label='プロジェクト' mono value={project} />
    <DetailRow label='リージョン' value={region} />
    <DetailRow label='インスタンス' mono value={instance} />
    <DetailRow label='データベース' mono value={database} />
    <DetailRow label='IP タイプ' value={ipType} />
    <DetailRow label='認証' value={auth} />
    <DetailRow label='IAM 認証' value={iam} />
    <DetailSep />
    <DetailRow label='アドレス' mono value={address} />
    <DetailRow label='ローカルポート' mono value={port} />
  </>
);

// =============================================================================
// Exported stub components — will be replaced with real components in M2
// =============================================================================

export const StubHeader = () => (
  <div
    style={{marginLeft: 14, display: 'flex', alignItems: 'center', gap: 6, pointerEvents: 'none', userSelect: 'none'}}
  >
    <div
      style={{
        width: 8,
        height: 8,
        borderRadius: '50%',
        background: colors.statusPrimary,
        boxShadow: `0 0 6px ${colors.statusPrimary}`,
      }}
    />
    <span style={{fontSize: 13, fontWeight: 500, color: colors.textPrimary}}>2接続中 / 4プロファイル</span>
  </div>
);

export const StubCardList = () => {
  const {isOpen, toggle} = useCardToggle(['card1', 'card2']);
  return (
    <>
      <SectionLabel>接続中</SectionLabel>
      <div
        style={{
          margin: '2px 6px',
          borderRadius: 10,
          overflow: 'hidden',
          border: isOpen('card1') ? `1px solid ${colors.border}` : undefined,
        }}
      >
        <CardCollapsed
          icon={<StopIcon />}
          name='prod-db (write)'
          onClick={() => toggle('card1')}
          open={isOpen('card1')}
          role='primary'
          tag='prod'
        />
        {isOpen('card1') && (
          <CardBody>
            <ConnectionDetails
              address='127.0.0.1'
              auth='ADC'
              database='myapp_production'
              iam='有効'
              instance='prod-postgres-instance'
              ipType='Private'
              port='5432'
              project='my-gcp-project'
              region='asia-northeast1'
            />
            <DetailSep />
            <div style={{display: 'flex', justifyContent: 'space-between', padding: '3px 0'}}>
              <span style={{fontSize: 11, color: colors.textSecondary}}>接続時間</span>
              <span style={{fontFamily: fontMono, fontSize: 11, color: colors.textSecondary}}>00:32:10</span>
            </div>
          </CardBody>
        )}
      </div>
      <div
        style={{
          margin: '2px 6px',
          borderRadius: 10,
          overflow: 'hidden',
          border: isOpen('card2') ? `1px solid ${colors.border}` : undefined,
        }}
      >
        <CardCollapsed
          icon={<WarningIcon />}
          name='prod-db (read)'
          onClick={() => toggle('card2')}
          open={isOpen('card2')}
          role='replica'
          tag='prod'
        />
        {isOpen('card2') && (
          <CardBody>
            <ConnectionDetails
              address='127.0.0.1'
              auth='ADC'
              database='myapp_production'
              iam='有効'
              instance='prod-postgres-instance'
              ipType='Private'
              port='5433'
              project='my-gcp-project'
              region='asia-northeast1'
            />
            <DetailSep />
            <div style={{display: 'flex', justifyContent: 'space-between', padding: '3px 0'}}>
              <span style={{fontSize: 11, color: colors.textSecondary}}>接続時間</span>
              <span style={{fontFamily: fontMono, fontSize: 11, color: colors.statusWarning}}>01:05:42</span>
            </div>
            <div
              style={{
                marginTop: 6,
                fontSize: 11,
                color: colors.statusWarning,
                background: 'rgba(255, 214, 10, 0.08)',
                border: '1px solid rgba(255, 214, 10, 0.15)',
                borderRadius: 6,
                padding: '5px 8px',
              }}
            >
              60分以上接続中です。不要な場合は切断してください。
            </div>
          </CardBody>
        )}
      </div>
      <SectionLabel>未接続</SectionLabel>
      <div
        style={{
          margin: '2px 6px',
          borderRadius: 10,
          overflow: 'hidden',
          border: isOpen('card3') ? `1px solid ${colors.border}` : undefined,
        }}
      >
        <CardCollapsed
          icon={<PlayIcon />}
          name='staging-db'
          onClick={() => toggle('card3')}
          open={isOpen('card3')}
          role='primary'
          tag='staging'
        />
        {isOpen('card3') && (
          <CardBody>
            <ConnectionDetails
              address='127.0.0.1'
              auth='ADC'
              database='myapp_staging'
              iam='無効'
              instance='staging-postgres-instance'
              ipType='Public'
              port='5434'
              project='my-gcp-project'
              region='us-central1'
            />
            <DetailSep />
            <div style={{display: 'flex', justifyContent: 'space-between', padding: '3px 0'}}>
              <span style={{fontSize: 11, color: colors.textSecondary}}>最終接続</span>
              <span style={{fontSize: 11, color: colors.textSecondary}}>2026-02-24 09:15</span>
            </div>
          </CardBody>
        )}
      </div>
      <div
        style={{
          margin: '2px 6px',
          borderRadius: 10,
          overflow: 'hidden',
          border: isOpen('card4') ? `1px solid ${colors.border}` : undefined,
        }}
      >
        <CardCollapsed
          icon={<PlayIcon />}
          name='dev-db'
          onClick={() => toggle('card4')}
          open={isOpen('card4')}
          role='replica'
          tag='dev'
        />
        {isOpen('card4') && (
          <CardBody>
            <ConnectionDetails
              address='127.0.0.1'
              auth='ADC'
              database='myapp_dev'
              iam='有効'
              instance='dev-postgres-instance'
              ipType='Public'
              port='5435'
              project='my-gcp-project'
              region='asia-northeast1'
            />
            <DetailSep />
            <div style={{display: 'flex', justifyContent: 'space-between', padding: '3px 0'}}>
              <span style={{fontSize: 11, color: colors.textSecondary}}>最終接続</span>
              <span style={{fontSize: 11, color: colors.textTertiary}}>—</span>
            </div>
            <div
              style={{
                marginTop: 6,
                fontSize: 11,
                background: 'rgba(255, 69, 58, 0.08)',
                border: '1px solid rgba(255, 69, 58, 0.15)',
                borderRadius: 6,
                overflow: 'hidden',
              }}
            >
              <div style={{padding: '5px 8px', color: colors.statusPrimary, fontWeight: 500}}>認証エラー</div>
            </div>
          </CardBody>
        )}
      </div>
    </>
  );
};

export const StubFooter = () => (
  <button
    type='button'
    style={{
      fontSize: 12,
      color: colors.textSecondary,
      background: 'none',
      border: 'none',
      cursor: 'pointer',
      padding: '2px 4px',
      borderRadius: 4,
    }}
  >
    Quit
  </button>
);
