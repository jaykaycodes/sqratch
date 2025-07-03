import { createQueryKeyStore, type inferQueryKeyStore } from '@lukemorales/query-key-factory'

import { taurpc } from './utils'

const querySchema = {
	project: {
		get: {
			queryKey: null,
			queryFn: taurpc.projects.get_project,
		},
	},
	db: {
		isConnected: {
			queryKey: null,
			queryFn: taurpc.db.is_connected,
		},
		entities: {
			queryKey: null,
			queryFn: taurpc.db.get_all_entities,
		},
	},
} as const satisfies Parameters<typeof createQueryKeyStore>[0]

const Q = createQueryKeyStore(querySchema)

export default Q

export type QueryKeys = inferQueryKeyStore<typeof Q>

export type QueryData = {
	[TKey in keyof typeof querySchema]: {
		[TNestedKey in keyof (typeof querySchema)[TKey]]: (typeof querySchema)[TKey][TNestedKey] extends {
			queryFn: infer Fn extends (...args: any) => any
		}
			? Awaited<ReturnType<Fn>>
			: never
	}
}
