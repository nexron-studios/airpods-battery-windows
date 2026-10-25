const PERCENT_SCALE = 100

export const formatPercent = (locale: string, percent: number): string =>
  new Intl.NumberFormat(locale, { style: 'percent' }).format(percent / PERCENT_SCALE)
