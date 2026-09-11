import { computed, ref, watch } from 'vue'
import {
  cloneValidationProfile,
  createValidationProfile,
  findValidationProfile,
  getValidationProfileMissingFields,
  isValidationProfileComplete,
  loadSelectedValidationProfileId,
  loadValidationProfiles,
  saveSelectedValidationProfileId,
  saveValidationProfiles,
} from '../lib/validation-profiles.js'

export function useValidationProfiles() {
  let preserveExpandedStateOnNextSelection = false
  const validationProfiles = ref(loadValidationProfiles())
  const selectedValidationProfileId = ref(loadSelectedValidationProfileId(validationProfiles.value))
  const selectedValidationProfile = findValidationProfile(validationProfiles.value, selectedValidationProfileId.value)
  const validationProfile = ref(selectedValidationProfile ? cloneValidationProfile(selectedValidationProfile) : createValidationProfile())
  const isValidationConfigExpanded = ref(false)

  const missingValidationFields = computed(() => getValidationProfileMissingFields(validationProfile.value))
  const isValidationConfigComplete = computed(() => isValidationProfileComplete(validationProfile.value))

  watch(selectedValidationProfileId, (nextId) => {
    const profile = findValidationProfile(validationProfiles.value, nextId)
    if (!profile) {
      return
    }

    if (preserveExpandedStateOnNextSelection) {
      preserveExpandedStateOnNextSelection = false
    } else {
      isValidationConfigExpanded.value = false
    }
    validationProfile.value = cloneValidationProfile(profile)
    saveSelectedValidationProfileId(nextId)
  })

  function saveProfile() {
    const draft = cloneValidationProfile(validationProfile.value)
    const index = validationProfiles.value.findIndex((profile) => profile.id === draft.id)
    if (index === -1) {
      validationProfiles.value = [...validationProfiles.value, draft]
    } else {
      const nextProfiles = [...validationProfiles.value]
      nextProfiles[index] = draft
      validationProfiles.value = nextProfiles
    }

    saveValidationProfiles(validationProfiles.value)
    selectedValidationProfileId.value = draft.id
    isValidationConfigExpanded.value = false
    return draft
  }

  function createProfile() {
    const profile = createValidationProfile(`测试库 ${validationProfiles.value.length + 1}`)
    validationProfiles.value = [...validationProfiles.value, profile]
    saveValidationProfiles(validationProfiles.value)
    preserveExpandedStateOnNextSelection = true
    selectedValidationProfileId.value = profile.id
    validationProfile.value = cloneValidationProfile(profile)
    isValidationConfigExpanded.value = true
    return profile
  }

  function deleteProfile() {
    if (!selectedValidationProfileId.value) {
      return null
    }

    const remainingProfiles = validationProfiles.value.filter((profile) => profile.id !== selectedValidationProfileId.value)
    if (remainingProfiles.length === 0) {
      const replacement = createValidationProfile()
      validationProfiles.value = [replacement]
      saveValidationProfiles(validationProfiles.value)
      preserveExpandedStateOnNextSelection = true
      selectedValidationProfileId.value = replacement.id
      validationProfile.value = cloneValidationProfile(replacement)
      isValidationConfigExpanded.value = true
      return replacement
    }

    validationProfiles.value = remainingProfiles
    saveValidationProfiles(validationProfiles.value)
    selectedValidationProfileId.value = remainingProfiles[0].id
    validationProfile.value = cloneValidationProfile(remainingProfiles[0])
    isValidationConfigExpanded.value = false
    return remainingProfiles[0]
  }

  return {
    validationProfiles,
    selectedValidationProfileId,
    validationProfile,
    isValidationConfigExpanded,
    missingValidationFields,
    isValidationConfigComplete,
    saveProfile,
    createProfile,
    deleteProfile,
  }
}
