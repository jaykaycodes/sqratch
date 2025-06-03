import { observable, when } from '@legendapp/state'
import { synced } from '@legendapp/state/sync'

import { taurpc } from '#/lib/utils'
import { type WorkbenchItem } from '#/routes/-project/workbench'

export type ConnectionStatus = 'connected' | 'disconnected' | 'loading'

const connectionStatus$ = observable('loading' as ConnectionStatus)

const queries$ = synced({
	activate: 'auto',
	get: async (): Promise<WorkbenchItem[]> => [],
})

const entities$ = synced({
	activate: 'auto',
	get: async () => {
		await when(() => connectionStatus$.get() === 'connected')
		return taurpc.db.get_all_entities()
	},
})

const projectStore$ = observable({
	name: undefined as string | undefined,
	connectionString: undefined as string | undefined,
	connectionStatus: connectionStatus$,
	queries: queries$,
	entities: entities$,
	favorites: new Set<string>(),
	async connect(connect = true) {
		const fn = connect ? taurpc.db.connect : taurpc.db.disconnect

		if (connectionStatus$.get() === 'disconnected') connectionStatus$.set('loading')

		try {
			await fn()
			connectionStatus$.set(connect ? 'connected' : 'disconnected')
		} catch (error) {
			console.error(error)
			connectionStatus$.set('disconnected')
		}
	},
})
