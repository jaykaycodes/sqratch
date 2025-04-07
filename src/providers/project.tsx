import { observable } from '@legendapp/state'
import { useMount } from '@legendapp/state/react'

import { taurpc } from '#/lib/queries'

export type ConnectionStatus = 'connected' | 'disconnected' | 'connecting' | 'disconnecting'

const REFRESH_INTERVAL = 30_000

// Create project state with computed and nested observables
const project$ = observable({
	name: '',
	status: 'disconnected' as ConnectionStatus,
	// Computed value - gets loaded when accessed
	connectionString: () => taurpc.db.get_connection_string(),
})

export default project$

export function useProjectInit() {
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
