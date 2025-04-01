import { atom, useAtomValue, type WritableAtom } from 'jotai'
import {
	atomWithStorage,
	createJSONStorage,
	type RESET as RESET_STORAGE,
	unstable_withStorageValidator as withStorageValidator,
} from 'jotai/utils'
import type { z } from 'zod'

import type { FileRoutesById } from '#/routeTree.gen'

type SetStateActionWithReset<T> = T | typeof RESET_STORAGE | ((prev: T) => T | typeof RESET_STORAGE)
type AtomWithTypedStorage<
	TSchema extends z.ZodTypeAny,
	TOutput extends z.output<TSchema>,
	TInput extends z.input<TSchema>,
> = WritableAtom<TOutput, [SetStateActionWithReset<TInput>], void>
export function atomWithTypedStorage<
	TSchema extends z.ZodTypeAny,
	TOutput extends z.output<TSchema>,
	TInput extends z.input<TSchema>,
>(
	key: string,
	schema: TSchema,
	defaultValue: TOutput,
	options?: Parameters<typeof atomWithStorage>[3],
): AtomWithTypedStorage<TSchema, TOutput, TInput> {
	const isValid = (v: unknown): v is TOutput => schema.safeParse(v).success
	return atomWithStorage(
		key,
		defaultValue,
		withStorageValidator(isValid)(createJSONStorage()),
		options,
	)
}

/** Returns an atom synced with local storage for a given page's state */
export const atomForRoute = <TRoute extends keyof FileRoutesById, TSchema extends z.ZodTypeAny>(
	routeId: TRoute,
	schema: TSchema,
	defaultValue: z.output<TSchema>,
	options?: Parameters<typeof atomWithTypedStorage>[3],
) => atomWithTypedStorage(routeId, schema, defaultValue, { ...options, getOnInit: true })

export const devModeAtom = atom(false)
export const useDevMode = () => useAtomValue(devModeAtom)
