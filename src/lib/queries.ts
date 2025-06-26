import { createQueryKeyStore, type inferQueryKeyStore } from '@lukemorales/query-key-factory'

import { taurpc } from './utils'

const Q = createQueryKeyStore({
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
})

export default Q

export type QueryKeys = inferQueryKeyStore<typeof Q>
