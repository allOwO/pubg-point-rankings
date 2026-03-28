export type ActivitySentenceType = 'knock' | 'kill' | 'revive';

interface FormatActivitySentenceInput {
  locale: string;
  type: ActivitySentenceType;
  actionText: string;
  targetName: string;
  extra: string | null | undefined;
}

export interface ActivitySentenceParts {
  beforeTarget: string;
  afterTarget: string;
}

function isChineseLocale(locale: string): boolean {
  return locale.toLowerCase().startsWith('zh');
}

export function formatActivitySentenceParts(input: FormatActivitySentenceInput): ActivitySentenceParts {
  const { locale, type, actionText, targetName, extra } = input;
  const trimmedExtra = extra?.trim();

  if (isChineseLocale(locale)) {
    if (type !== 'revive' && trimmedExtra) {
      return {
        beforeTarget: `用 ${trimmedExtra} ${actionText}了 `,
        afterTarget: '',
      };
    }

    return {
      beforeTarget: `${actionText}了 `,
      afterTarget: '',
    };
  }

  if (trimmedExtra && type !== 'revive') {
    return {
      beforeTarget: `${actionText} `,
      afterTarget: ` with ${trimmedExtra}`,
    };
  }

  return {
    beforeTarget: `${actionText} `,
    afterTarget: '',
  };
}

export function formatActivitySentence(input: FormatActivitySentenceInput): string {
  const { targetName } = input;
  const { beforeTarget, afterTarget } = formatActivitySentenceParts(input);
  return `${beforeTarget}${targetName}${afterTarget}`;
}
