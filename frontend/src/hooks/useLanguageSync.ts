import { useSettings } from '@/contexts/SettingsContext'
import { useEffect } from 'react'
import i18n from '@/i18n'

/**
 * Sync i18n language with app settings.
 * Drop this into App.tsx or a top-level component.
 */
export function useLanguageSync() {
  const { settings } = useSettings()

  useEffect(() => {
    const lang = settings.general.language
    if (lang && i18n.language !== lang) {
      i18n.changeLanguage(lang)
    }
  }, [settings.general.language])
}
