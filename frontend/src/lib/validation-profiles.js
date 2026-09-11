const profilesStorageKey = 'sql-processor.validationProfiles'
const selectedProfileStorageKey = 'sql-processor.selectedValidationProfileId'

function canUseStorage() {
  return typeof window !== 'undefined' && typeof window.localStorage !== 'undefined'
}

function nextProfileName(index) {
  return `测试库 ${index}`
}

export function createValidationProfile(name = nextProfileName(1)) {
  return {
    id: `validation-profile-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    profileName: name,
    enabled: false,
    host: '',
    port: '1521',
    service: '',
    username: '',
    password: '',
    timeoutSeconds: '5',
  }
}

export function cloneValidationProfile(profile) {
  return {
    id: profile?.id || '',
    profileName: profile?.profileName || '',
    enabled: Boolean(profile?.enabled),
    host: profile?.host || '',
    port: profile?.port || '1521',
    service: profile?.service || '',
    username: profile?.username || '',
    password: profile?.password || '',
    timeoutSeconds: profile?.timeoutSeconds || '5',
  }
}

function normalizeProfiles(rawProfiles) {
  if (!Array.isArray(rawProfiles) || rawProfiles.length === 0) {
    return [createValidationProfile()]
  }

  return rawProfiles.map((profile, index) => ({
    ...createValidationProfile(nextProfileName(index + 1)),
    ...cloneValidationProfile(profile),
    id: profile?.id || createValidationProfile().id,
    profileName: profile?.profileName || nextProfileName(index + 1),
    password: '',
  }))
}

function profilesForStorage(profiles) {
  return normalizeProfiles(profiles).map((profile) => ({
    ...profile,
    password: '',
  }))
}

export function loadValidationProfiles() {
  if (!canUseStorage()) {
    return [createValidationProfile()]
  }

  try {
    const raw = window.localStorage.getItem(profilesStorageKey)
    if (!raw) {
      return [createValidationProfile()]
    }
    return normalizeProfiles(JSON.parse(raw))
  } catch {
    return [createValidationProfile()]
  }
}

export function saveValidationProfiles(profiles) {
  if (!canUseStorage()) {
    return
  }
  window.localStorage.setItem(profilesStorageKey, JSON.stringify(profilesForStorage(profiles)))
}

export function loadSelectedValidationProfileId(profiles) {
  if (!canUseStorage()) {
    return profiles[0]?.id || ''
  }

  const selectedId = window.localStorage.getItem(selectedProfileStorageKey)
  if (selectedId && profiles.some((profile) => profile.id === selectedId)) {
    return selectedId
  }
  return profiles[0]?.id || ''
}

export function saveSelectedValidationProfileId(profileId) {
  if (!canUseStorage()) {
    return
  }
  window.localStorage.setItem(selectedProfileStorageKey, profileId)
}

export function findValidationProfile(profiles, profileId) {
  return profiles.find((profile) => profile.id === profileId) || null
}

export function getValidationProfileMissingFields(profile) {
  const missing = []
  if (!profile?.host?.trim()) {
    missing.push('host')
  }
  if (!String(profile?.port || '').trim() || Number(profile.port) <= 0) {
    missing.push('port')
  }
  if (!profile?.service?.trim()) {
    missing.push('service')
  }
  if (!profile?.username?.trim()) {
    missing.push('username')
  }
  if (!profile?.password?.trim()) {
    missing.push('password')
  }
  return missing
}

export function isValidationProfileComplete(profile) {
  return getValidationProfileMissingFields(profile).length === 0
}

export function buildValidationConfig(profile) {
  if (!profile) {
    return null
  }

  return {
    enabled: Boolean(profile.enabled),
    profileName: profile.profileName || '',
    host: profile.host.trim(),
    port: Number(profile.port) || 0,
    service: profile.service.trim(),
    username: profile.username.trim(),
    password: profile.password,
    timeoutSeconds: Number(profile.timeoutSeconds) || 0,
  }
}
