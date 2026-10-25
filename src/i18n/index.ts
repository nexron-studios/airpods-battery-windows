import { createI18n } from 'vue-i18n'
import de from '@/i18n/locales/de'
import en from '@/i18n/locales/en'

const FALLBACK_LOCALE = 'de'
const localeOf = (language: string): 'de' | 'en' => (language.startsWith('en') ? 'en' : FALLBACK_LOCALE)

export default createI18n({
  legacy: false,
  locale: localeOf(navigator.language),
  fallbackLocale: FALLBACK_LOCALE,
  messages: { de, en },
})
