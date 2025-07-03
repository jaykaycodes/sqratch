import { basename, sep } from '@tauri-apps/api/path'

import { type Observable, observable } from '@legendapp/state'
import { synced } from '@legendapp/state/sync'

import Q, { QueryData } from '#/lib/queries'
import { queryClient, taurpc } from '#/lib/utils'

export type ResourceType = 'table' | 'file'

interface BaseResource {
	id: string
	type: ResourceType
	name: string
	path: string[]
	isTemp?: boolean
	isDirty?: boolean
}

interface TableResource extends BaseResource {
	type: 'table'
}

interface FileResource extends BaseResource {
	type: 'file'
}

export type Resource = TableResource | FileResource

const createTableResource = (id: string) =>
	observable<TableResource>(() => {
		return {
			id,
			type: 'table',
			name: () => {
				const entities = queryClient
					.getQueryCache()
					.find<QueryData['db']['entities']>({ queryKey: Q.db.entities.queryKey })
				const entity = entities?.state.data?.[id]
				return entity?.name ?? id
			},
			path: () => [],
		}
	})

const createFileResource = (id: string) =>
	observable<FileResource>({
		id,
		type: 'file',
		name: () => basename(id),
		path: () => id.split(sep()).slice(0, -1).join(sep()),
	})

export const ResourceStore$ = observable({
	resources: new Map<string, Observable<Resource>>(),
	entities: synced({
		get: () => taurpc.db.get_all_entities(),
		activate: 'auto',

		initial: {},
	}),
})

ResourceStore$.entities.sdf.name
