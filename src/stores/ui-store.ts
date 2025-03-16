import { atom, useAtomValue } from 'jotai'

export const devModeAtom = atom(false)
export const useDevMode = () => useAtomValue(devModeAtom)
