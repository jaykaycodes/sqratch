import React, { useMemo } from 'react'

import { observable } from '@legendapp/state'
import { use$, useMount } from '@legendapp/state/react'
import { createQueryKeys } from '@lukemorales/query-key-factory'
import { useQuery } from '@tanstack/react-query'

import { taurpc } from '#/lib/utils'

export type ConnectionStatus = 'connected' | 'disconnected' | 'loading'

// const STATUS_POLL_INTERVAL = 1000 * 30

const queryKeys = createQueryKeys('db', {
	schemas: {
		queryKey: null,
		queryFn: () => taurpc.db.get_all_schemas(),
	},
})

const createDbStore = (connectionString: string) => {
	let name = connectionString.split('/').pop()
	try {
		const url = new URL(connectionString)
		name = url.hostname.split(':')[0]
	} catch (error) {
		console.error(error)
	}

	const store$ = observable({
		name,
		connectionStatus: 'disconnected' as ConnectionStatus,
		connectionString,
		async connect(disconnect = false) {
			const fn = disconnect ? taurpc.db.disconnect : taurpc.db.connect

			if (store$.connectionStatus.get() === 'disconnected') store$.connectionStatus.set('loading')

			try {
				await fn()
				store$.connectionStatus.set(disconnect ? 'disconnected' : 'connected')
			} catch (error) {
				console.error(error)
				store$.connectionStatus.set('disconnected')
			}
		},
	})

	return store$
}

// biome-ignore lint/style/noNonNullAssertion: <explanation>
const DbContext = React.createContext<ReturnType<typeof createDbStore>>(null!)

export const DbProvider = ({
	children,
	connectionString,
}: {
	children: React.ReactNode
	connectionString: string
}) => {
	const store$ = useMemo(() => createDbStore(connectionString), [connectionString])

	useMount(() => {
		store$.connect()
	})

	return <DbContext.Provider value={store$}>{children}</DbContext.Provider>
}

export const useDbStore$ = () => React.useContext(DbContext)

export const useSchemasQuery = () => {
	const store$ = useDbStore$()
	const enabled = use$(() => store$.connectionStatus.get() === 'connected')

	return useQuery({
		...queryKeys.schemas,
		initialData: [],
		enabled,
		throwOnError: true,
	})
}
