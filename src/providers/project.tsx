import { observable } from '@legendapp/state'
import { useMount } from '@legendapp/state/react'
import { syncedQuery } from '@legendapp/state/sync-plugins/tanstack-query'

import queryClient, { taurpc } from '#/lib/queries'

export type ConnectionStatus = 'connected' | 'disconnected' | 'connecting' | 'disconnecting'

const REFRESH_INTERVAL = 30_000

const connectionString = syncedQuery({
	queryClient,
	query: {
		queryKey: ['connectionString'],
		queryFn: () => taurpc.db.get_connection_string(),
	},
})

const entities = syncedQuery({
	queryClient,
	query: {
		queryKey: ['entities'],
		queryFn: () => taurpc.db.get_entities(),
	},
})

const project$ = observable({
	name: '',
	status: 'disconnected' as ConnectionStatus,
	connectionString,
	entities,
})

export default project$

export function useInitProject() {
	// connect to the db
	useMount(() => {
		project$.status.set('connecting')
		taurpc.db
			.connect()
			.then(() => {
				project$.status.set('connected')
			})
			.catch((error) => {
				console.error('Error connecting to the database:', error)
				project$.status.set('disconnected')
			})

		// Poll for connection status
		const intervalId = setInterval(() => {
			taurpc.db
				.is_connected()
				.then((status) => {
					project$.status.set(status ? 'connected' : 'disconnected')
				})
				.catch((error) => {
					console.error('Error checking connection status:', error)
				})
		}, REFRESH_INTERVAL)

		// Clean up interval when component unmounts
		return () => clearInterval(intervalId)
	})
}
